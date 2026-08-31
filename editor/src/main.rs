use lithium_engine::{
    math::{ApplyTransformationShape, ToHitBox},
    prelude,
};

use std::fmt::Write;
use std::fs;
use std::path::Path;
use std::time::Instant;

const WIDTH: usize = 1600;
const HEIGHT: usize = 900;
const FPS: usize = 60;
const BG_COLOR: u32 = 0x000000;

const STEPS_PER_TICK: usize = 15;
const STEP: f32 = 1.0 / (STEPS_PER_TICK as f32);
const MAX_COLLISION_ITERATIONS: usize = 10;

const GRAVITY: prelude::Vec2 = prelude::Vec2 { x: 0.0, y: 0.3 };

fn main() {
    // initialize window
    let mut window =
        prelude::minifb::Window::new("lithium-engine: editor", WIDTH, HEIGHT, prelude::minifb::WindowOptions::default()).unwrap();
    let mut frame_buffer = prelude::FrameBuffer::new((WIDTH, HEIGHT), BG_COLOR);

    window.set_target_fps(FPS);

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
    let mut camera = prelude::Camera::new(
        prelude::Vec2::ZERO,
        prelude::Rect::new_checked(WIDTH as f32, HEIGHT as f32).unwrap(),
    );
    *camera.pos_mut() = prelude::Vec2::new(-(WIDTH as f32) / 2.0, -(HEIGHT as f32) / 2.0);

    let mut frame_start = Instant::now();
    let mut frame_idx = 1;
    let hot_reload_frames = 10;

    // create mouse pointer
    let mut prev_pointer_down = false;
    let mut pointer_pos = prelude::Vec2::ZERO;
    let mut pointer_rel_pos = prelude::Vec2::ZERO;
    let mut dragging_entity = None;

    while window.is_open() && !window.is_key_down(prelude::minifb::Key::Escape) {
        // empty frame
        frame_buffer.clean_screen(BG_COLOR);

        // hot reload
        if frame_idx == hot_reload_frames {
            for cache in hot_reload_caches.iter_mut() {
                if let Err(err) = prelude::hot_reload(cache, &mut world, &mut entity_manager, None, None) {
                    println!("error hot reloading: {err}")
                }
            }
            frame_idx = 1;
        } else {
            frame_idx += 1;
        }

        // reset force
        if simulate {
            prelude::set_all_lin_acc(&mut world, GRAVITY);
            prelude::set_all_ang_acc(&mut world, 0.0);
        }

        // get mouse pos
        (pointer_pos.x, pointer_pos.y) = window.get_mouse_pos(prelude::minifb::MouseMode::Clamp).unwrap();
        (pointer_pos.x, pointer_pos.y) = (pointer_pos.x + camera.pos().x, pointer_pos.y + camera.pos().y);

        // commands
        let delta_move = if window.is_key_down(prelude::minifb::Key::LeftShift) || window.is_key_down(prelude::minifb::Key::RightShift) {
            20.0
        } else {
            5.0
        };
        if window.is_key_down(prelude::minifb::Key::Up) || window.is_key_down(prelude::minifb::Key::W) {
            camera.pos_mut().y -= delta_move;
        }
        if window.is_key_down(prelude::minifb::Key::Down) || window.is_key_down(prelude::minifb::Key::S) {
            camera.pos_mut().y += delta_move;
        }
        if window.is_key_down(prelude::minifb::Key::Right) || window.is_key_down(prelude::minifb::Key::D) {
            camera.pos_mut().x += delta_move;
        }
        if window.is_key_down(prelude::minifb::Key::Left) || window.is_key_down(prelude::minifb::Key::A) {
            camera.pos_mut().x -= delta_move;
        }
        if window.is_key_pressed(prelude::minifb::Key::R, prelude::minifb::KeyRepeat::No) {
            // reset environment
            entity_manager.reset();
            world = prelude::World::default();

            // load game map
            hot_reload_caches = load_assets(&mut world, &mut entity_manager);
        }
        if window.is_key_pressed(prelude::minifb::Key::P, prelude::minifb::KeyRepeat::No) {
            simulate = !simulate;
        }

        let pointer_down = window.get_mouse_down(prelude::minifb::MouseButton::Left);
        let pointer_pressed = pointer_down && !prev_pointer_down;
        let pointer_released = !pointer_down && prev_pointer_down;
        prev_pointer_down = pointer_down;

        if pointer_pressed {
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
        if pointer_released {
            dragging_entity = None;
        }

        if simulate {
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
        }

        if let Some(entity) = dragging_entity {
            *world.engine_mut().transform.get_mut(entity).unwrap().pos_mut() = pointer_pos.add(pointer_rel_pos);
            if let Some(translation) = world.engine_mut().translation.get_mut(entity) {
                *translation.lin_vel_mut() = prelude::Vec2::ZERO;
            }
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

        frame_buffer.rasterize_text(&msg, (20, 25), 2, 0xFFFFFF);

        window.update_with_buffer(frame_buffer.buffer(), WIDTH, HEIGHT).unwrap();
    }
}
