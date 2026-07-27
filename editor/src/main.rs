use lithium_engine::{
    math::{ApplyTransformationShape, ToHitBox},
    prelude,
};

use macroquad::{
    input::{is_mouse_button_pressed, is_mouse_button_released},
    prelude as mq_prelude,
};

use std::{fmt::Write, fs, path::Path};

fn get_window_config() -> mq_prelude::Conf {
    mq_prelude::Conf {
        window_title: String::from("lithium-editor"),
        window_width: 1600,
        window_height: 900,
        window_resizable: true,
        ..Default::default()
    }
}

#[macroquad::main(get_window_config())]
async fn main() {
    const GRAVITY: prelude::Vec2 = prelude::Vec2 { x: 0.0, y: 0.2 };

    // initialize environment
    let mut entity_manager = prelude::EntityManager::new();
    let mut world = prelude::World::default();
    let mut simulate = false;

    // load assets

    fn load_assets(world: &mut prelude::World<0>, entity_manager: &mut prelude::EntityManager) -> Vec<prelude::AssetCache> {
        let assets_path = Path::new("assets");
        let mut hot_reload_caches: Vec<prelude::AssetCache> = Vec::new();

        for file in fs::read_dir(assets_path).unwrap() {
            let asset_path = file.unwrap().path();

            if asset_path.extension().is_some_and(|ext| ext == "yaml") {
                hot_reload_caches.push(prelude::load(asset_path.to_str().unwrap(), world, entity_manager, None).unwrap());
            }
        }

        hot_reload_caches
    }

    let mut hot_reload_caches = load_assets(&mut world, &mut entity_manager);

    // create camera
    let (screen_width, screen_height) = (mq_prelude::screen_width(), mq_prelude::screen_height());
    let mut camera = prelude::Camera::new(prelude::Vec2::zero(), prelude::Rect::new(screen_width, screen_height).unwrap());
    *camera.pos_mut() = prelude::Vec2::new(-screen_width / 2.0, -screen_height / 2.0);

    // create mouse pointer
    let mut pointer_pos = prelude::Vec2::zero();
    let mut pointer_rel_pos = prelude::Vec2::zero();
    let mut dragging_entity = None;

    loop {
        // empty frame
        mq_prelude::clear_background(mq_prelude::BLACK);

        // hot reload
        for cache in hot_reload_caches.iter_mut() {
            if let Err(err) = prelude::hot_reload(cache, &mut world, &mut entity_manager, None, None) {
                println!("error hot reloading: {err}")
            }
        }

        // reset force
        if simulate {
            prelude::set_all_force(&mut world, GRAVITY);
        }

        // get mouse pos
        (pointer_pos.x, pointer_pos.y) = mq_prelude::mouse_position();
        (pointer_pos.x, pointer_pos.y) = (pointer_pos.x + camera.pos().x, pointer_pos.y + camera.pos().y);

        // commands
        let delta_move =
            if mq_prelude::is_key_down(mq_prelude::KeyCode::LeftShift) || mq_prelude::is_key_down(mq_prelude::KeyCode::RightShift) {
                20.0
            } else {
                5.0
            };
        if mq_prelude::is_key_down(mq_prelude::KeyCode::Up) || mq_prelude::is_key_down(mq_prelude::KeyCode::W) {
            camera.pos_mut().y -= delta_move;
        }
        if mq_prelude::is_key_down(mq_prelude::KeyCode::Down) || mq_prelude::is_key_down(mq_prelude::KeyCode::S) {
            camera.pos_mut().y += delta_move;
        }
        if mq_prelude::is_key_down(mq_prelude::KeyCode::Right) || mq_prelude::is_key_down(mq_prelude::KeyCode::D) {
            camera.pos_mut().x += delta_move;
        }
        if mq_prelude::is_key_down(mq_prelude::KeyCode::Left) || mq_prelude::is_key_down(mq_prelude::KeyCode::A) {
            camera.pos_mut().x -= delta_move;
        }
        if mq_prelude::is_key_pressed(mq_prelude::KeyCode::R) {
            // reset environment
            entity_manager.reset();
            world = prelude::World::default();

            // load game map
            hot_reload_caches = load_assets(&mut world, &mut entity_manager);
        }
        if mq_prelude::is_key_pressed(mq_prelude::KeyCode::P) {
            simulate = !simulate;
        }
        if mq_prelude::is_key_pressed(mq_prelude::KeyCode::Escape) {
            panic!("user panicked")
        }
        if is_mouse_button_pressed(mq_prelude::MouseButton::Left) {
            // drag stuff
            let mats = world.engine.material.get_comps();
            let ents = world.engine.material.get_ents();
            let mut pairs: Vec<(&prelude::Material, &u32)> = mats.iter().zip(ents).collect();
            pairs.sort_by_key(|(m, _)| m.layer());
            for &(material, &entity) in pairs.iter().rev() {
                if !material.show() {
                    continue;
                }

                if let Some(transform) = world.engine.transform.get(entity)
                    && let Some(body) = world.engine.body.get(entity)
                {
                    let mut shape = body.shape().clone();
                    if let Some(rot_mat) = world.engine.rotation_matrix.get(entity) {
                        shape = shape.apply_mat2x3_checked(rot_mat.rot_mat()).unwrap();
                    }
                    shape = shape.apply_vec2_checked(transform.pos()).unwrap();
                    let hitbox = shape.to_hitbox();
                    if hitbox.min_x() <= pointer_pos.x && pointer_pos.x <= hitbox.max_x() {
                        if hitbox.min_y() <= pointer_pos.y && pointer_pos.y <= hitbox.max_y() {
                            dragging_entity = Some(entity);
                            pointer_rel_pos = transform.pos().sub(pointer_pos);
                            break;
                        }
                    }
                }
            }
        }
        if is_mouse_button_released(mq_prelude::MouseButton::Left) {
            dragging_entity = None;
        }

        if simulate {
            prelude::integrate_all_lin_vel(&mut world);
            prelude::integrate_all_ang_vel(&mut world);
            prelude::reset_all_rest(&mut world);
            prelude::resolve_collisions(&mut world, 10).unwrap();
            prelude::integrate_all_pos(&mut world);
            prelude::integrate_all_rot_mat(&mut world);
        }

        if let Some(entity) = dragging_entity {
            *world.engine_mut().transform.get_mut(entity).unwrap().pos_mut() = pointer_pos.add(pointer_rel_pos);
            if let Some(translation) = world.engine_mut().translation.get_mut(entity) {
                *translation.lin_vel_mut() = prelude::Vec2::zero();
            }
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
        _ = write!(msg, "{}\n", camera.pos());
        _ = write!(msg, "controls:\n");
        _ = write!(msg, "- wasd/arrows to move the camera (+shift to move it quicker)\n");
        _ = write!(msg, "- R to reset the simulation to its original state\n");
        _ = write!(msg, "- P to toggle physics (physics: {})\n", simulate);
        _ = write!(
            msg,
            "- drag entities with mouse (hold left button) (dragging entity: {:?})\n",
            dragging_entity
        );
        _ = write!(msg, "- Esc to quit\n");
        mq_prelude::draw_multiline_text(&msg, 20.0, 25.0, 16.0, None, mq_prelude::WHITE);

        mq_prelude::next_frame().await;
    }
}
