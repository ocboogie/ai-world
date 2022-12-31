mod connection;
mod genome;
mod genome_visualizer;
mod innovation_record;
mod node;
mod organism;
mod species;

use std::io;

use ::rand::{thread_rng, Rng};
use connection::Connection;
use genome::Genome;
use genome_visualizer::GenomeVisualizer;
use innovation_record::InnovationRecord;
use macroquad::prelude::*;
use node::Node;
use organism::Organism;

// fn cost(genome: &Genome<2, 1>) -> f32 {
//     (genome.calculate::<[f32; 2], [f32; 1]>([1.0, 0.0])[0] - 1.0).abs()
//         + (genome.calculate::<[f32; 2], [f32; 1]>([0.0, 1.0])[0] - 1.0).abs()
// }

fn cost(genome: &Genome<2, 1>) -> f32 {
    (genome.calculate::<[f32; 2], [f32; 1]>([1.0, 0.0])[0] - 1.0).abs()
        + (genome.calculate::<[f32; 2], [f32; 1]>([0.0, 1.0])[0] - 1.0).abs()
        + (genome.calculate::<[f32; 2], [f32; 1]>([0.0, 0.0])[0]).abs()
        + (genome.calculate::<[f32; 2], [f32; 1]>([1.0, 1.0])[0]).abs()
}
fn get_corrected_mouse_pos() -> Vec2 {
    Vec2::from(mouse_position()) - vec2(screen_width() / 2.0, screen_height() / 2.0)
}

#[macroquad::main("Events")]
async fn main() {
    let mut pop = Vec::new();
    let mut generation: usize = 0;
    let mut rng = thread_rng();
    let mut innovation_record = InnovationRecord::default();
    let mut genome_visualizer = GenomeVisualizer::default();

    for _ in 0..100 {
        pop.push(Organism::new(Genome {
            hidden_nodes: 0,
            connections: vec![
                Connection {
                    in_node: Node(1),
                    out_node: Node(3),
                    weight: rng.gen(),
                    enabled: true,
                    innovation_number: 0,
                },
                Connection {
                    in_node: Node(2),
                    out_node: Node(3),
                    weight: rng.gen(),
                    enabled: true,
                    innovation_number: 1,
                },
            ],
        }));
    }

    loop {
        clear_background(BLACK);

        let dt = get_frame_time();

        let genome = &pop.last().unwrap().genome;
        genome_visualizer.update(dt, &mut rng, genome);
        genome_visualizer.draw(genome);

        if is_key_pressed(KeyCode::Space) {
            for org in pop.iter_mut() {
                org.fitness = cost(&org.genome);
            }

            pop.sort_by(|a, b| b.fitness.total_cmp(&a.fitness));

            dbg!(pop.iter().map(|a| a.fitness).collect::<Vec<_>>());

            println!("Generate: {}", generation);
            println!("Best fitness: {}", pop.last().unwrap().fitness);
            println!(
                "Average fitness: {}",
                pop.iter().map(|a| a.fitness).sum::<f32>() / pop.len() as f32
            );

            generation += 1;
            let best = pop.pop().unwrap();
            dbg!(best.genome.calculate::<[f32; 2], [f32; 1]>([0.0, 1.0])[0]);
            dbg!(best.genome.calculate::<[f32; 2], [f32; 1]>([1.0, 0.0])[0]);
            dbg!(best.genome.calculate::<[f32; 2], [f32; 1]>([1.0, 1.0])[0]);
            dbg!(best.genome.calculate::<[f32; 2], [f32; 1]>([0.0, 0.0])[0]);

            for org in &mut pop[0..50] {
                org.genome = Genome::crossover(&best.genome, &org.genome, &mut rng);
            }

            pop.push(best);

            for org in pop.iter_mut() {
                org.genome.mutate(&mut rng, &mut innovation_record);
            }
        }

        set_camera(&Camera2D::from_display_rect(Rect {
            x: -screen_width() / 2.,
            y: -screen_height() / 2.,
            w: screen_width(),
            h: screen_height(),
        }));

        next_frame().await;
    }
}
