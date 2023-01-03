use crate::{
    connection::{self, Connection},
    innovation_record::InnovationRecord,
    node::Node,
};
use rand::seq::SliceRandom;
use rand::Rng;
use std::{
    collections::HashSet,
    ops::{Index, IndexMut},
};

const BIAS: f32 = 1.0;

const MUTATE_WEIGHTS_RATE: f32 = 0.80;
const MUTATE_PERTURB_WEIGHT_RATE: f32 = 0.90;
const MUTATE_NEW_NODE_RATE: f32 = 0.03;
const MUTATE_NEW_CONNECTION_RATE: f32 = 0.05;

const CROSSOVER_PICK_FITTEST_CONNECTION_PROB: f64 = 0.5;

const DIST_DISJOINT_FACTOR: f32 = 1.0;
const DIST_WEIGHT_DIFFERENCE_FACTOR: f32 = 0.4;

#[derive(Default, Debug, Clone)]
pub struct Genome<const INPUT_SZ: usize, const OUTPUT_SZ: usize> {
    pub hidden_nodes: usize,
    // TODO: Make innovation number the index into a hash set instead of using
    // an vector
    pub connections: Vec<Connection<INPUT_SZ, OUTPUT_SZ>>,
}

#[derive(Debug)]
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
                    rng.gen(),
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
            if rng.gen::<f32>() < MUTATE_PERTURB_WEIGHT_RATE {
                connection.weight += rng.gen::<f32>() - 0.5;
            } else {
                connection.weight = rng.gen();
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

    pub fn mutate(
        &mut self,
        rng: &mut impl Rng,
        innovation_record: &mut InnovationRecord<INPUT_SZ, OUTPUT_SZ>,
    ) {
        if rng.gen::<f32>() < MUTATE_WEIGHTS_RATE {
            self.mutate_weights(rng);
        }
        if rng.gen::<f32>() < MUTATE_NEW_CONNECTION_RATE {
            self.mutate_new_connection(rng, innovation_record);
        }
        if rng.gen::<f32>() < MUTATE_NEW_NODE_RATE {
            self.mutate_new_node(rng, innovation_record);
        }
    }

    fn activation_function(x: f32) -> f32 {
        use std::f32::consts::E;
        1.0 / (1.0 + E.powf(-x))
    }

    pub fn activate<I, O>(&self, input: I) -> O
    where
        I: Into<[f32; INPUT_SZ]>,
        O: From<[f32; OUTPUT_SZ]>,
    {
        let input = input.into();
        let mut activation = GenomeActivation::new(input, self.hidden_nodes);
        let mut other_activation = GenomeActivation::new(input, self.hidden_nodes);
        let mut switch = false;

        // FIXME:
        for _ in 0..10 {
            if switch {
                self.activate_step::<I, O>(&mut activation, &other_activation);
            } else {
                self.activate_step::<I, O>(&mut other_activation, &activation);
            }
            switch = !switch;
        }

        if switch {
            other_activation.output.into()
        } else {
            activation.output.into()
        }
    }

    pub fn activate_step<I, O>(
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

        for i in 0..max_innovation_number {
            let this_connection = self
                .connections
                .iter()
                .find(|connection| connection.innovation_number == i);
            let other_connection = other
                .connections
                .iter()
                .find(|connection| connection.innovation_number == i);

            match (this_connection, other_connection) {
                (Some(a), Some(b)) => {
                    weight_difference_sum += (a.weight - b.weight).abs();
                    matching += 1;
                }
                _ => disjoint += 1,
            }
        }

        return DIST_DISJOINT_FACTOR * disjoint as f32
            + DIST_WEIGHT_DIFFERENCE_FACTOR * (weight_difference_sum / matching as f32);
    }
}
