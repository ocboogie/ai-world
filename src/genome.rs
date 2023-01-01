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

const DIST_EXCESS_FACTOR: f32 = 1.0;
const DIST_DISJOINT_FACTOR: f32 = 1.0;
const DIST_MATCHING_WEIGHT_FACTOR: f32 = 0.4;

#[derive(Default, Debug, Clone)]
pub struct Genome<const INPUT_SZ: usize, const OUTPUT_SZ: usize> {
    pub hidden_nodes: usize,
    // TODO: Make innovation number the index into a hash set instead of using
    // an vector
    pub connections: Vec<Connection<INPUT_SZ, OUTPUT_SZ>>,
}

#[derive(Debug)]
struct GenomeRuntime<const INPUT_SZ: usize, const OUTPUT_SZ: usize> {
    pub input: [f32; INPUT_SZ],
    pub output: [f32; OUTPUT_SZ],
    pub hidden: Vec<f32>,
}

impl<const INPUT_SZ: usize, const OUTPUT_SZ: usize> GenomeRuntime<INPUT_SZ, OUTPUT_SZ> {
    pub fn new(input: [f32; INPUT_SZ], hidden_nodes: usize) -> Self {
        Self {
            input,
            output: [f32::NAN; OUTPUT_SZ],
            hidden: vec![f32::NAN; hidden_nodes],
        }
    }
}

/// 0 = Bias
/// 1:INPUT_SZ + 1 = Input nodes
/// INPUT_SZ + 1:OUTPUT_SZ + INPUT_SZ + 1 = Output nodes
/// OUTPUT_SZ + INPUT_SZ + 1: = Hidden nodes
impl<const INPUT_SZ: usize, const OUTPUT_SZ: usize> Index<Node<INPUT_SZ, OUTPUT_SZ>>
    for GenomeRuntime<INPUT_SZ, OUTPUT_SZ>
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
    for GenomeRuntime<INPUT_SZ, OUTPUT_SZ>
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

    pub fn nodes(&self) -> usize {
        INPUT_SZ + OUTPUT_SZ + self.hidden_nodes + 1
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

    fn compute_node(
        &self,
        target: Node<INPUT_SZ, OUTPUT_SZ>,
        runtime: &mut GenomeRuntime<INPUT_SZ, OUTPUT_SZ>,
    ) -> f32 {
        if !runtime[target].is_nan() {
            return runtime[target];
        }

        let val = self
            .connections
            .iter()
            .filter(|connection| connection.out_node == target && connection.enabled)
            .map(|connection| self.compute_node(connection.in_node, runtime) * connection.weight)
            .sum();

        runtime[target] = Self::activation_function(val);

        val
    }

    pub fn calculate<I, O>(&self, input: I) -> O
    where
        I: Into<[f32; INPUT_SZ]>,
        O: From<[f32; OUTPUT_SZ]>,
    {
        let mut runtime = GenomeRuntime::new(input.into(), self.hidden_nodes);

        for i in 0..OUTPUT_SZ {
            self.compute_node(Node(i + INPUT_SZ + 1), &mut runtime);
        }

        runtime.output.into()
    }

    pub fn distance(&self, other: &Genome<INPUT_SZ, OUTPUT_SZ>) -> f32 {
        let mut weight_difference_sum: f32 = 0.0;
        let mut matching: usize = 0;
        let mut disjoint: usize = 0;
        let mut excess: usize = 0;

        let self_max_innovation_number = self
            .connections
            .iter()
            .map(|connection| connection.innovation_number)
            .max()
            .unwrap_or(0);
        let other_max_innovation_number = other
            .connections
            .iter()
            .map(|connection| connection.innovation_number)
            .max()
            .unwrap_or(0);

        let max_innovation_number = self_max_innovation_number.max(other_max_innovation_number);

        if max_innovation_number == 0 {
            return 0.0;
        }

        for i in 0..max_innovation_number {
            let this_connection = self
                .connections
                .iter()
                .find(|connection| connection.innovation_number == i);
            let other_connection = self
                .connections
                .iter()
                .find(|connection| connection.innovation_number == i);

            match (this_connection, other_connection) {
                (Some(a), Some(b)) => {
                    weight_difference_sum += (a.weight - b.weight).abs();
                    matching += 1;
                }
                (Some(_), None) if i < other_max_innovation_number => disjoint += 1,
                (None, Some(_)) if i < other_max_innovation_number => disjoint += 1,
                _ => excess += 1,
            }
        }

        return DIST_EXCESS_FACTOR * excess as f32
            + DIST_DISJOINT_FACTOR * disjoint as f32
            + DIST_MATCHING_WEIGHT_FACTOR * (weight_difference_sum / matching as f32);
    }
}

