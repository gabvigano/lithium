use lithium_engine::{
    debug,
    ecs::{
        components::{self, ToHitBox},
        entities,
        systems::{collisions, physics},
    },
    loader, mq_adapter, scene,
    world::World,
};

use macroquad::prelude;

const GRAVITY: components::Vec2 = components::Vec2 { x: 0.0, y: 0.25 };

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
    let _static_map = loader::load_static_map("assets/map/static.ron", &mut world, &mut entity_manager);
    let _dynamic_map = loader::load_dynamic_map("assets/map/dynamic.ron", &mut world, &mut entity_manager);

    // create player
    let player = entity_manager.create();
    let player_spawn = components::Vec2::new(250.0, 400.0);
    world.transform.insert(
        player,
        components::Transform::new(player_spawn, player_spawn, components::Angle { radians: 0.0 }),
    );
    world.rigid_body.insert(
        player,
        components::RigidBody::new(
            components::Vec2::new(0.0, 0.0),
            components::Vec2::new(0.0, 0.0),
            1.0,
            false,
        ),
    );
    world.shape.insert(
        player,
        // components::Shape::Circle(components::Circle::new(10.0)),
        components::Shape::Rect(components::Rect::new(15.0, 15.0)),
    );
    world.collider.insert(
        player,
        components::Collider::new(world.shape.get(player).expect("missing shape").hitbox(), 0.5),
    );
    world.material.insert(
        player,
        components::Material::new(components::Color::new(0, 255, 0, 255), 2, true),
    );

    // create camera
    let mut camera = scene::Camera::new(
        components::Vec2::new(0.0, -100.0),
        components::Rect::new(prelude::screen_width(), prelude::screen_height()),
    );

    // game loop
    loop {
        // empty frame
        prelude::clear_background(prelude::BLACK);

        // reset accelleration
        physics::reset_acc(&mut world, GRAVITY);

        // handle user inputs
        if prelude::is_key_down(prelude::KeyCode::W) && world.rigid_body.get(player).expect("missing rigid_body").rest {
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
            physics::reset_vel(&mut world, components::Vec2::new(0.0, 0.0));
            physics::reset_acc(&mut world, components::Vec2::new(0.0, 0.0));
        }
        if prelude::is_key_down(prelude::KeyCode::P) {
            panic!("user panicked")
        }

        // update world and camera
        physics::update_vel(&mut world);
        physics::reset_rest(&mut world);
        collisions::resolve_collisions(&mut world, true);
        physics::update_pos(&mut world);

        camera.update(world.transform.get(player).expect("missing transform").pos);

        // render entities
        mq_adapter::render(&mut world, &camera);

        // render text
        prelude::draw_text(
            &format!("FPS: {}", prelude::get_fps()),
            prelude::screen_width() - 100.0,
            25.0,
            25.0,
            prelude::WHITE,
        );

        debug::display(&[
            format!("player_id: {:?}", player),
            format!("player_transform: {:?}", world.transform.get(player)),
            format!("player_rigid_body: {:?}", world.rigid_body.get(player)),
            // String::from("----------------------------------------"),
        ]);

        // std::thread::sleep(std::time::Duration::from_millis(300));
        prelude::next_frame().await;
    }
}
