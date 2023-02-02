use eframe::egui::*;
use rand::random;

use crate::node::Node;

const NODE_SIZE: f32 = 5.0;
const NODE_DIST: f32 = 20.0;
const SPAWN_SIZE: f32 = 100.0;
const STAY_LENIENCE: f32 = 25.0;
const STAY_FORCE: f32 = 85.0;
const ATTRACTION_FORCE: f32 = 3.0;
const REJECTION_FORCE: f32 = 580000.0;
const DRAG_FORCE: f32 = 0.8;
const MAX_SPEED: f32 = 500.0;

#[derive(Debug, Default)]
pub struct NodeEntity {
    pub pos: Pos2,
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

        self.vel = self
            .vel
            .clamp(Vec2::splat(-MAX_SPEED), Vec2::splat(MAX_SPEED));

        self.pos += self.vel * dt;
        self.vel *= DRAG_FORCE * dt;
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
                let rejection_force =
                    (other_pos - this.pos).normalized() * (REJECTION_FORCE / (dist * dist)) * dt;

                this.vel -= rejection_force;

                if graph.connected(Node(i), Node(j)) {
                    let ideal_dist = dist - NODE_DIST;
                    let attraction_force =
                        (other_pos - this.pos).normalized() * ATTRACTION_FORCE * ideal_dist * dt;

                    this.vel += attraction_force;
                }
            }
        }

        let space_size = space.size();

        for entity in &mut self.entities {
            let target_dist = space_size.x.min(space_size.y) / 2.0 - STAY_LENIENCE;
            let dist = entity.pos.distance(Pos2::ZERO);

            if target_dist < dist {
                let dist_to_target_dist = dist - target_dist;

                let stay_force = (Pos2::ZERO - entity.pos).normalized()
                    * STAY_FORCE
                    * dist_to_target_dist.max(1.0)
                    * dt;

                entity.vel += stay_force;
            }

            entity.update(dt);
        }
    }

    pub fn show(
        &mut self,
        ui: &mut Ui,
        graph: &impl Graph<INPUT_SZ, OUTPUT_SZ>,
        mut on_select: impl FnMut(usize),
    ) -> Response {
        let (response, painter) = ui.allocate_painter(ui.available_size(), Sense::drag());

        let mut space = painter.clip_rect();
        space.set_center(Pos2::ZERO);
        let to_screen = emath::RectTransform::from_to(space, painter.clip_rect());
        let from_screen = emath::RectTransform::from_to(painter.clip_rect(), space);

        let dt = ui.input().stable_dt;
        ui.ctx().request_repaint();
        self.update(dt as f32, graph, space);

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
            let point_rect =
                Rect::from_center_size(to_screen * entity.pos, Vec2::splat(NODE_SIZE * 1.5));
            let point_id = response.id.with(i);
            let point_response = ui.interact(point_rect, point_id, Sense::click_and_drag());

            if point_response.clicked() {
                on_select(i);
            }

            if point_response.dragged() {
                if let Some(pos) = ui.input().pointer.hover_pos() {
                    entity.pos = from_screen * pos;
                }
            }

            let fill = ui.style().interact(&point_response).bg_fill;

            painter.circle_filled(to_screen * entity.pos, NODE_SIZE, fill);
        }

        response
    }
}
