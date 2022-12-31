mod connection;
mod genome;
mod innovation_record;
mod node;
mod species;

use ::rand::Rng;
use genome::Genome;
use innovation_record::InnovationRecord;
use macroquad::prelude::*;

const NODE_COLOR: Color = GREEN;
const NODE_SIZE: f32 = 10.0;
const NODE_DIST: f32 = 50.0;
const CONNECTION_THICKNESS: f32 = 5.0;
const CONNECTION_COLOR: Color = RED;
const SPAWN_SIZE: f32 = 100.0;
const ATTRACTION_FORCE: f32 = 50.0;
const REJECTION_FORCE: f32 = 1000000.0;
const DRAG_FORCE: f32 = 0.8;

#[derive(Debug)]
struct NeuronVisualizer {
    pos: Vec2,
    vel: Vec2,
    fixed: bool,
}

impl NeuronVisualizer {
    fn update(&mut self, dt: f32) {
        if !self.fixed {
            self.pos += self.vel * dt;
            self.vel *= DRAG_FORCE;
        }
    }

    fn draw(&self) {
        draw_circle(self.pos.x, self.pos.y, NODE_SIZE, NODE_COLOR);
    }
}

#[derive(Default, Debug)]
pub struct GenomeVisualizer<const INPUT_SZ: usize, const OUTPUT_SZ: usize> {
    nodes: Vec<NeuronVisualizer>,
}

impl<const INPUT_SZ: usize, const OUTPUT_SZ: usize> GenomeVisualizer<INPUT_SZ, OUTPUT_SZ> {
    fn create_random(&mut self, rng: &mut impl Rng) {
        self.nodes.push(NeuronVisualizer {
            pos: vec2(
                (rng.gen::<f32>()) * SPAWN_SIZE,
                (rng.gen::<f32>()) * SPAWN_SIZE,
            ),
            vel: vec2(0.0, 0.0),
            fixed: false,
        });
    }

    fn update(&mut self, dt: f32, rng: &mut impl Rng, genome: &Genome<INPUT_SZ, OUTPUT_SZ>) {
        for _ in self.nodes.len()..genome.nodes() {
            self.create_random(rng);
        }

        for connection in genome.connections.iter() {
            let in_node = self.nodes[*connection.in_node].pos;
            let out_node = &mut self.nodes[*connection.out_node];

            let dist = in_node.distance(out_node.pos);
            let ideal_dist = dist - NODE_DIST;

            let force = (in_node - out_node.pos).normalize() * ATTRACTION_FORCE * ideal_dist * dt;
            out_node.vel += force;

            let in_node = &mut self.nodes[*connection.in_node];
            in_node.vel -= force;
        }

        for i in 0..genome.nodes() {
            for j in 0..genome.nodes() {
                if i == j {
                    continue;
                }

                let this_pos = self.nodes[i].pos;
                let other = &self.nodes[j];

                let dist = other.pos.distance(this_pos);
                if dist == 0.0 {
                    continue;
                }
                let force =
                    (other.pos - this_pos).normalize() * (REJECTION_FORCE / (dist * dist)) * dt;

                let this = &mut self.nodes[i];
                this.vel -= force;
            }
        }

        for node in self.nodes.iter_mut() {
            node.update(dt);
        }
    }

    fn draw(&self, genome: &Genome<INPUT_SZ, OUTPUT_SZ>) {
        for node in genome.connections.iter() {
            let in_pos = self.nodes[*node.in_node].pos;
            let out_pos = self.nodes[*node.out_node].pos;

            draw_line(
                in_pos.x,
                in_pos.y,
                out_pos.x,
                out_pos.y,
                CONNECTION_THICKNESS,
                CONNECTION_COLOR,
            );
        }

        for node in self.nodes.iter().take(genome.nodes()) {
            node.draw();
        }
    }
}

fn get_corrected_mouse_pos() -> Vec2 {
    Vec2::from(mouse_position()) - vec2(screen_width() / 2.0, screen_height() / 2.0)
}

#[macroquad::main("Events")]
async fn main() {
    let mut genome = Genome::<1, 1> {
        hidden_nodes: 0,
        connections: vec![],
    };

    let mut genome_visualizer = GenomeVisualizer::default();
    let mut rng = ::rand::thread_rng();
    let mut selected_node = 0;
    let mut innovation_record = InnovationRecord::default();

    loop {
        clear_background(BLACK);

        let dt = get_frame_time();

        let mouse_pos = get_corrected_mouse_pos();
        if is_mouse_button_pressed(MouseButton::Left) {
            selected_node = genome_visualizer
                .nodes
                .iter()
                .map(|node| node.pos.distance_squared(mouse_pos))
                .enumerate()
                .min_by(|(_, a), (_, b)| a.total_cmp(b))
                .map(|(i, _)| i)
                .unwrap_or(selected_node);
        }

        if is_key_pressed(KeyCode::A) {
            for _ in 0..10 {
                genome.mutate(&mut rng, &mut innovation_record);
            }
        }

        if is_mouse_button_down(MouseButton::Left) {
            let mut node = &mut genome_visualizer.nodes[selected_node];
            node.pos = mouse_pos;
            node.vel = Vec2::ZERO;
        }

        genome_visualizer.update(dt, &mut rng, &genome);
        genome_visualizer.draw(&genome);

        set_camera(&Camera2D::from_display_rect(Rect {
            x: -screen_width() / 2.,
            y: -screen_height() / 2.,
            w: screen_width(),
            h: screen_height(),
        }));

        next_frame().await;
    }
}
