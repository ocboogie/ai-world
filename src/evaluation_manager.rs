use crate::{
    client::ClientId, environment::Environment, evaluation::Evaluation, evaluator::Evaluator,
    genome::GenomeActivation, population::Population, population_manager::PopulationManager,
    speciation::Speciation,
};

use eframe::{
    egui::{
        self,
        plot::{Corner, Legend},
    },
    epaint::Vec2,
};

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

        egui::Window::new("My Window").show(ctx, |ui| {
            use egui::plot::{Line, Plot, PlotPoints};
            let max_fitness: PlotPoints = self
                .history
                .iter()
                .enumerate()
                .map(|(i, gen)| [i as f64, gen.evaluation.champion().1 as f64])
                .collect();
            let max_fitness_line = Line::new(max_fitness).name("Champion Fitness");
            let avg_fitness: PlotPoints = self
                .history
                .iter()
                .enumerate()
                .map(|(i, gen)| [i as f64, gen.evaluation.average_fitness() as f64])
                .collect();
            let avg_fitness_line = Line::new(avg_fitness).name("Average Fitness");

            Plot::new("my_plot")
                .clamp_grid(true)
                .allow_drag(false)
                .allow_zoom(false)
                .allow_scroll(false)
                .set_margin_fraction(Vec2::splat(0.1))
                .label_formatter(|name, value| {
                    if !name.is_empty() {
                        format!("{:.2}", value.y).to_owned()
                    } else {
                        "".to_owned()
                    }
                })
                .legend(Legend::default().position(Corner::RightBottom))
                .view_aspect(2.0)
                .show(ui, |plot_ui| {
                    plot_ui.line(max_fitness_line);
                    plot_ui.line(avg_fitness_line);
                });
        });

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
