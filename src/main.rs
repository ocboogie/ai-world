mod connection;
mod environment;
mod genome;
mod genome_visualizer;
mod innovation_record;
mod node;
mod organism;
mod population;
mod species;

use ::rand::thread_rng;
use environment::Environment;
use genome::Genome;
use genome_visualizer::GenomeVisualizer;
use innovation_record::InnovationRecord;
use macroquad::prelude::*;

struct XOREnv;

impl Environment<2, 1> for XOREnv {
    fn evaluate(&mut self, genome: &Genome<2, 1>) -> f32 {
        (4.0 - ((genome.calculate::<[f32; 2], [f32; 1]>([1.0, 0.0])[0] - 1.0).abs()
            + (genome.calculate::<[f32; 2], [f32; 1]>([0.0, 1.0])[0] - 1.0).abs()
            + genome.calculate::<[f32; 2], [f32; 1]>([1.0, 1.0])[0]
            + genome.calculate::<[f32; 2], [f32; 1]>([0.0, 0.0])[0]))
            .powi(2)
    }
}

#[macroquad::main("egui with macroquad")]
async fn main() {
    // let mut innovation_record = InnovationRecord::default();
    let genome = Genome::<2, 2>::default();
    let mut genome_visualizer = GenomeVisualizer::default();
    let mut rng = thread_rng();

    loop {
        clear_background(BLACK);

        let dt = get_frame_time();

        genome_visualizer.update(dt, &mut rng, &genome);
        genome_visualizer.draw(&genome);

        set_camera(&Camera2D::from_display_rect(Rect {
            x: -screen_width() / 2.,
            y: -screen_height() / 2.,
            w: screen_width(),
            h: screen_height(),
        }));

        egui_macroquad::ui(|egui_ctx| {
            egui::Window::new("egui ❤ macroquad").show(egui_ctx, |ui| {
                ui.label("Test");
            });
        });

        egui_macroquad::draw();

        next_frame().await;
    }
}
