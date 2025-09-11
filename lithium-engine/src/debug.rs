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
    mut start_pos: components::Vec2,
    mut vec: components::Vec2,
    scale: Option<f32>,
    camera: &scene::Camera,
    color: prelude::Color,
    compose: bool,
) {
    start_pos.sub_mut(camera.pos());

    if let Some(scale_value) = scale {
        vec.scale_mut(scale_value);
    }

    vec.add_mut(start_pos);

    if compose {
        prelude::draw_line(start_pos.x, start_pos.y, vec.x, vec.y, 3.0, color);
    } else {
        prelude::draw_line(start_pos.x, start_pos.y, vec.x, start_pos.y, 3.0, color);
        prelude::draw_line(start_pos.x, start_pos.y, start_pos.x, vec.y, 3.0, color);
    }
}
