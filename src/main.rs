mod connection;
mod environment;
mod evaluation_manager;
mod evaluator;
mod genome;
mod genome_visualizer;
mod innovation_record;
mod node;
mod organism;
mod population;
mod species;

use ::rand::{thread_rng, Rng};
use connection::Connection;
use environment::Environment;
use evaluation_manager::EvaluationManager;
use evaluator::Evaluator;
use genome::Genome;
use genome_visualizer::GenomeVisualizer;
use innovation_record::InnovationRecord;
use macroquad::prelude::*;
use node::Node;
use organism::Organism;
use population::Population;
use species::Species;

struct XOREnv;

impl Environment<2, 1> for XOREnv {
    fn evaluate(&mut self, genome: &Genome<2, 1>) -> f32 {
        (4.0 - ((genome.activate::<[f32; 2], [f32; 1]>([1.0, 0.0])[0] - 1.0).abs()
            + (genome.activate::<[f32; 2], [f32; 1]>([0.0, 1.0])[0] - 1.0).abs()
            + genome.activate::<[f32; 2], [f32; 1]>([1.0, 1.0])[0]
            + genome.activate::<[f32; 2], [f32; 1]>([0.0, 0.0])[0]))
            .powi(2)
    }
}

#[macroquad::main("egui with macroquad")]
async fn main() {
    let evaluator = Evaluator::new(XOREnv, 150);
    let mut evaluation_manager = EvaluationManager::new(evaluator);

    loop {
        clear_background(BLACK);

        set_camera(&Camera2D::from_display_rect(Rect {
            x: -screen_width() / 2.,
            y: -screen_height() / 2.,
            w: screen_width(),
            h: screen_height(),
        }));

        evaluation_manager.update();

        egui_macroquad::draw();

        next_frame().await;
    }
}
