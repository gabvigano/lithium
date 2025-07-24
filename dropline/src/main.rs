use lithium_engine::{
    debug,
    ecs::{
        components, entities,
        systems::{collision, physics},
    },
    mq_adapter, scene,
    world::World,
};

use macroquad::prelude;

const GRAVITY: components::Acc = components::Acc { x: 0.0, y: 0.25 };
// const GROUND_FRICTION: f32 = 0.15;
// const AIR_FRICTION: f32 = 0.015;
// const SWING_FRICTION: f32 = 0.0015;
// const ROPE_SPEED: usize = 40;

fn get_window_config() -> prelude::Conf {
    prelude::Conf {
        window_title: String::from("dropline"),
        window_width: 1600,
        window_height: 900,
        window_resizable: false,
        ..Default::default()
    }
}

#[macroquad::main(get_window_config())]
async fn main() {
    // initialize environment
    let mut entity_manager = entities::EntityManager::new();
    let mut world = World::new();

    fn add_map(
        world: &mut World,
        entity: entities::Entity,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        r: u8,
        g: u8,
        b: u8,
        a: u8,
    ) {
        world.pos.insert(entity, components::Pos { x: x, y: y });
        world.elast.insert(entity, components::Elast(1.0)); // keep constant for static objects that compose the game map
        world.shape.insert(
            entity,
            components::Shape::Rect(components::Rect {
                width: width,
                height: height,
            }),
        );
        world.color.insert(entity, components::Color { r: r, g: g, b: b, a: a });
        world.layer.insert(entity, components::Layer(0));
        world.show.insert(entity, true);
    }

    // create game map
    // make sure in 0..x x is the number of times you call add_map(); it is important you only allocate the objects you insert!
    let game_map: Vec<_> = (0..1).map(|_| entity_manager.create()).collect();
    add_map(&mut world, game_map[0], 0.0, 500.0, 1500.0, 150.0, 255, 255, 255, 255);
    // add_map(&mut world, game_map[1], 0.0, 450.0, 50.0, 50.0, 255, 255, 255, 255);

    // create player
    let player = entity_manager.create();
    world.pos.insert(player, components::Pos { x: 200.0, y: 100.0 });
    world.vel.insert(player, components::Vel { x: 0.0, y: 0.0 });
    world.acc.insert(player, components::Acc { x: 0.0, y: 0.0 });
    world.rest.insert(player, false);
    world.mass.insert(player, components::Mass(1.0));
    world.elast.insert(player, components::Elast(0.5));
    world
        .shape
        .insert(player, components::Shape::Circle(components::Circle { radius: 10.0 }));
    world.color.insert(
        player,
        components::Color {
            r: 0,
            g: 255,
            b: 0,
            a: 255,
        },
    );
    world.layer.insert(player, components::Layer(2));
    world.show.insert(player, true);

    // create square_1
    let square_1 = entity_manager.create();
    world.pos.insert(square_1, components::Pos { x: 100.0, y: 250.0 });
    world.vel.insert(square_1, components::Vel { x: 0.0, y: 0.0 });
    world.acc.insert(square_1, components::Acc { x: 0.0, y: 0.0 });
    world.rest.insert(square_1, false);
    world.mass.insert(square_1, components::Mass(2.5));
    world.elast.insert(square_1, components::Elast(0.5));
    world.shape.insert(
        square_1,
        components::Shape::Rect(components::Rect {
            width: 125.0,
            height: 75.0,
        }),
    );
    world.color.insert(
        square_1,
        components::Color {
            r: 255,
            g: 0,
            b: 255,
            a: 255,
        },
    );
    world.layer.insert(square_1, components::Layer(2));
    world.show.insert(square_1, true);

    // create square_2
    let square_2 = entity_manager.create();
    world.pos.insert(square_2, components::Pos { x: 130.0, y: 0.0 });
    world.vel.insert(square_2, components::Vel { x: 0.0, y: 0.0 });
    world.acc.insert(square_2, components::Acc { x: 0.0, y: 0.0 });
    world.rest.insert(square_2, false);
    world.mass.insert(square_2, components::Mass(0.5));
    world.elast.insert(square_2, components::Elast(0.65));
    world.shape.insert(
        square_2,
        components::Shape::Rect(components::Rect {
            width: 15.0,
            height: 15.0,
        }),
    );
    world.color.insert(
        square_2,
        components::Color {
            r: 255,
            g: 255,
            b: 0,
            a: 255,
        },
    );
    world.layer.insert(square_2, components::Layer(2));
    world.show.insert(square_2, true);

    // create camera
    let mut camera = scene::Camera::new(
        components::Pos { x: 0.0, y: -100.0 },
        components::Rect {
            width: prelude::screen_width(),
            height: prelude::screen_height(),
        },
    );

    // game loop
    loop {
        // empty frame
        prelude::clear_background(prelude::BLACK);

        // reset accellerations
        world.acc.set(player, GRAVITY);
        world.acc.set(square_1, GRAVITY);
        world.acc.set(square_2, GRAVITY);

        // handle user inputs
        if prelude::is_key_down(prelude::KeyCode::W) && *world.rest.get(player).expect("missing rest") {
            physics::apply_vel(&mut world, player, -12.0, Some(-12.0), components::Axis::Vertical);
        }
        if prelude::is_key_down(prelude::KeyCode::D) {
            physics::apply_vel(&mut world, player, 1.0, Some(7.0), components::Axis::Horizontal);
        }
        if prelude::is_key_down(prelude::KeyCode::A) {
            physics::apply_vel(&mut world, player, -1.0, Some(-7.0), components::Axis::Horizontal);
        }
        if prelude::is_key_down(prelude::KeyCode::R) {
            world.pos.set(player, components::Pos { x: 200.0, y: 100.0 });
            world.vel.set(player, components::Vel { x: 0.0, y: 0.0 });
            world.acc.set(player, components::Acc { x: 0.0, y: 0.0 });

            world.pos.set(square_1, components::Pos { x: 100.0, y: 250.0 });
            world.vel.set(square_1, components::Vel { x: 0.0, y: 0.0 });
            world.acc.set(square_1, components::Acc { x: 0.0, y: 0.0 });

            world.pos.set(square_2, components::Pos { x: 130.0, y: 0.0 });
            world.vel.set(square_2, components::Vel { x: 0.0, y: 0.0 });
            world.acc.set(square_2, components::Acc { x: 0.0, y: 0.0 });
        }

        // update world and camera
        physics::update_vel(&mut world);
        let collisions = collision::compute_collision(&mut world);
        physics::reset_rest(&mut world);
        collision::simulate_collisions(&mut world, collisions);
        physics::update_pos(&mut world);

        camera.update(world.pos.get(player).expect("missing position"));

        // render entities
        for map_obj in &game_map {
            mq_adapter::render(&mut world, *map_obj, &camera);
        }

        mq_adapter::render(&mut world, player, &camera);
        mq_adapter::render(&mut world, square_1, &camera);
        mq_adapter::render(&mut world, square_2, &camera);

        // render text
        prelude::draw_text(
            &format!("FPS: {}", prelude::get_fps()),
            prelude::screen_width() - 100.0,
            25.0,
            25.0,
            prelude::WHITE,
        );

        // prelude::draw_text(
        //     &format!("vel_x: {}", world.vel.get(player).expect("missing velocity").x),
        //     world.pos.get(player).expect("missing position").x - camera.pos().x,
        //     world.pos.get(player).expect("missing position").y - 25.0 - camera.pos().y,
        //     25.0,
        //     prelude::GREEN,
        // );

        // prelude::draw_text(
        //     &format!("vel_x: {}", world.vel.get(square_1).expect("missing velocity").x),
        //     world.pos.get(square_1).expect("missing position").x - camera.pos().x,
        //     world.pos.get(square_1).expect("missing position").y - 25.0 - camera.pos().y,
        //     25.0,
        //     prelude::PURPLE,
        // );

        // prelude::draw_text(
        //     &format!("vel_x: {}", world.vel.get(square_2).expect("missing velocity").x),
        //     world.pos.get(square_2).expect("missing position").x - camera.pos().x,
        //     world.pos.get(square_2).expect("missing position").y - 25.0 - camera.pos().y,
        //     25.0,
        //     prelude::YELLOW,
        // );

        debug::display(&[
            format!("player_id: {:?}", player),
            format!("player_rest: {:?}", world.rest.get(player)),
            format!("player_pos: {:?}", world.pos.get(player)),
            format!("player_vel: {:?}", world.vel.get(player)),
            format!("player_acc: {:?}", world.acc.get(player)),
            String::from("----------------------------------------"),
            format!("square_1_id: {:?}", square_1),
            format!("square_1_rest: {:?}", world.rest.get(square_1)),
            format!("square_1_pos: {:?}", world.pos.get(square_1)),
            format!("square_1_vel: {:?}", world.vel.get(square_1)),
            format!("square_1_acc: {:?}", world.acc.get(square_1)),
            String::from("----------------------------------------"),
            format!("square_2_id: {:?}", square_2),
            format!("square_2_rest: {:?}", world.rest.get(square_2)),
            format!("square_2_pos: {:?}", world.pos.get(square_2)),
            format!("square_2_vel: {:?}", world.vel.get(square_2)),
            format!("square_2_acc: {:?}", world.acc.get(square_2)),
        ]);

        // debug::render_vector(
        //     world.pos.get(player).expect("missing pos"),
        //     world.vel.get(player).expect("missing vel").x * 20.0,
        //     world.vel.get(player).expect("missing vel").y * 20.0,
        //     &camera,
        //     prelude::GREEN,
        //     true,
        // );

        // debug::render_vector(
        //     world.pos.get(square_1).expect("missing pos"),
        //     world.vel.get(square_1).expect("missing vel").x * 20.0,
        //     world.vel.get(square_1).expect("missing vel").y * 20.0,
        //     &camera,
        //     prelude::PURPLE,
        //     true,
        // );

        // debug::render_vector(
        //     world.pos.get(square_2).expect("missing pos"),
        //     world.vel.get(square_2).expect("missing vel").x * 20.0,
        //     world.vel.get(square_2).expect("missing vel").y * 20.0,
        //     &camera,
        //     prelude::YELLOW,
        //     true,
        // );

        //td::thread::sleep(std::time::Duration::from_millis(250));
        prelude::next_frame().await;
    }
}