#[cfg(test)]
mod tests {
    use rand::thread_rng;

    use super::*;

    #[test]
    fn test_basic() {
        assert_eq!(
            Genome::<1, 1> {
                hidden_nodes: 1,
                connections: vec![
                    Connection {
                        in_node: Node(1),
                        out_node: Node(3),
                        weight: 0.5,
                        enabled: true,
                        innovation_number: 0,
                    },
                    Connection {
                        in_node: Node(3),
                        out_node: Node(2),
                        weight: 0.5,
                        enabled: true,
                        innovation_number: 1,
                    },
                ],
            }
            .calculate::<[f32; 1], [f32; 1]>([12.0]),
            [3.0]
        );

        assert_eq!(
            Genome::<1, 1> {
                hidden_nodes: 0,
                connections: vec![Connection {
                    in_node: Node(1),
                    out_node: Node(2),
                    weight: 0.5,
                    enabled: false,
                    innovation_number: 0,
                }]
            }
            .calculate::<[f32; 1], [f32; 1]>([12.0]),
            [0.0]
        );
    }

    #[test]
    fn test_crossover() {
        let fittest = Genome::<3, 1> {
            hidden_nodes: 2,
            connections: vec![
                Connection {
                    in_node: Node(1),
                    out_node: Node(4),
                    weight: 0.5,
                    enabled: true,
                    innovation_number: 1,
                },
                Connection {
                    in_node: Node(2),
                    out_node: Node(4),
                    weight: 0.5,
                    enabled: false,
                    innovation_number: 2,
                },
                Connection {
                    in_node: Node(3),
                    out_node: Node(4),
                    weight: 0.5,
                    enabled: true,
                    innovation_number: 3,
                },
                Connection {
                    in_node: Node(2),
                    out_node: Node(5),
                    weight: 0.5,
                    enabled: true,
                    innovation_number: 4,
                },
                Connection {
                    in_node: Node(5),
                    out_node: Node(4),
                    weight: 0.5,
                    enabled: true,
                    innovation_number: 5,
                },
                Connection {
                    in_node: Node(1),
                    out_node: Node(5),
                    weight: 0.5,
                    enabled: true,
                    innovation_number: 8,
                },
            ],
        };

        let other = Genome::<3, 1> {
            hidden_nodes: 2,
            connections: vec![
                Connection {
                    in_node: Node(1),
                    out_node: Node(4),
                    weight: 0.5,
                    enabled: true,
                    innovation_number: 1,
                },
                Connection {
                    in_node: Node(2),
                    out_node: Node(4),
                    weight: 0.5,
                    enabled: false,
                    innovation_number: 2,
                },
                Connection {
                    in_node: Node(3),
                    out_node: Node(4),
                    weight: 0.5,
                    enabled: true,
                    innovation_number: 3,
                },
                Connection {
                    in_node: Node(2),
                    out_node: Node(5),
                    weight: 0.5,
                    enabled: true,
                    innovation_number: 4,
                },
                Connection {
                    in_node: Node(5),
                    out_node: Node(4),
                    weight: 0.5,
                    enabled: false,
                    innovation_number: 5,
                },
                Connection {
                    in_node: Node(5),
                    out_node: Node(6),
                    weight: 0.5,
                    enabled: true,
                    innovation_number: 6,
                },
                Connection {
                    in_node: Node(6),
                    out_node: Node(4),
                    weight: 0.5,
                    enabled: true,
                    innovation_number: 7,
                },
                Connection {
                    in_node: Node(3),
                    out_node: Node(5),
                    weight: 0.5,
                    enabled: true,
                    innovation_number: 9,
                },
                Connection {
                    in_node: Node(1),
                    out_node: Node(6),
                    weight: 0.5,
                    enabled: true,
                    innovation_number: 10,
                },
            ],
        };

        // The 'ol looks good to me unit test
        dbg!(Genome::crossover(&fittest, &other, &mut thread_rng()));
        // panic!();
    }
}
