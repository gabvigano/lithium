use crate::{
    ecs::{components, entities},
    world::World,
};

pub const EPS: f32 = 1e-6;
pub const EPS_SQR: f32 = EPS * EPS;

#[inline(always)]
pub fn pow2(x: f32) -> f32 {
    x * x
}

#[inline]
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

#[inline]
pub fn reset_rest(world: &mut World) {
    for (_, rigid_body) in world.rigid_body.iter_mut() {
        rigid_body.rest = false;
    }
}

#[inline]
pub fn reset_pos(world: &mut World) {
    for (_, transform) in world.transform.iter_mut() {
        transform.reset_pos();
    }
}

#[inline]
pub fn reset_vel(world: &mut World, new_vel: components::Vec2) {
    for (_, rigid_body) in world.rigid_body.iter_mut() {
        rigid_body.reset_vel(new_vel);
    }
}

#[inline]
pub fn reset_force(world: &mut World, new_force: components::Vec2) {
    for (_, rigid_body) in world.rigid_body.iter_mut() {
        rigid_body.reset_force(new_force.scale(rigid_body.mass()));
    }
}

#[inline]
pub fn update_pos(world: &mut World) {
    for (entity, transform) in world.transform.iter_mut() {
        if let Some(components::RigidBody { vel, .. }) = world.rigid_body.get(entity) {
            transform.pos.add_mut(*vel);
        }
    }
}

#[inline]
pub fn update_vel(world: &mut World) {
    for (_, rigid_body) in world.rigid_body.iter_mut() {
        rigid_body.vel.add_mut(rigid_body.force.scale(rigid_body.inv_mass()));
    }
}

pub fn apply_axis_vel(
    world: &mut World,
    entity: entities::Entity,
    new_vel: f32,
    limit: Option<f32>,
    axis: components::Axis,
) {
    let rigid_body = world.rigid_body.get_mut(entity).expect("missing rigid_body");

    match axis {
        components::Axis::Horizontal => {
            rigid_body.vel.x = clamp_toward_zero(rigid_body.vel.x + new_vel, limit);
        }
        components::Axis::Vertical => {
            rigid_body.vel.y = clamp_toward_zero(rigid_body.vel.y + new_vel, limit);
        }
    }
}

pub fn apply_vel(world: &mut World, entity: entities::Entity, new_vel: components::Vec2, limit: Option<f32>) {
    let rigid_body = world.rigid_body.get_mut(entity).expect("missing rigid_body");

    rigid_body.vel.x = clamp_toward_zero(rigid_body.vel.x + new_vel.x, limit);
    rigid_body.vel.y = clamp_toward_zero(rigid_body.vel.y + new_vel.y, limit);
}

pub fn apply_axis_force(
    world: &mut World,
    entity: entities::Entity,
    new_force: f32,
    limit: Option<f32>,
    axis: components::Axis,
) {
    let rigid_body = world.rigid_body.get_mut(entity).expect("missing rigid_body");

    match axis {
        components::Axis::Horizontal => {
            rigid_body.force.x = clamp_toward_zero(rigid_body.force.x + new_force, limit);
        }
        components::Axis::Vertical => {
            rigid_body.force.y = clamp_toward_zero(rigid_body.force.y + new_force, limit);
        }
    }
}

pub fn apply_force(world: &mut World, entity: entities::Entity, new_force: components::Vec2) {
    let rigid_body = world.rigid_body.get_mut(entity).expect("missing rigid_body");

    rigid_body.force.add_mut(new_force);
}
