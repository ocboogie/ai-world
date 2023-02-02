use crate::force_directed_graph::{FDGraph, Graph, NodeEntity};
use crate::genome::Genome;
use crate::node::Node;
use eframe::{
    egui,
    epaint::{pos2, Vec2},
};

const INPUT_OUTPUT_DIST: f32 = 120.0;
const ADJACENT_NODE_DIST: f32 = 50.0;

pub struct GenomeVisualizer<const INPUT_SZ: usize, const OUTPUT_SZ: usize> {
    pub genome: Genome<INPUT_SZ, OUTPUT_SZ>,
    fd_graph: FDGraph<INPUT_SZ, OUTPUT_SZ>,
}

impl<const INPUT_SZ: usize, const OUTPUT_SZ: usize> GenomeVisualizer<INPUT_SZ, OUTPUT_SZ> {
    fn spawner(node: Node<INPUT_SZ, OUTPUT_SZ>) -> Option<NodeEntity> {
        if node.is_hidden() {
            return None;
        }
        let input_width = (INPUT_SZ - 1) as f32 * ADJACENT_NODE_DIST;
        let output_width = (OUTPUT_SZ - 1) as f32 * ADJACENT_NODE_DIST;

        let pos = if node.is_bias() {
            pos2(
                -input_width / 2.0 - ADJACENT_NODE_DIST * 1.25,
                INPUT_OUTPUT_DIST / 2.0,
            )
        } else if node.is_input() {
            pos2(
                (input_width / INPUT_SZ as f32) * 2.0 * (node.0 - 1) as f32 - input_width / 2.0,
                INPUT_OUTPUT_DIST / 2.0,
            )
        } else {
            pos2(
                (output_width / OUTPUT_SZ as f32) * 2.0 * (node.0 - INPUT_SZ - 1) as f32
                    - output_width / 2.0,
                -INPUT_OUTPUT_DIST / 2.0,
            )
        };

        Some(NodeEntity {
            pos,
            vel: Vec2::ZERO,
            fixed: true,
        })
    }

    pub fn new(genome: Genome<INPUT_SZ, OUTPUT_SZ>) -> Self {
        Self {
            genome,
            fd_graph: FDGraph::with_spawner(Box::new(Self::spawner), true),
        }
    }
}

impl<const INPUT_SZ: usize, const OUTPUT_SZ: usize> Graph<INPUT_SZ, OUTPUT_SZ>
    for Genome<INPUT_SZ, OUTPUT_SZ>
{
    fn connected(
        &self,
        node_1: Node<INPUT_SZ, OUTPUT_SZ>,
        node_2: Node<INPUT_SZ, OUTPUT_SZ>,
    ) -> bool {
        self.connections.iter().any(|connection| {
            connection.enabled
                && ((connection.in_node == node_1 && connection.out_node == node_2)
                    || (connection.in_node == node_2 && connection.out_node == node_1))
        })
    }

    fn connection_text(
        &self,
        node_1: Node<INPUT_SZ, OUTPUT_SZ>,
        node_2: Node<INPUT_SZ, OUTPUT_SZ>,
    ) -> Option<String> {
        self.connections
            .iter()
            .find(|connection| {
                connection.enabled
                    && ((connection.in_node == node_1 && connection.out_node == node_2)
                        || (connection.in_node == node_2 && connection.out_node == node_1))
            })
            .map(|connection| format!("{:.2}", connection.weight))
    }

    fn size(&self) -> usize {
        self.nodes()
    }
}

impl<const INPUT_SZ: usize, const OUTPUT_SZ: usize> egui::Widget
    for &mut GenomeVisualizer<INPUT_SZ, OUTPUT_SZ>
{
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        self.fd_graph.show(ui, &self.genome, |_| {})
    }
}
