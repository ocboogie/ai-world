mod connection;
mod environment;
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
        (4.0 - ((genome.calculate::<[f32; 2], [f32; 1]>([1.0, 0.0])[0] - 1.0).abs()
            + (genome.calculate::<[f32; 2], [f32; 1]>([0.0, 1.0])[0] - 1.0).abs()
            + genome.calculate::<[f32; 2], [f32; 1]>([1.0, 1.0])[0]
            + genome.calculate::<[f32; 2], [f32; 1]>([0.0, 0.0])[0]))
            .powi(2)
    }
}

fn test(rng: &mut impl Rng, innovation_record: &mut InnovationRecord<2, 1>) -> Genome<2, 1> {
    let mut genome = Genome::<2, 1>::default();
    genome.connect(Node(0), Node(3), rng.gen(), innovation_record);
    genome.connect(Node(1), Node(3), rng.gen(), innovation_record);
    genome.connect(Node(2), Node(3), rng.gen(), innovation_record);

    genome
}

#[macroquad::main("egui with macroquad")]
async fn main() {
    let mut genome_visualizer = GenomeVisualizer::default();
    let mut innovation_record = InnovationRecord::default();
    let mut rng = thread_rng();
    let mut population = Population::<2, 1> {
        species: (0..10)
            .into_iter()
            .map(|_| {
                let champ = test(&mut rng, &mut innovation_record);
                Species {
                    representative: champ.clone(),
                    members: (0..100)
                        .into_iter()
                        .map(|_| Organism::new(test(&mut rng, &mut innovation_record)))
                        .collect(),
                    average_fitness: 0.0,
                    champion: champ,
                    age: 1,
                }
            })
            .collect(),
        generation: 0,
        size: 1000,
    };

    loop {
        clear_background(BLACK);

        let dt = get_frame_time();

        if is_key_pressed(KeyCode::Space) {
            println!("Generation: {}", population.species.len());
            for (i, species) in population.species.iter().enumerate() {
                println!("S{}, average: {}", i, species.average_fitness);
            }
            population.evaluate(&mut XOREnv);
            population.evolve(&mut rng, &mut innovation_record);
        }

        let mut a = 0.0;
        let mut b = Genome::default();

        for s in population.species.iter() {
            for o in s.members.iter() {
                if o.fitness > a {
                    a = o.fitness;
                    b = o.genome.clone();
                }
            }
        }

        genome_visualizer.update(dt, &mut rng, &b);
        genome_visualizer.draw(&b);

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
