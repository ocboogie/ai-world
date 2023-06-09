use crate::{
    connection::{self, Connection},
    innovation_record::InnovationRecord,
    node::Node,
};
use rand::seq::SliceRandom;
use rand::Rng;
use rand_distr::{Distribution, Normal, StandardNormal};
use std::{
    collections::{hash_map::DefaultHasher, HashSet},
    hash::{Hash, Hasher},
    ops::{Index, IndexMut, RangeInclusive},
};

const BIAS: f32 = 1.0;

const INTIAL_WEIGHT_RANGE: RangeInclusive<f32> = -1.0..=1.0;

const MUTATE_WEIGHTS_RATE: f64 = 0.90;
const MUTATE_PERTURB_WEIGHT_RATE: f64 = 0.90;
const MUTATE_WEIGHT_POWER: f32 = 0.5;
const MUTATE_REPLACE_RANGE: RangeInclusive<f32> = -30.0..=30.0;
const MUTATE_NEW_NODE_RATE: f64 = 0.2;
const MUTATE_NEW_CONNECTION_RATE: f64 = 0.5;

const CROSSOVER_PICK_FITTEST_CONNECTION_PROB: f64 = 0.9;
const CROSSOVER_DISABLE_CONNECTION_PROB: f64 = 0.75;

const DIST_DISJOINT_FACTOR: f32 = 1.0;
const DIST_WEIGHT_DIFFERENCE_FACTOR: f32 = 2.0;

#[derive(Debug, Clone, Hash)]
pub struct Genome<const INPUT_SZ: usize, const OUTPUT_SZ: usize> {
    pub hidden_nodes: usize,
    // TODO: Make innovation number the index into a hash set instead of using
    // an vector
    pub connections: Vec<Connection<INPUT_SZ, OUTPUT_SZ>>,
}

#[derive(Debug, Clone)]
pub struct GenomeActivation<const INPUT_SZ: usize, const OUTPUT_SZ: usize> {
    pub input: [f32; INPUT_SZ],
    pub output: [f32; OUTPUT_SZ],
    pub hidden: Vec<f32>,
}

impl<const INPUT_SZ: usize, const OUTPUT_SZ: usize> GenomeActivation<INPUT_SZ, OUTPUT_SZ> {
    pub fn new(input: [f32; INPUT_SZ], hidden_nodes: usize) -> Self {
        Self {
            input,
            output: [0.; OUTPUT_SZ],
            hidden: vec![0.; hidden_nodes],
        }
    }
}

/// 0 = Bias
/// 1:INPUT_SZ + 1 = Input nodes
/// INPUT_SZ + 1:OUTPUT_SZ + INPUT_SZ + 1 = Output nodes
/// OUTPUT_SZ + INPUT_SZ + 1: = Hidden nodes
impl<const INPUT_SZ: usize, const OUTPUT_SZ: usize> Index<Node<INPUT_SZ, OUTPUT_SZ>>
    for GenomeActivation<INPUT_SZ, OUTPUT_SZ>
{
    type Output = f32;

    fn index(&self, node: Node<INPUT_SZ, OUTPUT_SZ>) -> &Self::Output {
        if node.is_bias() {
            return &BIAS;
        }

        if node.is_input() {
            return &self.input[node.0 - 1];
        }

        if node.is_output() {
            return &self.output[node.0 - INPUT_SZ - 1];
        }

        &self.hidden[node.0 - INPUT_SZ - OUTPUT_SZ - 1]
    }
}

impl<const INPUT_SZ: usize, const OUTPUT_SZ: usize> IndexMut<Node<INPUT_SZ, OUTPUT_SZ>>
    for GenomeActivation<INPUT_SZ, OUTPUT_SZ>
{
    fn index_mut(&mut self, node: Node<INPUT_SZ, OUTPUT_SZ>) -> &mut Self::Output {
        if node.is_bias() {
            panic!("Can't mutate bias node which is at index 0");
        }

        if node.is_input() {
            return &mut self.input[node.0 - 1];
        }

        if node.is_output() {
            return &mut self.output[node.0 - INPUT_SZ - 1];
        }

        &mut self.hidden[node.0 - INPUT_SZ - OUTPUT_SZ - 1]
    }
}

