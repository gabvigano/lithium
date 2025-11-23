use crate::{
    core::world,
    ecs::components,
    math::{self, algebra},
    renderer::scene,
};

use macroquad::{math as mq_math, prelude as mq_prelude};

#[inline]
pub fn color_to_mq(color: math::Color) -> mq_prelude::Color {
    let math::Color { r, g, b, a } = color;

    mq_prelude::Color {
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

    let math::Vec2 { x: cam_x, y: cam_y } = camera.pos();

    for (material, entity) in pairs {
        if material.show {
            let Some(&components::Transform { pos, .. }) = world.transform.get(entity) else {
                continue;
            };
            let Some(shape) = world.shape.get(entity) else {
                continue;
            };

            let color = color_to_mq(material.color);

            let rot_mat = world.rotation_matrix.get(entity);
            let rot_mat_is_none = rot_mat.is_none();
            let rot_mat = if rot_mat_is_none {
                &algebra::IDENTITY_MAT2X3
            } else {
                &rot_mat.unwrap().curr
            };

            match shape {
                math::Shape::Segment(segment) => {
                    let (a, b) = if rot_mat_is_none {
                        (pos.add(segment.a), pos.add(segment.b))
                    } else {
                        (
                            pos.add(rot_mat.pre_mul_vec2(segment.a)),
                            pos.add(rot_mat.pre_mul_vec2(segment.b)),
                        )
                    };

                    mq_prelude::draw_line(a.x - cam_x, a.y - cam_y, b.x - cam_x, b.y - cam_y, 1.0, color);
                }
                math::Shape::Triangle(triangle) => {
                    let (a, b, c) = if rot_mat_is_none {
                        (pos.add(triangle.a), pos.add(triangle.b), pos.add(triangle.c))
                    } else {
                        (
                            pos.add(rot_mat.pre_mul_vec2(triangle.a)),
                            pos.add(rot_mat.pre_mul_vec2(triangle.b)),
                            pos.add(rot_mat.pre_mul_vec2(triangle.c)),
                        )
                    };

                    mq_prelude::draw_triangle(
                        mq_math::Vec2::new(a.x - cam_x, a.y - cam_y),
                        mq_math::Vec2::new(b.x - cam_x, b.y - cam_y),
                        mq_math::Vec2::new(c.x - cam_x, c.y - cam_y),
                        color,
                    )
                }
                math::Shape::Quad(quad) => {
                    let (a, b, c, d) = if rot_mat_is_none {
                        (pos.add(quad.a), pos.add(quad.b), pos.add(quad.c), pos.add(quad.d))
                    } else {
                        (
                            pos.add(rot_mat.pre_mul_vec2(quad.a)),
                            pos.add(rot_mat.pre_mul_vec2(quad.b)),
                            pos.add(rot_mat.pre_mul_vec2(quad.c)),
                            pos.add(rot_mat.pre_mul_vec2(quad.d)),
                        )
                    };

                    mq_prelude::draw_triangle(
                        mq_math::Vec2::new(a.x - cam_x, a.y - cam_y),
                        mq_math::Vec2::new(b.x - cam_x, b.y - cam_y),
                        mq_math::Vec2::new(c.x - cam_x, c.y - cam_y),
                        color,
                    );

                    mq_prelude::draw_triangle(
                        mq_math::Vec2::new(a.x - cam_x, a.y - cam_y),
                        mq_math::Vec2::new(c.x - cam_x, c.y - cam_y),
                        mq_math::Vec2::new(d.x - cam_x, d.y - cam_y),
                        color,
                    );
                }
                math::Shape::Polygon(polygon) => {
                    if rot_mat_is_none {
                        let v0 = pos.add(polygon.verts[0]);
                        let mut vi = pos.add(polygon.verts[1]);

                        for i in 1..(polygon.verts.len() - 1) {
                            let vi1 = pos.add(polygon.verts[i + 1]);

                            mq_prelude::draw_triangle(
                                mq_math::Vec2::new(v0.x - cam_x, v0.y - cam_y),
                                mq_math::Vec2::new(vi.x - cam_x, vi.y - cam_y),
                                mq_math::Vec2::new(vi1.x - cam_x, vi1.y - cam_y),
                                color,
                            );

                            vi = vi1;
                        }
                    } else {
                        let v0 = pos.add(rot_mat.pre_mul_vec2(polygon.verts[0]));
                        let mut vi = pos.add(rot_mat.pre_mul_vec2(polygon.verts[1]));

                        for i in 1..(polygon.verts.len() - 1) {
                            let vi1 = pos.add(rot_mat.pre_mul_vec2(polygon.verts[i + 1]));

                            mq_prelude::draw_triangle(
                                mq_math::Vec2::new(v0.x - cam_x, v0.y - cam_y),
                                mq_math::Vec2::new(vi.x - cam_x, vi.y - cam_y),
                                mq_math::Vec2::new(vi1.x - cam_x, vi1.y - cam_y),
                                color,
                            );

                            vi = vi1;
                        }
                    }
                }
                math::Shape::Circle(_) => {
                    unimplemented!();
                    // mq_prelude::draw_circle(
                    //     pos.x + circle.radius - cam_x, // sum radius because macroquad use centre for circles instead of top left
                    //     pos.y + circle.radius - cam_y,
                    //     circle.radius,
                    //     color,
                    // )
                }
            }
        }
    }
}
