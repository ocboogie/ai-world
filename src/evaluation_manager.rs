use crate::{
    environment::Environment, evaluator::Evaluator, genome::GenomeActivation,
    genome_visualizer::GenomeVisualizer, innovation_record::InnovationRecord,
    population::Population,
};
use ::rand::rngs::ThreadRng;
use macroquad::prelude::*;

pub struct EvaluationManager<
    const INPUT_SZ: usize,
    const OUTPUT_SZ: usize,
    E: Environment<INPUT_SZ, OUTPUT_SZ>,
> {
    evaluator: Evaluator<INPUT_SZ, OUTPUT_SZ, E>,
    genome_visualizer: GenomeVisualizer<INPUT_SZ, OUTPUT_SZ>,
    selected_organism: usize,
    selected_species: usize,
    selected_genome_activation: Option<GenomeActivation<INPUT_SZ, OUTPUT_SZ>>,
    input: [f32; INPUT_SZ],
}

impl<const INPUT_SZ: usize, const OUTPUT_SZ: usize, E: Environment<INPUT_SZ, OUTPUT_SZ>>
    EvaluationManager<INPUT_SZ, OUTPUT_SZ, E>
{
    pub fn new(evaluator: Evaluator<INPUT_SZ, OUTPUT_SZ, E>) -> Self {
        Self {
            evaluator,
            genome_visualizer: GenomeVisualizer::new(),
            selected_organism: 0,
            selected_species: 0,
            selected_genome_activation: None,
            input: [0.0; INPUT_SZ],
        }
    }

    pub fn activate_selected(&mut self) {
        let organism = &mut self.evaluator.population.species[self.selected_species].members
            [self.selected_organism];

        let mut new_activation = GenomeActivation::new(self.input, organism.genome.hidden_nodes);

        let last_activation = self
            .selected_genome_activation
            .take()
            .unwrap_or_else(|| GenomeActivation::new(self.input, organism.genome.hidden_nodes));

        organism
            .genome
            .activate_step::<[f32; INPUT_SZ], [f32; OUTPUT_SZ]>(
                &mut new_activation,
                &last_activation,
            );

        self.selected_genome_activation = Some(new_activation);
    }

    pub fn evaluate_and_evolve(&mut self) {
        self.evaluator.evaluate_and_evolve();
    }

    pub fn update(&mut self) {
        let dt = get_frame_time();

        let organism = &self.evaluator.population.species[self.selected_species].members
            [self.selected_organism];

        self.genome_visualizer
            .update(dt, &mut self.evaluator.rng, &organism.genome);
        self.genome_visualizer
            .draw(&organism.genome, self.selected_genome_activation.as_ref());

        set_camera(&Camera2D::from_display_rect(Rect {
            x: -screen_width() / 2.,
            y: -screen_height() / 2.,
            w: screen_width(),
            h: screen_height(),
        }));

        egui_macroquad::ui(|egui_ctx| {
            egui::Window::new("egui ❤ macroquad").show(egui_ctx, |ui| {
                ui.vertical(|ui| {
                    if ui.button("Evolve").clicked() {
                        self.selected_genome_activation = None;
                        self.evaluate_and_evolve();
                    }

                    ui.label(format!(
                        "Generation: {}",
                        self.evaluator.population.generation
                    ));
                    ui.label(format!(
                        "Species: {}",
                        self.evaluator.population.species.len()
                    ));
                    ui.label(format!("Size: {}", self.evaluator.population.size()));

                    ui.add(
                        egui::Slider::new(
                            &mut self.selected_species,
                            0..=self.evaluator.population.species.len() - 1,
                        )
                        .clamp_to_range(true),
                    );

                    if let Some(selected_species) =
                        self.evaluator.population.species.get(self.selected_species)
                    {
                        ui.label(format!("Age: {}", selected_species.age));
                        ui.label(format!(
                            "Average fitness: {}",
                            selected_species.average_fitness
                        ));
                        ui.label(format!(
                            "Champion fitness: {}",
                            selected_species.champion.fitness
                        ));
                        ui.label(format!(
                            "Number of members: {}",
                            selected_species.members.len()
                        ));

                        if ui
                            .add(
                                egui::Slider::new(
                                    &mut self.selected_organism,
                                    0..=selected_species.members.len() - 1,
                                )
                                .clamp_to_range(true),
                            )
                            .changed()
                        {
                            // dbg!(selected_species.members.get(self.selected_organism));
                        }

                        if let Some(selected_organism) =
                            selected_species.members.get(self.selected_organism)
                        {
                            ui.label(format!("Fitness: {}", selected_organism.fitness));
                            ui.label(format!("Nodes: {}", selected_organism.genome.nodes()));
                            ui.label(format!(
                                "Connections: {}",
                                selected_organism.genome.connections.len()
                            ));

                            ui.label("Input values");
                            for i in 0..INPUT_SZ {
                                ui.add(egui::Slider::new(&mut self.input[i], 0.0..=1.0));
                            }

                            if ui.button("Activate").clicked() {
                                self.activate_selected();
                            }
                        }
                    }
                });
            });
        });
    }
}
