#![feature(generic_const_exprs)]
use std::f32::consts::TAU;

mod learning;

use ::rand::random;
use macroquad::prelude::*;

const ACTOR_SIZE: f32 = 10.;
const ACTOR_SPEED: f32 = 100.;
const ACTOR_ROTATION_SPEED: f32 = TAU / 2.;

const ARENA_SIZE: f32 = 400.;

const ROUND_TIME: f32 = 10.;

const NEURON_SIZE: f32 = 10.;
const NEURON_GAP: f32 = 30.;

const LAYER_MARGIN: f32 = 50.;
const SCREEN_MARGIN: f32 = 20.;

struct BrainOutput {
    forward: f32,
    left: f32,
    right: f32,
}

struct BrainInput {
    rotation: f32,
    x: f32,
    y: f32,
}

struct Actor {
    pub pos: Vec2,
    pub rotation: f32,
    pub brain: Layer,

    pub brain_input: BrainInput,
    pub brain_output: BrainOutput,

    pub time_finished: Option<f32>,
}

struct Layer {
    pub input_size: usize,
    pub output_size: usize,
    pub weights: Vec<f32>,
    pub biases: Vec<f32>,
}

impl Layer {
    pub fn new(input_size: usize, output_size: usize) -> Self {
        Layer {
            input_size,
            output_size,
            weights: (0..(input_size * output_size))
                .into_iter()
                .map(|_| random::<f32>())
                .collect(),
            biases: (0..output_size)
                .into_iter()
                .map(|_| random::<f32>())
                .collect(),
        }
    }

    pub fn sigmoid(x: f32) -> f32 {
        1. / (1. + (-x).exp())
    }

    pub fn calculate(&self, input: &[f32]) -> Vec<f32> {
        assert_eq!(self.input_size, input.len());

        (0..self.output_size)
            .into_iter()
            .map(|o| {
                Self::sigmoid(
                    (0..self.input_size)
                        .into_iter()
                        .map(|i| input[i] * self.weights[o + i * self.input_size])
                        .sum::<f32>()
                        + self.biases[o],
                )
            })
            .collect()
    }

    pub fn mix(&self, other: &Layer) -> Layer {
        Layer {
            input_size: self.input_size,
            output_size: self.output_size,
            weights: self
                .weights
                .iter()
                .zip(other.weights.iter())
                .map(|(a, b)| (a + b) / 2. + random::<f32>() / 2. - 0.25)
                .collect(),
            biases: self
                .biases
                .iter()
                .zip(other.biases.iter())
                .map(|(a, b)| (a + b) / 2. + random::<f32>() / 2. - 0.25)
                .collect(),
        }
    }
}

impl Actor {
    pub fn think(&mut self) {
        self.brain_input = BrainInput {
            rotation: self.rotation / TAU,
            x: (self.pos.x + ARENA_SIZE / 2.) / ARENA_SIZE,
            y: (self.pos.y + ARENA_SIZE / 2.) / ARENA_SIZE,
        };
        let output = self.brain.calculate(&[
            self.brain_input.rotation,
            self.brain_input.x,
            self.brain_input.y,
        ]);
        self.brain_output = BrainOutput {
            forward: output[0],
            left: output[1],
            right: output[2],
        }
    }

    pub fn act(&mut self, delta_time: f32) -> Vec2 {
        self.rotation +=
            ACTOR_ROTATION_SPEED * delta_time * self.brain_output.right.min(1.).max(0.)
                - ACTOR_ROTATION_SPEED * delta_time * self.brain_output.left.min(1.).max(0.);

        if self.rotation < 0. {
            self.rotation += TAU;
        }
        if self.rotation > TAU {
            self.rotation -= TAU;
        }

        vec2(
            ACTOR_SPEED
                * self.rotation.cos()
                * delta_time
                * self.brain_output.forward.min(1.).max(0.),
            ACTOR_SPEED
                * self.rotation.sin()
                * delta_time
                * self.brain_output.forward.min(1.).max(0.),
        )
    }
}

struct World {
    actors: Vec<Actor>,
    walls: Vec<Rect>,
    finish: Rect,

    since_round_start: f32,
}

impl World {
    fn update(&mut self, delta_time: f32) {
        self.since_round_start += delta_time;
        // dbg!(&self.since_round_start);

        for actor in self
            .actors
            .iter_mut()
            .filter(|actor| actor.time_finished.is_none())
        {
            actor.think();
            let velocity = actor.act(delta_time);

            let collides = self.walls.iter().any(|wall| {
                wall.overlaps(&Rect {
                    x: actor.pos.x + velocity.x - ACTOR_SIZE / 2.,
                    y: actor.pos.y + velocity.y - ACTOR_SIZE / 2.,
                    w: ACTOR_SIZE,
                    h: ACTOR_SIZE,
                })
            });

            if !collides {
                actor.pos += velocity;
            }

            if self.finish.overlaps(&Rect {
                x: actor.pos.x - ACTOR_SIZE / 2.,
                y: actor.pos.y - ACTOR_SIZE / 2.,
                w: ACTOR_SIZE,
                h: ACTOR_SIZE,
            }) {
                actor.time_finished = Some(self.since_round_start);
                dbg!(&actor.time_finished);
            }
        }

        let mut winners = self
            .actors
            .iter()
            .filter(|actor| actor.time_finished.is_some());

        if let Some((first_winner, second_winner)) =
            winners.next().and_then(|a| Some((a, winners.next()?)))
        {
            let new_actors = (0..100)
                .into_iter()
                .map(|_| Actor {
                    pos: vec2(0., 100.),
                    rotation: 3. * (TAU / 4.),
                    brain: first_winner.brain.mix(&second_winner.brain),

                    time_finished: None,

                    brain_output: BrainOutput {
                        forward: 0.,
                        left: 0.,
                        right: 0.,
                    },
                    brain_input: BrainInput {
                        rotation: 0.,
                        x: 0.,
                        y: 0.,
                    },
                })
                .collect();

            self.actors = new_actors;
            self.since_round_start = 0.;
        }
    }
}

