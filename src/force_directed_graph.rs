use eframe::egui::*;
use rand::random;

use crate::node::Node;

const METERS2PIXELS: f32 = 7.0;
const PIXELS2METERS: f32 = 1.0 / METERS2PIXELS;
const NODE_SIZE: f32 = 1.0;
const SPAWN_SIZE: f32 = 5.0;
const IDEAL_DIST: f32 = NODE_SIZE * 2.0;
const GRAVITY: f32 = 9.8;
const ATTRACTION_FORCE: f32 = 0.5;
const REPLUSION_FORCE: f32 = 125.0;
const MASS: f32 = 0.125;
const FRICTION: f32 = 0.8;
const MAX_FORCE: f32 = 100.0;
const MAX_SPEED: f32 = 50.0;

const NODE_INTERACTION_SIZE: f32 = NODE_SIZE * 2.0;

#[derive(Debug, Default)]
pub struct NodeEntity {
    pub pos: Pos2,
    pub force: Vec2,
    pub vel: Vec2,
    pub fixed: bool,
}

impl NodeEntity {
    pub fn spawn() -> Self {
        NodeEntity {
            pos: pos2(
                (random::<f32>() - 0.5) * SPAWN_SIZE,
                (random::<f32>() - 0.5) * SPAWN_SIZE,
            ),
            ..Default::default()
        }
    }

    fn update(&mut self, dt: f32) {
        if self.fixed {
            return;
        }

        self.force = self
            .force
            .clamp(vec2(-MAX_FORCE, -MAX_FORCE), vec2(MAX_FORCE, MAX_FORCE));

        self.vel += (self.force / MASS) * dt;
        self.vel *= FRICTION;
        self.vel = self
            .vel
            .clamp(vec2(-MAX_SPEED, -MAX_SPEED), vec2(MAX_SPEED, MAX_SPEED));

        self.pos += self.vel * dt;

        self.force = Vec2::ZERO;
    }
}

pub trait Graph<const INPUT_SZ: usize, const OUTPUT_SZ: usize> {
    fn connected(
        &self,
        node_1: Node<INPUT_SZ, OUTPUT_SZ>,
        node_2: Node<INPUT_SZ, OUTPUT_SZ>,
    ) -> bool;

    fn size(&self) -> usize;

    fn connection_text(
        &self,
        _node_1: Node<INPUT_SZ, OUTPUT_SZ>,
        _node_2: Node<INPUT_SZ, OUTPUT_SZ>,
    ) -> Option<String> {
        None
    }

    fn node_text(&self, _node: Node<INPUT_SZ, OUTPUT_SZ>) -> Option<String> {
        None
    }
}

#[derive(Default)]
pub struct FDGraph<const INPUT_SZ: usize, const OUTPUT_SZ: usize> {
    entities: Vec<NodeEntity>,
    draw_lines: bool,
    spawner: Option<Box<dyn FnMut(Node<INPUT_SZ, OUTPUT_SZ>) -> Option<NodeEntity>>>,
}

impl<const INPUT_SZ: usize, const OUTPUT_SZ: usize> FDGraph<INPUT_SZ, OUTPUT_SZ> {
    pub fn with_spawner(
        spawner: Box<dyn FnMut(Node<INPUT_SZ, OUTPUT_SZ>) -> Option<NodeEntity>>,
        draw_lines: bool,
    ) -> Self {
        Self {
            spawner: Some(spawner),
            draw_lines,
            ..Default::default()
        }
    }

