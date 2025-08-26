use crate::{ecs::components, scene, world};
use macroquad::prelude;

#[inline]
pub fn color_to_mq(color: components::Color) -> prelude::Color {
    let components::Color { r, g, b, a } = color;

    prelude::Color {
        r: (r as f32) / 255.0,
        g: (g as f32) / 255.0,
        b: (b as f32) / 255.0,
        a: (a as f32) / 255.0,
    }
}

pub fn render(world: &world::World, camera: &scene::Camera) {
    // get reference of the material vector
    let mats = world.material.get_ref();

    // copy entities implementing material
    let ents = world.material.get_ents();

    // zip vector toghether
    let mut pairs: Vec<(&components::Material, u32)> = mats.iter().zip(ents).collect();

    // sort by layer
    pairs.sort_by_key(|(m, _)| m.layer);

    let components::Vec2 { x: cam_x, y: cam_y } = camera.pos();

    for (material, entity) in pairs {
        if material.show {
            let Some(&components::Transform { pos, .. }) = world.transform.get(entity) else {
                continue;
            };
            let Some(shape) = world.shape.get(entity) else {
                continue;
            };

            match shape {
                components::Shape::Rect(rect) => prelude::draw_rectangle(
                    pos.x - cam_x,
                    pos.y - cam_y,
                    rect.width,
                    rect.height,
                    color_to_mq(material.color),
                ),
                components::Shape::Circle(circle) => prelude::draw_circle(
                    pos.x + circle.radius - cam_x, // sum radius because macroquad use centre for circles instead of top left
                    pos.y + circle.radius - cam_y,
                    circle.radius,
                    color_to_mq(material.color),
                ),
                _ => unimplemented!(),
            }
        }
    }
}