impl<const INPUT_SZ: usize, const OUTPUT_SZ: usize> Genome<INPUT_SZ, OUTPUT_SZ> {
    pub fn new() -> Self {
        Self {
            hidden_nodes: 0,
            connections: Vec::new(),
        }
    }

    pub fn identifier(&self) -> String {
        let mut hasher = DefaultHasher::default();

        self.hash(&mut hasher);

        format!("{:x}", hasher.finish()).to_string()
    }

    pub fn new_random_initial(
        rng: &mut impl Rng,
        innovation_record: &mut InnovationRecord<INPUT_SZ, OUTPUT_SZ>,
    ) -> Self {
        let mut res = Self::new();

        for i in 0..(INPUT_SZ + 1) {
            for j in 0..OUTPUT_SZ {
                res.connect(
                    Node(i),
                    Node::from_output_index(j),
                    rng.gen_range(INTIAL_WEIGHT_RANGE),
                    innovation_record,
                );
            }
        }

        res
    }

    pub fn nodes(&self) -> usize {
        INPUT_SZ + OUTPUT_SZ + self.hidden_nodes + 1
    }

    pub fn connect(
        &mut self,
        in_node: Node<INPUT_SZ, OUTPUT_SZ>,
        out_node: Node<INPUT_SZ, OUTPUT_SZ>,
        weight: f32,
        innovation_record: &mut InnovationRecord<INPUT_SZ, OUTPUT_SZ>,
    ) {
        self.connections.push(Connection {
            in_node,
            out_node,
            weight,
            enabled: true,
            innovation_number: innovation_record.get(in_node, out_node),
        })
    }

    pub fn crossover(
        fitter_genome: &Genome<INPUT_SZ, OUTPUT_SZ>,
        other_genome: &Genome<INPUT_SZ, OUTPUT_SZ>,
        rng: &mut impl Rng,
    ) -> Genome<INPUT_SZ, OUTPUT_SZ> {
        // TODO: O(n^2)
        let mut connections = Vec::new();

        for fittest_connection in fitter_genome.connections.iter() {
            if let Some(other_connection) = other_genome.connections.iter().find(|connection| {
                connection.innovation_number == fittest_connection.innovation_number
            }) {
                if !rng.gen_bool(CROSSOVER_PICK_FITTEST_CONNECTION_PROB) {
                    connections.push(other_connection.clone());
                    continue;
                }
            }

            connections.push(fittest_connection.clone());
        }

        // Fill in disjoint connections from other_genome.
        // However, I'm not sure if this should even be done so...
        // for connection in other_genome.connections.iter() {
        //     if fitter_genome
        //         .connections
        //         .iter()
        //         .find(|other_connection| {
        //             connection.innovation_number == other_connection.innovation_number
        //         })
        //         .is_none()
        //     {
        //         connections.push(connection.clone());
        //     }
        // }

        // FIXME: Ew
        let mut hidden_nodes = HashSet::new();
        for connection in connections.iter() {
            if connection.in_node.is_hidden() {
                hidden_nodes.insert(connection.in_node);
            }
            if connection.out_node.is_hidden() {
                hidden_nodes.insert(connection.out_node);
            }
        }

        Genome {
            hidden_nodes: hidden_nodes.len(),
            connections,
        }
    }

    fn mutate_weights(&mut self, rng: &mut impl Rng) {
        for mut connection in self.connections.iter_mut() {
            connection.weight = if rng.gen_bool(MUTATE_PERTURB_WEIGHT_RATE) {
                let normal = Normal::new(0.0, MUTATE_WEIGHT_POWER).unwrap();

                connection.weight + normal.sample(rng)
            } else {
                rng.sample::<f32, _>(StandardNormal)
                    .clamp(*MUTATE_REPLACE_RANGE.start(), *MUTATE_REPLACE_RANGE.end())
            }
        }
    }