    fn update(&mut self, dt: f32, graph: &impl Graph<INPUT_SZ, OUTPUT_SZ>, space: Rect) {
        self.entities.truncate(graph.size());
        for i in self.entities.len()..graph.size() {
            self.entities.push(
                self.spawner
                    .as_mut()
                    .and_then(|spawner| spawner(Node(i)))
                    .unwrap_or_else(|| NodeEntity::spawn()),
            );
        }

        for i in 0..graph.size() {
            for j in 0..graph.size() {
                if i == j {
                    continue;
                }

                let other_pos = self.entities[i].pos;
                let this = &mut self.entities[j];

                let dist = this.pos.distance(other_pos);
                if dist == 0.0 {
                    continue;
                }
                let mut force = 0.0;

                force -= REPLUSION_FORCE / (dist * dist);

                if graph.connected(Node(i), Node(j)) {
                    let ideal_dist = dist - IDEAL_DIST;

                    force += ATTRACTION_FORCE * ideal_dist;
                }

                this.force += (other_pos - this.pos).normalized() * force;
            }
        }

        // dbg!(self.entities.first());

        // let space_size = space.size();
        for entity in &mut self.entities {
            entity.force += (Pos2::ZERO - entity.pos).normalized() * GRAVITY;

            entity.update(dt);

            entity.pos = space.clamp(entity.pos);
        }

        // let space_size = space.size();

        // for entity in &mut self.entities {
        //     let target_dist = space_size.x.min(space_size.y) / 2.0 - STAY_LENIENCE;
        //     let dist = entity.pos.distance(Pos2::ZERO);
        //
        //     if target_dist < dist {
        //         let dist_to_target_dist = dist - target_dist;
        //
        //         let stay_force = (Pos2::ZERO - entity.pos).normalized()
        //             * STAY_FORCE
        //             * dist_to_target_dist.max(1.0)
        //             * dt;
        //
        //         entity.vel += stay_force;
        //     }
        //
        //     entity.update(dt);
        // }
    }

    pub fn show(
        &mut self,
        ui: &mut Ui,
        graph: &impl Graph<INPUT_SZ, OUTPUT_SZ>,
        mut on_select: impl FnMut(usize),
    ) -> Response {
        let (response, painter) = ui.allocate_painter(ui.available_size(), Sense::drag());

        let clip_rect = painter.clip_rect();
        let world_space = Rect::from_center_size(Pos2::ZERO, clip_rect.size() * PIXELS2METERS);
        let to_screen = emath::RectTransform::from_to(world_space, painter.clip_rect());
        let from_screen = emath::RectTransform::from_to(painter.clip_rect(), world_space);

        let dt = ui.input(|i| i.stable_dt);
        ui.ctx().request_repaint();
        self.update(dt as f32, graph, world_space);

        if self.draw_lines {
            for i in 0..graph.size() {
                for j in 0..graph.size() {
                    if graph.connected(Node(i), Node(j)) {
                        let style = ui.style().noninteractive();

                        painter.line_segment(
                            [
                                to_screen * self.entities[i].pos,
                                to_screen * self.entities[j].pos,
                            ],
                            style.fg_stroke,
                        );

                        if let Some(text) = graph.connection_text(Node(i), Node(j)) {
                            let font = TextStyle::Small.resolve(ui.style());
                            let pos = self.entities[i].pos + self.entities[j].pos.to_vec2();

                            painter.text(
                                to_screen * pos2(pos.x / 2.0, pos.y / 2.0),
                                Align2::LEFT_CENTER,
                                " ".to_string() + &text,
                                font,
                                style.text_color(),
                            );
                        }
                    }
                }
            }
        }

        for (i, entity) in self.entities.iter_mut().enumerate() {
            let point_rect = Rect::from_center_size(
                to_screen * entity.pos,
                Vec2::splat(NODE_INTERACTION_SIZE * METERS2PIXELS),
            );
            let point_id = response.id.with(i);
            let point_response = ui.interact(point_rect, point_id, Sense::click_and_drag());

            if point_response.clicked() {
                on_select(i);
            }

            if point_response.dragged() {
                if let Some(pos) = ui.input(|i| i.pointer.hover_pos()) {
                    entity.pos = from_screen * pos;
                }
            }

            let style = ui.style().interact(&point_response);

            let fill = style.bg_fill;

            painter.circle_filled(to_screen * entity.pos, NODE_SIZE * METERS2PIXELS, fill);

            if let Some(text) = graph.node_text(Node(i)) {
                let font = TextStyle::Small.resolve(ui.style());

                painter.text(
                    to_screen * pos2(entity.pos.x + 1.0, entity.pos.y),
                    Align2::LEFT_CENTER,
                    " ".to_string() + &text,
                    font,
                    style.text_color(),
                );
            }
        }

        response
    }
}
