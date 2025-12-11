use lithium_engine::{
    ecs::entities,
    prelude::{self, World},
};

use macroquad::prelude as mq_prelude;

use std::{collections::HashSet, fmt::Write};

const GRAVITY: prelude::Vec2 = prelude::Vec2 { x: 0.0, y: 0.25 };

// this is an example of how to define a custom component, how to add it to the world, how to access its SparseSet and how to attach it to an entity using the map file
//
// 1) first we define our component struct
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
// 6.2) create a match function for the loader with this signature
// fn match_engine<const N: usize>(
//     world: &mut World<N>,
//     comp: prelude::LoadableComponent,
// ) -> Result<(), prelude::EngineError> {
//     match comp.kind.as_str() {
//         "example_component" => {
//             let example_component = ExampleComponent::deserialize(comp.data).map_err(prelude::FileError::from)?;
//             world
//                 .user_mut()
//                 .get_mut::<ExampleComponent>(EXAMPLECOMPONENT)
//                 .unwrap()
//                 .insert(comp.entity, example_component.into())?;
//             Ok(())
//         }
//         other custom components if needed...
//         _ => Ok(()),
//     }
// }
//
// 6.3) pass the match function to the loader by changing this line
// - prelude::load("assets/map.yaml", world, entity_manager, None).unwrap()
// + prelude::load("assets/map.yaml", world, entity_manager, Some(match_engine)).unwrap()

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

fn load_map<const N: usize>(
    world: &mut World<N>,
    entity_manager: &mut entities::EntityManager,
) -> HashSet<entities::Entity> {
    prelude::load("assets/map.yaml", world, entity_manager, None).unwrap()
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
    let _map = load_map(&mut world, &mut entity_manager);

    // create player
    let player = 0;

    // create camera
    let mut camera = prelude::Camera::new(
        prelude::Vec2::new(0.0, -100.0),
        prelude::Rect::new(mq_prelude::screen_width(), mq_prelude::screen_height()).unwrap(),
    );

    // game loop
    loop {
        // empty frame
        mq_prelude::clear_background(mq_prelude::BLACK);

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
                let _map = load_map(&mut world, &mut entity_manager);
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
            prelude::resolve_collisions(&mut world, true, 7);
            prelude::update_pos(&mut world);

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
        _ = write!(
            msg,
            "player_transform: {}\n",
            world.engine().transform.get(player).expect("missing transform")
        );
        _ = write!(
            msg,
            "player_translation: {}\n",
            world.engine().translation.get(player).expect("missing translation")
        );
        _ = write!(
            msg,
            "player_rotation: {}\n",
            world.engine().rotation.get(player).expect("missing rotation")
        );
        _ = write!(
            msg,
            "player_rotation_matrix: {}\n",
            world
                .engine()
                .rotation_matrix
                .get(player)
                .expect("missing rotation_matrix")
        );
        _ = write!(
            msg,
            "player_surface: {}\n",
            world.engine().surface.get(player).expect("missing surface")
        );
        _ = write!(
            msg,
            "player_shape: {}\n",
            world.engine().shape.get(player).expect("missing shape")
        );
        _ = write!(
            msg,
            "player_material: {}\n",
            world.engine().material.get(player).expect("missing material")
        );

        mq_prelude::draw_multiline_text(&msg, 20.0, 25.0, 16.0, None, mq_prelude::WHITE);

        // prelude::render_vector(
        //     world.transform.get(player).expect("missing transform").pos(),
        //     world.translation.get(player).expect("missing translation").lin_vel(),
        //     Some(5.0),
        //     &camera,
        //     mq_prelude::RED,
        //     false,
        // );

        if !pause {
            prelude::swap_rotation_matrices(&mut world);
        }

        // std::thread::sleep(std::time::Duration::from_millis(300));
        mq_prelude::next_frame().await;
    }
}
