use crate::{
    ecs::{components, entities, world::World},
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
pub fn reset_rest<const N: usize>(world: &mut World<N>) {
    for (_, translation) in world.engine.translation.iter_mut() {
        translation.rest = false;
    }
}

#[inline]
pub fn reset_force<const N: usize>(world: &mut World<N>, new_force: math::Vec2) {
    for (_, translation) in world.engine.translation.iter_mut() {
        translation.set_force(new_force.scale(translation.mass()));
    }
}

#[inline]
pub fn update_pos<const N: usize>(world: &mut World<N>) {
    for (entity, transform) in world.engine.transform.iter_mut() {
        if let Some(components::Translation { lin_vel, .. }) = world.engine.translation.get(entity) {
            transform.pos.add_mut(*lin_vel);
        }
    }
}

#[inline]
pub fn update_lin_vel<const N: usize>(world: &mut World<N>) {
    for (_, translation) in world.engine.translation.iter_mut() {
        translation
            .lin_vel
            .add_mut(translation.force.scale(translation.inv_mass()));
    }
}

#[inline]
pub fn update_rot_mat<const N: usize>(world: &mut World<N>) {
    for (entity, rot_mat) in world.engine.rotation_matrix.iter_mut() {
        if let Some(components::Rotation { ang_vel, .. }) = world.engine.rotation.get(entity)
            && let Some(components::Body { centroid, .. }) = world.engine.body.get(entity)
        {
            _ = rot_mat.update_mut(math::Radians(*ang_vel), rot_mat.rot_mat.pre_mul_vec2(*centroid));
        }
    }
}

pub fn apply_axis_lin_vel<const N: usize>(
    world: &mut World<N>,
    entity: entities::Entity,
    new_lin_vel: f32,
    limit: Option<f32>,
    axis: math::Axis,
) -> Option<()> {
    let translation = world.engine.translation.get_mut(entity)?;

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

pub fn apply_vel<const N: usize>(
    world: &mut World<N>,
    entity: entities::Entity,
    new_vel: math::Vec2,
    limit: Option<f32>,
) -> Option<()> {
    let translation = world.engine.translation.get_mut(entity)?;

    translation.lin_vel.x = clamp_toward_zero(translation.lin_vel.x + new_vel.x, limit);
    translation.lin_vel.y = clamp_toward_zero(translation.lin_vel.y + new_vel.y, limit);

    Some(())
}

pub fn apply_axis_force<const N: usize>(
    world: &mut World<N>,
    entity: entities::Entity,
    new_force: f32,
    limit: Option<f32>,
    axis: math::Axis,
) -> Option<()> {
    let translation = world.engine.translation.get_mut(entity)?;

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

pub fn apply_force<const N: usize>(
    world: &mut World<N>,
    entity: entities::Entity,
    new_force: math::Vec2,
) -> Option<()> {
    let translation = world.engine.translation.get_mut(entity)?;

    translation.force.add_mut(new_force);

    Some(())
}

#[inline]
pub fn apply_rot<const N: usize>(
    world: &mut World<N>,
    entity: entities::Entity,
    delta_rot: math::Radians,
    pivot: math::Vec2,
) -> bool {
    let Some(rot_mat) = world.engine.rotation_matrix.get_mut(entity) else {
        return false;
    };

    if !rot_mat.update_mut(delta_rot, pivot) {
        return false;
    }

    true
}
