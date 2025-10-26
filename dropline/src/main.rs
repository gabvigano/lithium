use lithium_engine::prelude;

use macroquad::prelude as mq_prelude;

const GRAVITY: prelude::Vec2 = prelude::Vec2 { x: 0.0, y: 0.25 };

fn get_window_config() -> mq_prelude::Conf {
    mq_prelude::Conf {
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
    let mut entity_manager = prelude::EntityManager::new();
    let mut world = prelude::World::new();

    // load game map
    let _static_map = prelude::load_static_map("assets/map/static.ron", &mut world, &mut entity_manager).unwrap();
    let _dynamic_map = prelude::load_dynamic_map("assets/map/dynamic.ron", &mut world, &mut entity_manager).unwrap();

    // create player
    let player = entity_manager.create();
    let player_initial_transform =
        prelude::Transform::new(prelude::Vec2::new(625.0, 100.0), prelude::Vec2::new(1.0, 0.0));
    world
        .initial_transform
        .insert(player, player_initial_transform.clone())
        .unwrap();
    world.transform.insert(player, player_initial_transform).unwrap();
    world
        .translation
        .insert(
            player,
            prelude::Translation::new(prelude::Vec2::new(0.0, 0.0), prelude::Vec2::new(0.0, 0.0), 1.0).unwrap(),
        )
        .unwrap();
    world
        .rotation
        .insert(player, prelude::Rotation::new(0.0, 0.0, 1.0).unwrap())
        .unwrap();
    world
        .surface
        .insert(player, prelude::Surface::new(0.5, 0.2, 0.15))
        .unwrap();
    world
        .shape
        .insert(player, prelude::Shape::Rect(prelude::Rect::new(15.0, 15.0).unwrap()))
        .unwrap();
    world
        .material
        .insert(
            player,
            prelude::Material::new(prelude::Color::new(0, 255, 0, 255), 2, true),
        )
        .unwrap();

    // create camera
    let mut camera = prelude::Camera::new(
        prelude::Vec2::new(0.0, -100.0),
        prelude::Rect::new(mq_prelude::screen_width(), mq_prelude::screen_height()).unwrap(),
    );

    // game loop
    loop {
        // empty frame
        mq_prelude::clear_background(mq_prelude::BLACK);

        // reset force
        prelude::reset_force(&mut world, GRAVITY);

        // handle user inputs
        if mq_prelude::is_key_down(mq_prelude::KeyCode::W) && world.translation.get(player).unwrap().rest() {
            prelude::apply_axis_lin_vel(&mut world, player, -12.0, Some(-12.0), prelude::Axis::Y).unwrap();
            // prelude::apply_axis_force(&mut world, player, -5.0, None, prelude::Axis::Y);
        }
        // let lin_vel_x = world.translation.get(player).expect("missing translation").lin_vel().x;
        if mq_prelude::is_key_down(mq_prelude::KeyCode::D)
        /*&& lin_vel_x < 7.0*/
        {
            prelude::apply_axis_lin_vel(&mut world, player, 1.0, Some(10.0), prelude::Axis::X).unwrap();
            // prelude::apply_axis_force(&mut world, player, 2.0, None, prelude::Axis::X);
        }
        if mq_prelude::is_key_down(mq_prelude::KeyCode::A)
        /*&& lin_vel_x > -7.0*/
        {
            prelude::apply_axis_lin_vel(&mut world, player, -1.0, Some(-10.0), prelude::Axis::X).unwrap();
            // prelude::apply_axis_force(&mut world, player, -2.0, None, prelude::Axis::X);
        }
        if mq_prelude::is_key_down(mq_prelude::KeyCode::R) {
            prelude::reset_pos(&mut world);
            prelude::reset_vel(&mut world, prelude::Vec2::new(0.0, 0.0));
            prelude::reset_force(&mut world, prelude::Vec2::new(0.0, 0.0));
        }
        if mq_prelude::is_key_down(mq_prelude::KeyCode::P) {
            panic!("user panicked")
        }

        // update world and camera
        prelude::update_lin_vel(&mut world);
        prelude::reset_rest(&mut world);
        prelude::resolve_collisions(&mut world, true, 5);
        prelude::update_pos(&mut world);

        camera.update(world.transform.get(player).expect("missing transform").pos());

        // render entities
        prelude::render(&mut world, &camera);

        // render text
        mq_prelude::draw_text(
            &format!("FPS: {}", mq_prelude::get_fps()),
            mq_prelude::screen_width() - 70.0,
            25.0,
            16.0,
            mq_prelude::WHITE,
        );

        prelude::display(&[
            format!("player_id: {}", player),
            format!(
                "player_initial_transform: {}",
                world.initial_transform.get(player).expect("missing initial_transform")
            ),
            format!(
                "player_transform: {}",
                world.transform.get(player).expect("missing transform")
            ),
            format!(
                "player_translation: {}",
                world.translation.get(player).expect("missing translation")
            ),
            format!(
                "player_rotation: {}",
                world.rotation.get(player).expect("missing rotation")
            ),
            format!(
                "player_surface: {}",
                world.surface.get(player).expect("missing surface")
            ),
            format!("player_shape: {}", world.shape.get(player).expect("missing shape")),
            format!(
                "player_material: {}",
                world.material.get(player).expect("missing material")
            ),
            // String::from("----------------------------------------"),
        ]);

        // prelude::render_vector(
        //     world.transform.get(player).expect("missing transform").pos(),
        //     world.translation.get(player).expect("missing translation").lin_vel(),
        //     Some(5.0),
        //     &camera,
        //     mq_prelude::RED,
        //     false,
        // );

        // std::thread::sleep(std::time::Duration::from_millis(300));
        mq_prelude::next_frame().await;
    }
}
