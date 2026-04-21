use lithium_engine::prelude;

use macroquad::prelude as mq_prelude;

use std::fmt::Write;

const GRAVITY: prelude::Vec2 = prelude::Vec2 { x: 0.0, y: 0.3 };

// this is an example of how to define a custom component, how to add it to the world, how to access its SparseSet and how to attach it to an entity using the map file
//
// 1) first we define our component struct (careful not to use an already existing component name, otherwise the loader overwrite the existing component)
// use serde::Deserialize;
//
// const EXAMPLECOMPONENT: usize = 0; // this is not strictly necessary, but it is much easier than remembering the id of every custom component
//
// #[derive(Deserialize, Debug)]
// pub struct ExampleComponent {
//     pub field_a: f32,
//     pub field_b: u8,
//     pub field_c: bool,
// }
//
// 2) we mark our component as an UserComponent
// use std::any::Any;
//
// impl prelude::UserComponent for ExampleComponent {
//     fn as_any(&self) -> &dyn Any {
//         self
//     }

//     fn as_any_mut(&mut self) -> &mut dyn Any {
//         self
//     }
// }
//
// 4) now we can add it to the world by changing this line
// - prelude::World::default()
// + prelude::World::new([Box::new(prelude::SparseSet::<ExampleComponent>::new()), other custom components if needed...])
//
// 5) now, to access a SparseSet via code it is pretty similar to accessing an engine component
// world.engine().component // <- engine component
// world.user().get(COMPONENT_ID).unwrap() // <- user component
//
// if you need mutable you need to change the following
// engine() -> engine_mut()
// user() -> user_mut()
// get() -> get_mut()
//
// 6) to attach this component to an entity using the map file, we need to
// 6.1) edit the map file by adding this
// - entity: 0
//   kind: example_component
//   data: { field_a: 3.5, field_b: 4 , field_c: false}
//
// note that "example_component" only needs to be the same here and in step 6.2
//
// 6.2) create a match_user_upsert function and a match_user_remove function for the loader with these signatures
// fn match_user_upsert<const N: usize>(
//     world: &mut World<N>,
//     entity: prelude::Entity,
//     kind: &str,
//     data: serde_yaml::Value,
// ) -> Result<(), prelude::EngineError> {
//     match kind {
//         "example_component" => {
//             let example_component = ExampleComponent::deserialize(data).map_err(prelude::FileError::from)?;
//             world
//                 .user_mut()
//                 .get_mut::<ExampleComponent>(EXAMPLECOMPONENT)
//                 .unwrap()
//                 .insert(entity, example_component.into())?;
//             Ok(())
//         }
//         other custom components if needed...
//         _ => Ok(()),
//     }
// }
//
// fn match_user_remove<const N: usize>(world: &mut World<N>, entity: prelude::Entity, kind: &str) {
// match kind {
//     "example_component" => {
//         world
//              .user_mut()
//              .get_mut::<ExampleComponent>(EXAMPLECOMPONENT)
//              .unwrap()
//              .remove(entity);
//     }
//
// 6.3) pass the match functions to the loader by changing this line
// - let mut map_cache = prelude::load(map_path, &mut world, &mut entity_manager, None).unwrap();
// + let mut map_cache = prelude::load(map_path, &mut world, &mut entity_manager, Some(match_user_upsert)).unwrap();
//
// - prelude::new_loader::hot_reload(&mut map_cache, &mut world, &mut entity_manager, None, None)
// + prelude::new_loader::hot_reload(&mut map_cache, &mut world, &mut entity_manager, Some(match_user_upsert), Some(match_user_remove))

fn get_window_config() -> mq_prelude::Conf {
    mq_prelude::Conf {
        window_title: String::from("dropline"),
        window_width: 1600,
        window_height: 900,
        window_resizable: false,
        ..Default::default()
    }
}

fn init_world() -> prelude::World<0> {
    prelude::World::default()
}

