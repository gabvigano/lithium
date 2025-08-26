use crate::{ecs::components, scene, world};
use macroquad::{math, prelude};

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
                components::Shape::Segment(segment) => prelude::draw_line(
                    pos.x + segment.a.x - cam_x,
                    pos.y + segment.a.y - cam_y,
                    pos.x + segment.b.x - cam_x,
                    pos.y + segment.b.y - cam_y,
                    1.0,
                    color_to_mq(material.color),
                ),

                components::Shape::Triangle(triangle) => prelude::draw_triangle(
                    math::Vec2::new(pos.x + triangle.a.x - cam_x, pos.y + triangle.a.y - cam_y),
                    math::Vec2::new(pos.x + triangle.b.x - cam_x, pos.y + triangle.b.y - cam_y),
                    math::Vec2::new(pos.x + triangle.c.x - cam_x, pos.y + triangle.c.y - cam_y),
                    color_to_mq(material.color),
                ),
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
                components::Shape::Polygon(polygon) => {
                    for i in 0..(polygon.verts.len() - 1) {
                        prelude::draw_triangle(
                            math::Vec2::new(pos.x + polygon.verts[0].x - cam_x, pos.y + polygon.verts[0].y - cam_y),
                            math::Vec2::new(pos.x + polygon.verts[i].x - cam_x, pos.y + polygon.verts[i].y - cam_y),
                            math::Vec2::new(
                                pos.x + polygon.verts[i + 1].x - cam_x,
                                pos.y + polygon.verts[i + 1].y - cam_y,
                            ),
                            color_to_mq(material.color),
                        )
                    }
                }
            }
        }
    }
}
