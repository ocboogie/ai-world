use crate::client::ClientId;
use crate::force_directed_graph::{FDGraph, Graph, NodeEntity};
use crate::genome::{Genome, GenomeActivation};
use crate::node::Node;
use eframe::egui::style::Margin;
use eframe::egui::Frame;
use eframe::egui::{CentralPanel, TopBottomPanel};
use eframe::{egui, epaint::pos2};
use rand::{thread_rng, Rng};

const INPUT_OUTPUT_DIST: f32 = 35.0;
const ADJACENT_NODE_DIST: f32 = 10.0;

pub struct GenomeVisualizer<const INPUT_SZ: usize, const OUTPUT_SZ: usize> {
    client_id: ClientId,
    test_inputs: [f32; INPUT_SZ],
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
            fixed: true,
            ..Default::default()
        })
    }

    pub fn new(genome: Genome<INPUT_SZ, OUTPUT_SZ>, client_id: ClientId) -> Self {
        Self {
            client_id,
            test_inputs: [0.0; INPUT_SZ],
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

    fn node_text(&self, node: Node<INPUT_SZ, OUTPUT_SZ>) -> Option<String> {
        Some(format!("{:.2}", self.activation.as_ref()?[node]))
    }

    fn size(&self) -> usize {
        self.nodes()
    }
}

impl<const INPUT_SZ: usize, const OUTPUT_SZ: usize> egui::Widget
    for &mut GenomeVisualizer<INPUT_SZ, OUTPUT_SZ>
{
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        let mut updated = false;

        TopBottomPanel::bottom(format!("input_panel_{}", self.client_id)).show_inside(ui, |ui| {
            ui.horizontal(|ui| {
                for input in self.test_inputs.iter_mut() {
                    if ui
                        .add(
                            egui::DragValue::new(input)
                                .clamp_range(0.0..=1.0)
                                .speed(0.01),
                        )
                        .changed()
                    {
                        updated = true;
                    }
                }
            })
            .response
        });

        if updated {
            self.genome
                .activate::<[f32; INPUT_SZ], [f32; OUTPUT_SZ]>(self.test_inputs.clone());
        }

        CentralPanel::default()
            .show_inside(ui, |ui| self.fd_graph.show(ui, &self.genome, |_| {}))
            .response
    }
}
