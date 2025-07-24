use crate::{
    ecs::{components, entities},
    world::World,
};

fn clamp_toward_zero(value: f32, limit: Option<f32>) -> f32 {
    if limit.is_none() {
        value
    } else {
        let limit = limit.unwrap();
        if limit > 0.0 {
            value.min(limit)
        } else {
            value.max(limit)
        }
    }
}

pub fn reset_rest(world: &mut World) {
    for (_, rest) in world.rest.iter_mut() {
        *rest = false;
    }
}

pub fn reset_pos(world: &mut World) {
    for (entity, start_pos) in world.start_pos.iter() {
        world.pos.set(entity, *start_pos);
    }
}

pub fn reset_vel(world: &mut World, new_vel: components::Vel) {
    for (_, vel) in world.vel.iter_mut() {
        *vel = new_vel;
    }
}

pub fn reset_acc(world: &mut World, new_acc: components::Acc) {
    for (_, acc) in world.acc.iter_mut() {
        *acc = new_acc;
    }
}

pub fn update_pos(world: &mut World) {
    for (entity, pos) in world.pos.iter_mut() {
        if let Some(vel) = world.vel.get(entity) {
            pos.x += vel.x;
            pos.y += vel.y;
        }
    }
}

pub fn update_vel(world: &mut World) {
    for (entity, vel) in world.vel.iter_mut() {
        if let Some(acc) = world.acc.get(entity) {
            vel.x += acc.x;
            vel.y += acc.y;
        }
    }
}

pub fn apply_vel(
    world: &mut World,
    entity: entities::Entity,
    new_vel: f32,
    limit: Option<f32>,
    axis: components::Axis,
) {
    let current = world.vel.get(entity).expect("missing velocity");

    match axis {
        components::Axis::Horizontal => {
            world.vel.set(
                entity,
                components::Vel {
                    x: clamp_toward_zero(current.x + new_vel, limit),
                    y: current.y,
                },
            );
        }
        components::Axis::Vertical => {
            world.vel.set(
                entity,
                components::Vel {
                    x: current.x,
                    y: clamp_toward_zero(current.y + new_vel, limit),
                },
            );
        }
    }
}

pub fn apply_force(world: &mut World, entity: entities::Entity, new_force: components::Force) {
    let mass = world.mass.get(entity).expect("missing mass").0;
    let acc = world.acc.get_mut(entity).expect("missing accelleration");

    match new_force.dir {
        components::Dir::Angle(components::Angle { radians }) => {
            acc.x += new_force.mag * radians.cos() / mass;
            acc.y += new_force.mag * radians.sin() / mass;
        }
        components::Dir::Axis(axis) => match axis {
            components::Axis::Horizontal => {
                acc.x += new_force.mag / mass;
            }
            components::Axis::Vertical => {
                acc.y += new_force.mag / mass;
            }
        },
    }
}
