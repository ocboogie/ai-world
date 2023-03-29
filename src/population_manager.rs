use std::collections::HashMap;

use crate::{
    client::ClientId,
    evaluation::Evaluation,
    force_directed_graph::{FDGraph, Graph},
    genome::Genome,
    genome_visualizer::GenomeVisualizer,
    node::Node,
    population::Population,
    speciation::Speciation,
};
use eframe::{
    egui,
    epaint::{vec2, Vec2},
    Frame,
};
use egui_extras::{Column, TableBuilder};

const GENOME_WINDOW_SIZE: f32 = 200.0;

#[derive(Default)]
pub struct PopulationManager<const INPUT_SZ: usize, const OUTPUT_SZ: usize> {
    fd_graph: FDGraph<INPUT_SZ, OUTPUT_SZ>,
    genomes_open: Vec<GenomeVisualizer<INPUT_SZ, OUTPUT_SZ>>,
}

impl<const INPUT_SZ: usize, const OUTPUT_SZ: usize> PopulationManager<INPUT_SZ, OUTPUT_SZ> {
    pub fn show(
        &mut self,
        ctx: &egui::Context,
        population: &mut Population<INPUT_SZ, OUTPUT_SZ>,
        speciation: &Speciation<INPUT_SZ, OUTPUT_SZ>,
        evaluation: &Evaluation,
    ) {
        egui::SidePanel::left("Population controls").show(ctx, |ui| {
            ui.label(format!("Generation: {}", population.generation));
            ui.label(format!("Target size: {}", population.target_size));
            ui.label(format!("Size: {}", population.members.len()));
            ui.label(format!("Species: {}", speciation.species.len()));
            if let (Some(vis_1), Some(vis_2)) = (self.genomes_open.get(0), self.genomes_open.get(1))
            {
                ui.label(format!(
                    "Distance: {}",
                    vis_1.genome.distance(&vis_2.genome)
                ));
            }

            ui.separator();

            let interact_height = ui.spacing().interact_size.y;

            TableBuilder::new(ui)
                .striped(true)
                .column(Column::auto())
                .column(Column::initial(30.0))
                .column(Column::initial(60.0))
                .column(Column::initial(60.0))
                .header(interact_height, |mut header| {
                    header.col(|_| {});
                    header.col(|ui| {
                        ui.label("Size");
                    });
                    header.col(|ui| {
                        ui.label("Average fitness");
                    });
                    header.col(|ui| {
                        ui.label("Champion fitness");
                    });
                })
                .body(|mut body| {
                    for species in speciation.species.values() {
                        body.row(interact_height, |mut row| {
                            row.col(|ui| {
                                if ui.button(&species.id.to_string()[..6]).clicked() {
                                    let champion = evaluation.species_champion(species).0;
                                    let genome = population.members[champion].clone();

                                    self.genomes_open
                                        .push(GenomeVisualizer::new(genome, champion));
                                }
                            });
                            row.col(|ui| {
                                ui.label(species.members.len().to_string());
                            });
                            row.col(|ui| {
                                ui.label(format!(
                                    "{:.2}",
                                    evaluation.species_average_fitness(species)
                                ));
                            });
                            row.col(|ui| {
                                ui.label(format!("{:.2}", evaluation.species_champion(species).1));
                            });
                        });
                    }
                });
            // egui::ScrollArea::vertical()
            //     .auto_shrink([false; 2])
            //     .max_width(f32::INFINITY)
            //     .show(ui, |ui| {
            //         egui::Grid::new("some_unique_id")
            //             .num_columns(2)
            //             .striped(true)
            //             .show(ui, |ui| {
            //                 ui.label("Size");
            //                 ui.label("Average fitness");
            //                 ui.end_row();
            //
            //                 for species in speciation.species.values() {
            //                     ui.label(&species.members.len().to_string());
            //                     ui.label(&evaluation.species_average_fitness(species).to_string());
            //                     ui.end_row();
            //                 }
            //             })
            //     });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            let pop_graph = PopulationGraph { speciation };

            self.fd_graph.show(ui, &pop_graph, |client_id| {
                let genome = population.members[client_id].clone();

                self.genomes_open
                    .push(GenomeVisualizer::new(genome, client_id));
            });
        });

        // FIXME
        let mut hack = 0;

        self.genomes_open
            .retain_mut(|visualizer: &mut GenomeVisualizer<INPUT_SZ, OUTPUT_SZ>| {
                let mut open = true;

                hack += 1;

                egui::Window::new(hack.to_string())
                    .default_size(Vec2::splat(GENOME_WINDOW_SIZE))
                    .open(&mut open)
                    .show(ctx, |ui| {
                        ui.add(visualizer);
                    });

                open
            });
    }
}

struct PopulationGraph<'a, const INPUT_SZ: usize, const OUTPUT_SZ: usize> {
    speciation: &'a Speciation<INPUT_SZ, OUTPUT_SZ>,
}

impl<'a, const INPUT_SZ: usize, const OUTPUT_SZ: usize> Graph<INPUT_SZ, OUTPUT_SZ>
    for PopulationGraph<'a, INPUT_SZ, OUTPUT_SZ>
{
    fn connected(
        &self,
        node_1: Node<INPUT_SZ, OUTPUT_SZ>,
        node_2: Node<INPUT_SZ, OUTPUT_SZ>,
    ) -> bool {
        self.speciation.member_map[&node_1.0] == self.speciation.member_map[&node_2.0]
    }

    fn size(&self) -> usize {
        self.speciation.member_map.len()
    }
}