#[macroquad::main(get_window_config())]
async fn main() {
    println!(
        "welcome to dropline!\nplease make sure you are running the game from lithium/dropline/ (current dir: {})",
        std::env::current_dir().unwrap().display()
    );

    // initialize environment
    let mut pause = false;
    let mut entity_manager = prelude::EntityManager::new();
    let mut world = init_world();

    // load game map
    let map_path = "assets/map.yaml";
    let mut map_cache = prelude::load(map_path, &mut world, &mut entity_manager, None).unwrap();

    // create player
    let player = 0;

    // create camera
    let mut camera = prelude::Camera::new(
        prelude::Vec2::new(0.0, -100.0),
        prelude::Rect::new(mq_prelude::screen_width(), mq_prelude::screen_height()).expect("error creating camera"),
    );

    let mut frame_idx = 1;
    let hot_reload_frames = 10;

    // game loop
    loop {
        // empty frame
        mq_prelude::clear_background(mq_prelude::BLACK);

        // reload
        if frame_idx == hot_reload_frames {
            if let Err(err) = prelude::hot_reload(&mut map_cache, &mut world, &mut entity_manager, None, None) {
                println!("error hot reloading: {err}")
            }
            frame_idx = 1;
        } else {
            frame_idx += 1;
        }

        if !pause {
            // reset force
            prelude::reset_force(&mut world, GRAVITY);

            // handle user inputs
            if mq_prelude::is_key_down(mq_prelude::KeyCode::W) && world.engine().translation.get(player).unwrap().rest()
            {
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
                // reset environment
                entity_manager.reset();
                world = init_world();

                // load game map
                map_cache = prelude::load(map_path, &mut world, &mut entity_manager, None).unwrap();
            }
        }
        if mq_prelude::is_key_down(mq_prelude::KeyCode::P) {
            panic!("user panicked")
        }
        if mq_prelude::is_key_down(mq_prelude::KeyCode::I) {
            pause = false;
        }
        if mq_prelude::is_key_down(mq_prelude::KeyCode::O) {
            pause = true;
        }

        if !pause {
            // update world and camera
            prelude::update_lin_vel(&mut world);
            prelude::reset_rest(&mut world);
            prelude::resolve_collisions(&mut world, 10);
            prelude::update_pos(&mut world);
            prelude::update_rot_mat(&mut world);

            camera.update(world.engine().transform.get(player).expect("missing transform").pos());
        }

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

        let mut msg = String::new();
        _ = write!(msg, "pause: {}\n\n", pause);
        _ = write!(msg, "player_id: {}\n", player);
        match world.engine().transform.get(player) {
            Some(component) => _ = write!(msg, "player_transform: {}\n", component),
            None => (),
        }
        match world.engine().translation.get(player) {
            Some(component) => _ = write!(msg, "player_translation: {}\n", component),
            None => (),
        }
        match world.engine().rotation.get(player) {
            Some(component) => _ = write!(msg, "player_rotation: {}\n", component),
            None => (),
        }
        match world.engine().rotation_matrix.get(player) {
            Some(component) => _ = write!(msg, "player_rotation_matrix: {}\n", component),
            None => (),
        }
        match world.engine().surface.get(player) {
            Some(component) => _ = write!(msg, "player_surface: {}\n", component),
            None => (),
        }
        match world.engine().body.get(player) {
            Some(component) => _ = write!(msg, "player_body: {}\n", component),
            None => (),
        }
        match world.engine().material.get(player) {
            Some(component) => _ = write!(msg, "player_material: {}\n", component),
            None => (),
        }

        mq_prelude::draw_multiline_text(&msg, 20.0, 25.0, 16.0, None, mq_prelude::WHITE);

        // prelude::render_vector(
        //     world.transform.get(player).expect("missing transform").pos(),
        //     world.translation.get(player).expect("missing translation").lin_vel(),
        //     Some(5.0),
        //     &camera,
        //     mq_prelude::RED,
        //     false,
        // );

        // std::thread::sleep(std::time::Duration::from_millis(50));
        // println!("\n\nFRAME ENDED, PRESS ENTER TO CONTINUE\n\n");
        // std::io::stdin().read_line(&mut String::new()).expect("failed to read");
        mq_prelude::next_frame().await;
    }
}
