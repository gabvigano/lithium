use crate::{ecs::components, scene};
use macroquad::prelude;

pub fn display(msgs: &[String]) {
    let mut y = 25.0;

    for msg in msgs {
        prelude::draw_text(msg, 20.0, y, 20.0, prelude::WHITE);
        y += 20.0;
    }
}

pub fn render_vector(
    start_pos: &components::Pos,
    vec_x: f32,
    vec_y: f32,
    camera: &scene::Camera,
    color: prelude::Color,
    compose: bool,
) {
    let (start_x, start_y) = (start_pos.x - camera.pos().x, start_pos.y - camera.pos().y);

    if compose {
        prelude::draw_line(start_x, start_y, start_x + vec_x, start_y + vec_y, 3.0, color);
    } else {
        prelude::draw_line(start_x, start_y, start_x + vec_x, start_y, 3.0, color);
        prelude::draw_line(start_x, start_y, start_x, start_y + vec_y, 3.0, color);
    }
}