    fn mutate_new_connection(
        &mut self,
        rng: &mut impl Rng,
        innovation_db: &mut InnovationRecord<INPUT_SZ, OUTPUT_SZ>,
    ) {
        let in_node = Node(rng.gen_range(0..self.nodes()));
        let out_node = Node(rng.gen_range((INPUT_SZ + 1)..self.nodes()));

        // TODO: It would be better to not let this happen in the first place
        // rather than returning if it does happen
        if in_node.is_output() {
            return;
        }

        if in_node == out_node {
            return;
        }

        // Find connection with same connecting nodes and ensure it is enabled if
        // it exists
        // TODO: Use a hashmap to find connection by in, out nodes
        if let Some(mut existing_connection) = self
            .connections
            .iter_mut()
            .find(|connection| connection.in_node == in_node && connection.out_node == out_node)
        {
            existing_connection.enabled = true;
            return;
        }

        // TODO: Disallow cycles

        self.connections.push(Connection {
            in_node,
            out_node,
            weight: rng.gen(),
            enabled: true,
            innovation_number: innovation_db.get(in_node, out_node),
        })
    }

    fn mutate_new_node(
        &mut self,
        rng: &mut impl Rng,
        innovation_db: &mut InnovationRecord<INPUT_SZ, OUTPUT_SZ>,
    ) {
        let new_node = Node(self.nodes());

        if let Some(mut old_connection) = self.connections.choose_mut(rng) {
            old_connection.enabled = false;
            let in_node = old_connection.in_node;
            let out_node = old_connection.out_node;
            let weight = old_connection.weight;

            self.connections.push(Connection {
                in_node,
                out_node: new_node,
                weight: 1.0,
                enabled: true,
                innovation_number: innovation_db.get(in_node, new_node),
            });
            self.connections.push(Connection {
                in_node: new_node,
                out_node,
                weight,
                enabled: true,
                innovation_number: innovation_db.get(new_node, out_node),
            });
            self.hidden_nodes += 1;
        }
    }

    fn mutate_remove_connection(&mut self, rng: &mut impl Rng) {
        if self.connections.is_empty() {
            return;
        }

        self.connections
            .remove(rng.gen_range(0..self.connections.len()));
    }

    pub fn mutate(
        &mut self,
        rng: &mut impl Rng,
        innovation_record: &mut InnovationRecord<INPUT_SZ, OUTPUT_SZ>,
    ) {
        if rng.gen_bool(MUTATE_WEIGHTS_RATE) {
            self.mutate_weights(rng);
        }
        if rng.gen_bool(MUTATE_NEW_CONNECTION_RATE) {
            self.mutate_new_connection(rng, innovation_record);
        }
        if rng.gen_bool(MUTATE_NEW_NODE_RATE) {
            self.mutate_new_node(rng, innovation_record);
        }
        // TODO: Remove nodes, delete connections
    }

    fn activation_function(x: f32) -> f32 {
        1.0 / (1.0 + f32::exp(-x))
    }

    pub fn activate<I>(&self, input: I) -> GenomeActivation<INPUT_SZ, OUTPUT_SZ>
    where
        I: Into<[f32; INPUT_SZ]>,
    {
        let input = input.into();
        let mut activation = GenomeActivation::new(input, self.hidden_nodes);
        let mut other_activation = GenomeActivation::new(input, self.hidden_nodes);
        let mut switch = false;

        // FIXME:
        for _ in 0..20 {
            if switch {
                self.activate_step::<I>(&mut activation, &other_activation);
            } else {
                self.activate_step::<I>(&mut other_activation, &activation);
            }
            switch = !switch;
        }

        if switch {
            other_activation
        } else {
            activation
        }
    }

