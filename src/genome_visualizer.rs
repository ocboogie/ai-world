use std::iter::once;

use crate::genome::{Genome, GenomeActivation};
use crate::node::Node;
use ::rand::Rng;
use macroquad::prelude::*;

const NODE_COLOR: Color = GREEN;
const NODE_SIZE: f32 = 10.0;
const TEXT_COLOR: Color = YELLOW;
const INDEX_TEXT_COLOR: Color = BLACK;
const TEXT_SIZE: f32 = 20.0;
const NODE_DIST: f32 = 50.0;
const CONNECTION_THICKNESS: f32 = 5.0;
const CONNECTION_COLOR: Color = RED;
const SPAWN_SIZE: f32 = 100.0;
const ATTRACTION_FORCE: f32 = 50.0;
const REJECTION_FORCE: f32 = 3000000.0;
const DRAG_FORCE: f32 = 0.8;
const MIN_DRAG_DIST: f32 = 50.0 * 50.0;
const INPUT_OUTPUT_DIST: f32 = 300.0;

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

    fn draw(&self, index: usize, activation: Option<f32>) {
        draw_circle(self.pos.x, self.pos.y, NODE_SIZE, NODE_COLOR);
        draw_text(
            &format!("{}", index),
            self.pos.x - 5.0,
            self.pos.y + 4.0,
            TEXT_SIZE,
            INDEX_TEXT_COLOR,
        );
        if let Some(activation) = activation {
            draw_text(
                &format!("{:.2}", activation),
                self.pos.x + NODE_SIZE,
                self.pos.y - NODE_SIZE,
                TEXT_SIZE,
                TEXT_COLOR,
            );
        }
    }
}

#[derive(Default, Debug)]
pub struct GenomeVisualizer<const INPUT_SZ: usize, const OUTPUT_SZ: usize> {
    nodes: Vec<NeuronVisualizer>,
    selected_node: Option<usize>,
}

fn get_corrected_mouse_pos() -> Vec2 {
    Vec2::from(mouse_position()) - vec2(screen_width() / 2.0, screen_height() / 2.0)
}

impl<const INPUT_SZ: usize, const OUTPUT_SZ: usize> GenomeVisualizer<INPUT_SZ, OUTPUT_SZ> {
    pub fn new() -> Self {
        let input_width = INPUT_SZ as f32 * NODE_DIST;
        let output_width = OUTPUT_SZ as f32 * NODE_DIST;

        let nodes = once(NeuronVisualizer {
            pos: vec2(
                -input_width / 2.0 - NODE_DIST * 1.5,
                INPUT_OUTPUT_DIST / 2.0,
            ),
            vel: Vec2::ZERO,
            fixed: true,
        })
        .chain((0..INPUT_SZ).into_iter().map(|i| NeuronVisualizer {
            pos: vec2(
                (input_width / INPUT_SZ as f32) * i as f32 - input_width / 2.0,
                INPUT_OUTPUT_DIST / 2.0,
            ),
            vel: Vec2::ZERO,
            fixed: true,
        }))
        .chain((0..OUTPUT_SZ).into_iter().map(|i| NeuronVisualizer {
            pos: vec2(
                (output_width / OUTPUT_SZ as f32) * i as f32 - output_width / 2.0,
                -INPUT_OUTPUT_DIST / 2.0,
            ),
            vel: Vec2::ZERO,
            fixed: true,
        }))
        .collect();

        GenomeVisualizer {
            nodes,
            selected_node: None,
        }
    }

    fn spawn_node(&mut self, rng: &mut impl Rng) {
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
            self.spawn_node(rng);
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

        let mouse_pos = get_corrected_mouse_pos();
        if is_mouse_button_pressed(MouseButton::Left) {
            self.selected_node = self
                .nodes
                .iter()
                .map(|node| node.pos.distance_squared(mouse_pos))
                .enumerate()
                .min_by(|(_, a), (_, b)| a.total_cmp(b))
                .and_then(
                    |(i, dist)| {
                        if dist < MIN_DRAG_DIST {
                            Some(i)
                        } else {
                            None
                        }
                    },
                );
        }

        if let Some(selected_node) = self.selected_node {
            if is_mouse_button_down(MouseButton::Left) {
                let mut node = &mut self.nodes[selected_node];
                node.pos = mouse_pos;
                node.vel = Vec2::ZERO;
            }
        }
    }

    pub fn draw(
        &self,
        genome: &Genome<INPUT_SZ, OUTPUT_SZ>,
        activation: Option<&GenomeActivation<INPUT_SZ, OUTPUT_SZ>>,
    ) {
        for connection in genome.connections.iter().filter(|c| c.enabled) {
            let in_pos = self.nodes[*connection.in_node].pos;
            let out_pos = self.nodes[*connection.out_node].pos;

            draw_line(
                in_pos.x,
                in_pos.y,
                out_pos.x,
                out_pos.y,
                CONNECTION_THICKNESS,
                Color {
                    r: CONNECTION_COLOR.r,
                    g: CONNECTION_COLOR.g,
                    b: CONNECTION_COLOR.b,
                    a: connection.weight,
                },
            );
        }

        for (i, node) in self.nodes.iter().enumerate().take(genome.nodes()) {
            node.draw(i, activation.map(|activation| activation[Node(i)]));
        }
    }
}
