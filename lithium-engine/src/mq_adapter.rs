use crate::{
    ecs::{components, entities},
    scene, world,
};
use macroquad::prelude;

pub fn color_to_mq(color: &components::Color) -> prelude::Color {
    let components::Color { r, g, b, a } = color;

    prelude::Color {
        r: (*r as f32) / 255.0,
        g: (*g as f32) / 255.0,
        b: (*b as f32) / 255.0,
        a: (*a as f32) / 255.0,
    }
}

pub fn render(world: &world::World, entity: entities::Entity, camera: &scene::Camera) {
    if *world.show.get(entity).expect("missing show") {
        let components::Pos { x, y } = world.pos.get(entity).expect("missing position");
        let components::Pos { x: cam_x, y: cam_y } = camera.pos();

        match world.shape.get(entity).expect("missing shape") {
            components::Shape::Rect(rect) => prelude::draw_rectangle(
                x - cam_x,
                y - cam_y,
                rect.width,
                rect.height,
                color_to_mq(world.color.get(entity).expect("missing color")),
            ),
            components::Shape::Circle(circle) => prelude::draw_circle(
                x + circle.radius - cam_x, // sum radius because macroquad use centre for circles instead of top left
                y + circle.radius - cam_y,
                circle.radius,
                color_to_mq(world.color.get(entity).expect("missing color")),
            ),
        }
    }
}
