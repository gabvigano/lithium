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
pub fn reset_acc(world: &mut World, new_acc: components::Vec2) {
    for (_, rigid_body) in world.rigid_body.iter_mut() {
        rigid_body.reset_acc(new_acc);
    }
}

#[inline]
pub fn update_pos(world: &mut World) {
    for (entity, transform) in world.transform.iter_mut() {
        if let Some(components::RigidBody { vel, .. }) = world.rigid_body.get(entity) {
            transform.pos.add_inplace(*vel);
        }
    }
}

#[inline]
pub fn update_vel(world: &mut World) {
    for (_, rigid_body) in world.rigid_body.iter_mut() {
        rigid_body.vel.add_inplace(rigid_body.acc);
    }
}

pub fn apply_vel(
    world: &mut World,
    entity: entities::Entity,
    new_vel: f32,
    limit: Option<f32>,
    axis: components::Axis,
) {
    match axis {
        components::Axis::Horizontal => {
            world
                .rigid_body
                .get_mut(entity)
                .expect("missing rigid_body")
                .vel
                .add_scalar_inplace(clamp_toward_zero(new_vel, limit), 0.0);
        }
        components::Axis::Vertical => {
            world
                .rigid_body
                .get_mut(entity)
                .expect("missing rigid_body")
                .vel
                .add_scalar_inplace(0.0, clamp_toward_zero(new_vel, limit));
        }
    }
}

pub fn apply_force(world: &mut World, entity: entities::Entity, new_force: components::Force) {
    let rigid_body = world.rigid_body.get_mut(entity).expect("missing rigid_body");

    match new_force.dir {
        components::Dir::Angle(components::Angle { radians }) => {
            rigid_body.acc.x += new_force.mag * radians.cos() / rigid_body.mass;
            rigid_body.acc.y += new_force.mag * radians.sin() / rigid_body.mass;
        }
        components::Dir::Axis(axis) => match axis {
            components::Axis::Horizontal => {
                rigid_body.acc.x += new_force.mag / rigid_body.mass;
            }
            components::Axis::Vertical => {
                rigid_body.acc.y += new_force.mag / rigid_body.mass;
            }
        },
    }
}
