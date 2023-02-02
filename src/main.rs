mod app;
mod client;
mod connection;
mod environment;
mod evaluation;
mod evaluation_manager;
mod evaluator;
mod force_directed_graph;
mod genome;
mod genome_visualizer;
mod innovation_record;
mod node;
mod population;
mod population_manager;
mod speciation;
mod species;

use eframe::egui;
use environment::Environment;
use evaluation_manager::EvaluationManager;
use evaluator::Evaluator;
use genome::Genome;

const DATA: [([f32; 2], [f32; 1]); 4] = [
    ([0.0, 0.0], [0.0]),
    ([1.0, 0.0], [1.0]),
    ([0.0, 1.0], [1.0]),
    ([1.0, 1.0], [0.0]),
];

struct XOREnv;

impl Environment<2, 1> for XOREnv {
    fn evaluate(&mut self, genome: &Genome<2, 1>) -> f32 {
        let mut fitness = 4.0;

        for (input, output) in DATA {
            let diff = genome.activate::<[f32; 2], [f32; 1]>(input)[0] - output[0];
            fitness -= diff * diff;
        }

        fitness
    }
}

fn main() {
    tracing_subscriber::fmt::init();

    let options = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(1280.0, 720.0)),
        ..Default::default()
    };
    eframe::run_native(
        "Boogie NEAT",
        options,
        Box::new(|_cc| Box::new(MyApp::new())),
    );
}

struct MyApp {
    evaluation_manager: EvaluationManager<2, 1, XOREnv>,
}

impl MyApp {
    fn new() -> Self {
        Self {
            evaluation_manager: EvaluationManager::new(Evaluator::new(XOREnv, 150)),
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.evaluation_manager.show(ctx);
    }
}
