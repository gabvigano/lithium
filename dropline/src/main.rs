use lithium_engine::prelude;

use std::fmt::Write;
use std::time::Instant;

const WIDTH: usize = 1600;
const HEIGHT: usize = 900;
const FPS: usize = 60;
const BG_COLOR: u32 = 0x000000;

const STEPS_PER_TICK: usize = 5;
const STEP: f32 = 1.0 / (STEPS_PER_TICK as f32);
const MAX_COLLISION_ITERATIONS: usize = 10;

const GRAVITY: prelude::Vec2 = prelude::Vec2 { x: 0.0, y: 0.3 };

// if you are on hyprland, to make the window float, add this to your hyprland config:
//
// hl.on("window.title", function(window)
//     if window ~= nil and window.title:match("^lithium%-engine:") then
//         hl.dispatch(hl.dsp.window.float({
//             action = "set",
//         }))

//         hl.dispatch(hl.dsp.window.center())
//     end
// end)
//
// this makes hyprland listen for window changes, and if the window's title matches "lithium-engine:" it automatically makes it float and centers it
// --------------------------------------------------------------------------------------------------------------------------------------------------------------------
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

fn main() {
    println!(
        "welcome to dropline!\nplease make sure you are running the game from lithium/dropline/ (current dir: {})",
        std::env::current_dir().unwrap().display()
    );

    // initialize window
    let mut window =
        prelude::minifb::Window::new("lithium-engine: dropline", WIDTH, HEIGHT, prelude::minifb::WindowOptions::default()).unwrap();
    let mut frame_buffer = prelude::FrameBuffer::new((WIDTH, HEIGHT), BG_COLOR);

    window.set_target_fps(FPS);

    // initialize environment
    let mut pause = false;
    let mut entity_manager = prelude::EntityManager::new();
    let mut world = prelude::World::default();

    // load game map
    let map_path = "assets/map.yaml";
    let mut map_cache = prelude::load(map_path, &mut world, &mut entity_manager, None).unwrap();

    // create player
    let player = 0;

    // create camera
    let mut camera = prelude::Camera::new(
        prelude::Vec2::new(0.0, -100.0),
        prelude::Rect::new_checked(WIDTH as f32, HEIGHT as f32).expect("error creating camera"),
    );

    let mut frame_start = Instant::now();
    let mut frame_idx = 1;
    let hot_reload_frames = 10;

    // game loop
    while window.is_open() && !window.is_key_down(prelude::minifb::Key::Escape) {
        // empty frame
        frame_buffer.clean_screen(BG_COLOR);

        // hot reload
        if frame_idx == hot_reload_frames {
            if let Err(err) = prelude::hot_reload(&mut map_cache, &mut world, &mut entity_manager, None, None) {
                println!("error hot reloading: {err}")
            }
            frame_idx = 1;
        } else {
            frame_idx += 1;
        }

        if !pause {
            // reset linear and angular acceleration
            prelude::set_all_lin_acc(&mut world, GRAVITY);
            prelude::set_all_ang_acc(&mut world, 0.0);

            // handle user inputs
            if window.is_key_down(prelude::minifb::Key::W) && world.engine().translation.get(player).unwrap().rest() {
                prelude::apply_lin_vel_axis(&mut world, player, -12.0, prelude::Axis::Y);
                prelude::clamp_min(&mut world.engine_mut().translation.get_mut(player).unwrap().lin_vel_mut().y, -12.0);
            }
            if window.is_key_down(prelude::minifb::Key::D) {
                prelude::apply_lin_vel_axis(&mut world, player, 1.0, prelude::Axis::X).unwrap();
                prelude::clamp_max(&mut world.engine_mut().translation.get_mut(player).unwrap().lin_vel_mut().x, 12.0);
            }
            if window.is_key_down(prelude::minifb::Key::A) {
                prelude::apply_lin_vel_axis(&mut world, player, -1.0, prelude::Axis::X).unwrap();
                prelude::clamp_min(&mut world.engine_mut().translation.get_mut(player).unwrap().lin_vel_mut().x, -12.0);
            }
            if window.is_key_pressed(prelude::minifb::Key::R, prelude::minifb::KeyRepeat::No) {
                // reset environment
                entity_manager.reset();
                world = prelude::World::default();

                // load game map
                map_cache = prelude::load(map_path, &mut world, &mut entity_manager, None).unwrap();
            }
        }
        if window.is_key_pressed(prelude::minifb::Key::P, prelude::minifb::KeyRepeat::No) {
            pause = !pause;
        }

        if !pause {
            // update world and camera
            for _ in 0..STEPS_PER_TICK {
                prelude::integrate_all_lin_vel(&mut world, STEP);
                prelude::integrate_all_ang_vel(&mut world, STEP);
                prelude::reset_all_rest(&mut world);
                prelude::resolve_collisions(&mut world, MAX_COLLISION_ITERATIONS, STEP).unwrap();
                prelude::integrate_all_pos(&mut world, STEP);
                prelude::integrate_all_rot_mat(&mut world, STEP);
                prelude::set_all_lin_acc(&mut world, GRAVITY);
                prelude::set_all_ang_acc(&mut world, 0.0);
            }

            camera.update(world.engine().transform.get(player).expect("missing transform").pos());
        }

        // render entities
        frame_buffer.rasterize_all(&world, &camera).unwrap();

        // render text
        let frame_end = Instant::now();
        let elapsed = frame_end - frame_start;

        let fps = 1.0 / (elapsed.as_nanos() as f64 / 1_000_000_000 as f64);
        let fps_text = format!("fps: {}\nspt: {}", fps as usize, STEPS_PER_TICK);

        frame_buffer.rasterize_text(&fps_text, (WIDTH - 140, 25), 2, 0xFFFFFF);
        frame_start = frame_end;

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

        frame_buffer.rasterize_text(&msg, (20, 25), 1, 0xFFFFFF);

        window.update_with_buffer(frame_buffer.buffer(), WIDTH, HEIGHT).unwrap();
    }
}
