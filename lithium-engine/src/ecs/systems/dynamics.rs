use crate::{
    core::world::World,
    ecs::{components, entities},
    math,
};

#[inline]
fn clamp_toward_zero(value: f32, limit: Option<f32>) -> f32 {
    match limit {
        Some(limit) if limit > 0.0 => value.min(limit),
        Some(limit) => value.max(limit),
        None => value,
    }
}

#[inline]
pub fn reset_rest(world: &mut World) {
    for (_, translation) in world.translation.iter_mut() {
        translation.rest = false;
    }
}

#[inline]
pub fn reset_pos(world: &mut World) {
    for (entity, transform) in world.transform.iter_mut() {
        if let Some(initial_transform) = world.initial_transform.get(entity) {
            *transform = initial_transform.clone();
        }
    }
}

#[inline]
pub fn reset_vel(world: &mut World, new_vel: math::Vec2) {
    for (_, translation) in world.translation.iter_mut() {
        translation.set_lin_vel(new_vel);
    }
}

#[inline]
pub fn reset_force(world: &mut World, new_force: math::Vec2) {
    for (_, translation) in world.translation.iter_mut() {
        translation.set_force(new_force.scale(translation.mass()));
    }
}

#[inline]
pub fn update_pos(world: &mut World) {
    for (entity, transform) in world.transform.iter_mut() {
        if let Some(components::Translation { lin_vel, .. }) = world.translation.get(entity) {
            transform.pos.add_mut(*lin_vel);
        }
    }
}

#[inline]
pub fn update_lin_vel(world: &mut World) {
    for (_, translation) in world.translation.iter_mut() {
        translation
            .lin_vel
            .add_mut(translation.force.scale(translation.inv_mass()));
    }
}

pub fn apply_axis_lin_vel(
    world: &mut World,
    entity: entities::Entity,
    new_lin_vel: f32,
    limit: Option<f32>,
    axis: math::Axis,
) -> Option<()> {
    let translation = world.translation.get_mut(entity)?;

    match axis {
        math::Axis::X => {
            translation.lin_vel.x = clamp_toward_zero(translation.lin_vel.x + new_lin_vel, limit);
        }
        math::Axis::Y => {
            translation.lin_vel.y = clamp_toward_zero(translation.lin_vel.y + new_lin_vel, limit);
        }
    }

    Some(())
}

pub fn apply_vel(world: &mut World, entity: entities::Entity, new_vel: math::Vec2, limit: Option<f32>) -> Option<()> {
    let translation = world.translation.get_mut(entity)?;

    translation.lin_vel.x = clamp_toward_zero(translation.lin_vel.x + new_vel.x, limit);
    translation.lin_vel.y = clamp_toward_zero(translation.lin_vel.y + new_vel.y, limit);

    Some(())
}

pub fn apply_axis_force(
    world: &mut World,
    entity: entities::Entity,
    new_force: f32,
    limit: Option<f32>,
    axis: math::Axis,
) -> Option<()> {
    let translation = world.translation.get_mut(entity)?;

    match axis {
        math::Axis::X => {
            translation.force.x = clamp_toward_zero(translation.force.x + new_force, limit);
        }
        math::Axis::Y => {
            translation.force.y = clamp_toward_zero(translation.force.y + new_force, limit);
        }
    }

    Some(())
}

pub fn apply_force(world: &mut World, entity: entities::Entity, new_force: math::Vec2) -> Option<()> {
    let translation = world.translation.get_mut(entity)?;

    translation.force.add_mut(new_force);

    Some(())
}
