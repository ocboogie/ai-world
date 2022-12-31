use crate::genome::Genome;
use crate::innovation_record::InnovationRecord;
use ::rand::Rng;
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

    pub fn update(&mut self, dt: f32, rng: &mut impl Rng, genome: &Genome<INPUT_SZ, OUTPUT_SZ>) {
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

    pub fn draw(&self, genome: &Genome<INPUT_SZ, OUTPUT_SZ>) {
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