#[macroquad::main("Events")]
async fn main() {
    let mut world = World {
        actors: (0..500)
            .into_iter()
            .map(|_| Actor {
                pos: vec2(0., 100.),
                rotation: 3. * (TAU / 4.),
                brain: Layer::new(3, 3),

                time_finished: None,

                brain_output: BrainOutput {
                    forward: 0.,
                    left: 0.,
                    right: 0.,
                },
                brain_input: BrainInput {
                    rotation: 0.,
                    x: 0.,
                    y: 0.,
                },
            })
            .collect(),
        // walls: vec![Rect {
        //     x: -50.,
        //     y: -10.,
        //     w: 100.,
        //     h: 20.,
        // }],
        walls: vec![Rect {
            x: -50.,
            y: -10.,
            w: 100.,
            h: 20.,
        }],
        finish: Rect {
            x: -100.,
            y: -110.,
            w: 200.,
            h: 20.,
        },

        since_round_start: 0.,
    };

    let mut select: usize = 0;

    loop {
        world.update(get_frame_time());

        clear_background(BLACK);

        set_camera(&Camera2D::from_display_rect(Rect {
            x: -screen_width() / 2.,
            y: -screen_height() / 2.,
            w: screen_width(),
            h: screen_height(),
        }));

        if is_key_pressed(KeyCode::Right) {
            select = (select + 1) % world.actors.len();
        }
        if is_key_pressed(KeyCode::Left) {
            if select == 0 {
                select = world.actors.len() - 1;
            } else {
                select -= 1;
            }
        }

        for (i, actor) in world.actors.iter().enumerate() {
            draw_triangle(
                vec2(
                    actor.pos.x + ACTOR_SIZE * (actor.rotation).cos(),
                    actor.pos.y + ACTOR_SIZE * (actor.rotation).sin(),
                ),
                vec2(
                    actor.pos.x + ACTOR_SIZE / 2. * (actor.rotation + TAU / 3.).cos(),
                    actor.pos.y + ACTOR_SIZE / 2. * (actor.rotation + TAU / 3.).sin(),
                ),
                vec2(
                    actor.pos.x + ACTOR_SIZE / 2. * (actor.rotation + 2. * (TAU / 3.)).cos(),
                    actor.pos.y + ACTOR_SIZE / 2. * (actor.rotation + 2. * (TAU / 3.)).sin(),
                ),
                if i == select { YELLOW } else { RED },
            );
        }

        for wall in &world.walls {
            draw_rectangle(wall.x, wall.y, wall.w, wall.h, WHITE);
        }

        let finish = world.finish;
        draw_rectangle(finish.x, finish.y, finish.w, finish.h, GREEN);

        draw_rectangle_lines(
            -ARENA_SIZE / 2.,
            -ARENA_SIZE / 2.,
            ARENA_SIZE,
            ARENA_SIZE,
            2.,
            WHITE,
        );

        set_default_camera();

        let selected_actor = &world.actors[select];

        draw_circle(
            NEURON_SIZE + SCREEN_MARGIN,
            NEURON_SIZE + SCREEN_MARGIN,
            NEURON_SIZE,
            Color::new(1.0, 1.0, 1.0, selected_actor.brain_input.rotation),
        );
        draw_circle(
            NEURON_SIZE + SCREEN_MARGIN,
            NEURON_SIZE + SCREEN_MARGIN + NEURON_GAP,
            NEURON_SIZE,
            Color::new(1.0, 1.0, 1.0, selected_actor.brain_input.x),
        );
        draw_circle(
            NEURON_SIZE + SCREEN_MARGIN,
            NEURON_SIZE + SCREEN_MARGIN + NEURON_GAP * 2.,
            NEURON_SIZE,
            Color::new(1.0, 1.0, 1.0, selected_actor.brain_input.y),
        );

        draw_circle(
            NEURON_SIZE + SCREEN_MARGIN + LAYER_MARGIN,
            NEURON_SIZE + SCREEN_MARGIN,
            NEURON_SIZE,
            Color::new(1.0, 1.0, 1.0, selected_actor.brain.biases[0]),
        );
        draw_circle(
            NEURON_SIZE + SCREEN_MARGIN + LAYER_MARGIN,
            NEURON_SIZE + NEURON_GAP + SCREEN_MARGIN,
            NEURON_SIZE,
            Color::new(1.0, 1.0, 1.0, selected_actor.brain.biases[1]),
        );
        draw_circle(
            NEURON_SIZE + SCREEN_MARGIN + LAYER_MARGIN,
            NEURON_SIZE + NEURON_GAP * 2. + SCREEN_MARGIN,
            NEURON_SIZE,
            Color::new(1.0, 1.0, 1.0, selected_actor.brain.biases[2]),
        );

        for i in 0..selected_actor.brain.input_size {
            for o in 0..selected_actor.brain.output_size {
                draw_line(
                    NEURON_SIZE + SCREEN_MARGIN,
                    NEURON_SIZE + SCREEN_MARGIN + NEURON_GAP * i as f32,
                    NEURON_SIZE + SCREEN_MARGIN + LAYER_MARGIN,
                    NEURON_SIZE + SCREEN_MARGIN + NEURON_GAP * o as f32,
                    2.0,
                    Color::new(
                        1.0,
                        1.0,
                        1.0,
                        selected_actor.brain.weights[o + i * selected_actor.brain.input_size],
                    ),
                )
            }
        }

        next_frame().await;
    }
}