    pub fn activate_step<I>(
        &self,
        activation: &mut GenomeActivation<INPUT_SZ, OUTPUT_SZ>,
        last_activation: &GenomeActivation<INPUT_SZ, OUTPUT_SZ>,
    ) {
        assert_eq!(activation.hidden.len(), self.hidden_nodes);
        assert_eq!(last_activation.hidden.len(), self.hidden_nodes);

        for i in (INPUT_SZ + 1)..self.nodes() {
            activation[Node(i)] = Self::activation_function(
                self.connections
                    .iter()
                    .filter(|connection| connection.out_node == Node(i) && connection.enabled)
                    .map(|connection| last_activation[connection.in_node] * connection.weight)
                    .sum(),
            );
        }
    }

    pub fn distance(&self, other: &Genome<INPUT_SZ, OUTPUT_SZ>) -> f32 {
        let mut weight_difference_sum: f32 = 0.0;
        let mut matching: usize = 0;
        let mut disjoint: usize = 0;

        let max_innovation_number = if let Some(max_innovation_number) = self
            .connections
            .iter()
            .chain(other.connections.iter())
            .map(|connection| connection.innovation_number)
            .max()
        {
            max_innovation_number
        } else {
            return 0.0;
        };

        for i in 0..=(max_innovation_number) {
            let this_connection = self
                .connections
                .iter()
                .find(|connection| connection.innovation_number == i);
            let other_connection = other
                .connections
                .iter()
                .find(|connection| connection.innovation_number == i);

            match (this_connection, other_connection) {
                (Some(this_conn), Some(other_conn)) => {
                    if this_conn.enabled != other_conn.enabled {
                        weight_difference_sum += 1.0;
                    } else {
                        weight_difference_sum += (this_conn.weight - other_conn.weight).abs();
                    }
                    matching += 1;
                }
                (None, None) => {}
                _ => disjoint += 1,
            }
        }

        if matching == 0 {
            return DIST_DISJOINT_FACTOR * disjoint as f32;
        }

        return DIST_DISJOINT_FACTOR * disjoint as f32
            + DIST_WEIGHT_DIFFERENCE_FACTOR * (weight_difference_sum / matching as f32);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_distance() {
        dbg!(Genome::<2, 1> {
            hidden_nodes: 1,
            connections: vec![
                Connection {
                    in_node: Node(0,),
                    out_node: Node(3,),
                    weight: -0.84480023,
                    enabled: false,
                    innovation_number: 0,
                },
                Connection {
                    in_node: Node(1,),
                    out_node: Node(3,),
                    weight: 3.6848402,
                    enabled: true,
                    innovation_number: 1,
                },
                Connection {
                    in_node: Node(2,),
                    out_node: Node(3,),
                    weight: 4.579851,
                    enabled: true,
                    innovation_number: 2,
                },
                Connection {
                    in_node: Node(0,),
                    out_node: Node(4,),
                    weight: 1.0,
                    enabled: true,
                    innovation_number: 10,
                },
                Connection {
                    in_node: Node(4,),
                    out_node: Node(3,),
                    weight: -0.84480023,
                    enabled: true,
                    innovation_number: 4,
                },
                Connection {
                    in_node: Node(4,),
                    out_node: Node(3,),
                    weight: -0.84480023,
                    enabled: true,
                    innovation_number: 11,
                },
            ],
        }
        .distance(&Genome {
            hidden_nodes: 1,
            connections: vec![
                Connection {
                    in_node: Node(0,),
                    out_node: Node(3,),
                    weight: -0.84480023,
                    enabled: false,
                    innovation_number: 0,
                },
                Connection {
                    in_node: Node(1,),
                    out_node: Node(3,),
                    weight: 3.6848402,
                    enabled: true,
                    innovation_number: 1,
                },
                Connection {
                    in_node: Node(2,),
                    out_node: Node(3,),
                    weight: 4.579851,
                    enabled: true,
                    innovation_number: 2,
                },
                Connection {
                    in_node: Node(0,),
                    out_node: Node(4,),
                    weight: 1.0,
                    enabled: true,
                    innovation_number: 10,
                },
                Connection {
                    in_node: Node(4,),
                    out_node: Node(3,),
                    weight: -0.84480023,
                    enabled: true,
                    innovation_number: 4,
                },
            ],
        }));

        panic!();
    }
}
