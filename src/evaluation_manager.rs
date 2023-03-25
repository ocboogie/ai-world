use crate::{
    client::ClientId, environment::Environment, evaluation::Evaluation, evaluator::Evaluator,
    genome::GenomeActivation, population::Population, population_manager::PopulationManager,
    speciation::Speciation,
};

use eframe::egui;

struct Generation<const INPUT_SZ: usize, const OUTPUT_SZ: usize> {
    population: Population<INPUT_SZ, OUTPUT_SZ>,
    speciation: Speciation<INPUT_SZ, OUTPUT_SZ>,
    evaluation: Evaluation,
}

pub struct EvaluationManager<
    const INPUT_SZ: usize,
    const OUTPUT_SZ: usize,
    E: Environment<INPUT_SZ, OUTPUT_SZ>,
> {
    evaluator: Evaluator<INPUT_SZ, OUTPUT_SZ, E>,
    population_manager: PopulationManager<INPUT_SZ, OUTPUT_SZ>,
    history: Vec<Generation<INPUT_SZ, OUTPUT_SZ>>,
    selected_generation: usize,
}

impl<const INPUT_SZ: usize, const OUTPUT_SZ: usize, E: Environment<INPUT_SZ, OUTPUT_SZ>>
    EvaluationManager<INPUT_SZ, OUTPUT_SZ, E>
{
    pub fn new(evaluator: Evaluator<INPUT_SZ, OUTPUT_SZ, E>) -> Self {
        Self {
            evaluator,
            population_manager: PopulationManager::default(),
            history: Vec::new(),
            selected_generation: 0,
        }
    }

    pub fn evaluate_and_evolve(&mut self) {
        self.evaluator.evaluate_and_evolve();
        self.history.push(Generation {
            population: self.evaluator.population.clone(),
            speciation: self.evaluator.last_speciation.clone().unwrap(),
            evaluation: self.evaluator.last_evaluation.clone().unwrap(),
        });
    }

    pub fn show(&mut self, ctx: &egui::Context) {
        if self.history.is_empty() {
            egui::CentralPanel::default().show(ctx, |ui| {
                ui.centered_and_justified(|ui| {
                    if ui.button("Start").clicked() {
                        self.evaluate_and_evolve()
                    }
                });
            });
            return;
        }

        egui::TopBottomPanel::top("top").show(ctx, |ui| {
            ui.horizontal_centered(|ui| {
                if ui.button("<").clicked() {
                    self.selected_generation = self.selected_generation.saturating_sub(1);
                }
                ui.label(format!(
                    "{}/{}",
                    self.selected_generation + 1,
                    self.history.len()
                ));
                if ui.button(">").clicked() {
                    self.selected_generation += 1;
                    while self.selected_generation >= self.history.len() {
                        self.evaluate_and_evolve();
                    }
                }
            });
        });

        self.selected_generation = self
            .selected_generation
            .clamp(0, self.history.len().saturating_sub(1));

        if let Some(Generation {
            population,
            speciation,
            evaluation,
        }) = self.history.get_mut(self.selected_generation)
        {
            self.population_manager
                .show(ctx, population, speciation, evaluation);
        }
    }
}
