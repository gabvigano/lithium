use lithium_engine::{
    debug,
    ecs::{
        components, entities,
        systems::{collision, physics},
    },
    loader, mq_adapter, scene,
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
    println!(
        "welcome to dropline!\nplease make sure you are running the game from lithium/dropline/ (current dir: {})",
        std::env::current_dir().unwrap().display()
    );

    // initialize environment
    let mut entity_manager = entities::EntityManager::new();
    let mut world = World::new();

    // load game map
    let static_map = loader::load_static_map("assets/map/static.ron", &mut world, &mut entity_manager);
    let dynamic_map = loader::load_dynamic_map("assets/map/dynamic.ron", &mut world, &mut entity_manager);

    // create player
    let player = entity_manager.create();
    world.start_pos.insert(player, components::Pos { x: 190.0, y: 100.0 });
    world
        .pos
        .insert(player, *world.start_pos.get(player).expect("missing start position"));
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

        // reset accelleration
        physics::reset_acc(&mut world, GRAVITY);

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
            physics::reset_pos(&mut world);
            physics::reset_vel(&mut world, components::Vel { x: 0.0, y: 0.0 });
            physics::reset_acc(&mut world, components::Acc { x: 0.0, y: 0.0 });
        }
        if prelude::is_key_down(prelude::KeyCode::P) {
            panic!("user panicked")
        }

        // update world and camera
        physics::update_vel(&mut world);
        let collisions = collision::compute_collision(&mut world);
        physics::reset_rest(&mut world); // rest is updated in physics::simulate_collisions()
        collision::simulate_collisions(&mut world, collisions);
        physics::update_pos(&mut world);

        camera.update(world.pos.get(player).expect("missing position"));

        // render entities
        for map_obj in &static_map {
            mq_adapter::render(&mut world, *map_obj, &camera);
        }

        for map_obj in &dynamic_map {
            mq_adapter::render(&mut world, *map_obj, &camera);
        }

        mq_adapter::render(&mut world, player, &camera);

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

        debug::display(&[
            format!("player_id: {:?}", player),
            format!("player_rest: {:?}", world.rest.get(player)),
            format!("player_pos: {:?}", world.pos.get(player)),
            format!("player_vel: {:?}", world.vel.get(player)),
            format!("player_acc: {:?}", world.acc.get(player)),
            // String::from("----------------------------------------"),
        ]);

        // debug::render_vector(
        //     world.pos.get(player).expect("missing pos"),
        //     world.vel.get(player).expect("missing vel").x * 20.0,
        //     world.vel.get(player).expect("missing vel").y * 20.0,
        //     &camera,
        //     prelude::GREEN,
        //     true,
        // );

        // std::thread::sleep(std::time::Duration::from_millis(250));
        prelude::next_frame().await;
    }
}
