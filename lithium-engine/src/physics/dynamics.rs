use crate::{
    ecs::{components, entities, world},
    math,
};

// *** reset ***

#[inline]
pub fn reset_all_rest<const N: usize>(world: &mut world::World<N>) {
    for (_, translation) in world.engine.translation.iter_mut() {
        translation.rest = false;
    }
}

#[inline]
pub fn set_all_force<const N: usize>(world: &mut world::World<N>, force: math::Vec2) {
    for (_, translation) in world.engine.translation.iter_mut() {
        translation.force = force;
    }
}

#[inline]
pub fn set_all_lin_acc<const N: usize>(world: &mut world::World<N>, lin_acc: math::Vec2) {
    for (_, translation) in world.engine.translation.iter_mut() {
        translation.force = lin_acc.scale(translation.mass());
    }
}

#[inline]
pub fn set_all_torque<const N: usize>(world: &mut world::World<N>, torque: f32) {
    for (_, rotation) in world.engine.rotation.iter_mut() {
        rotation.torque = torque;
    }
}

#[inline]
pub fn set_all_ang_acc<const N: usize>(world: &mut world::World<N>, ang_acc: f32) {
    for (_, rotation) in world.engine.rotation.iter_mut() {
        rotation.torque = ang_acc * rotation.inertia();
    }
}

// *** integration ***

#[inline]
pub fn integrate_all_pos<const N: usize>(world: &mut world::World<N>) {
    for (entity, transform) in world.engine.transform.iter_mut() {
        if let Some(components::Translation { lin_vel, .. }) = world.engine.translation.get(entity) {
            transform.pos.add_mut(*lin_vel);
        }
    }
}

#[inline]
pub fn integrate_all_rot_mat<const N: usize>(world: &mut world::World<N>) {
    for (entity, rot_mat) in world.engine.rotation_matrix.iter_mut() {
        if let Some(components::Rotation { ang_vel, .. }) = world.engine.rotation.get(entity)
            && let Some(components::Body { centroid, .. }) = world.engine.body.get(entity)
        {
            _ = rot_mat.update_mut(math::Radians(*ang_vel), rot_mat.rot_mat.pre_mul_vec2(*centroid));
        }
    }
}

#[inline]
pub fn integrate_all_lin_vel<const N: usize>(world: &mut world::World<N>) {
    for (_, translation) in world.engine.translation.iter_mut() {
        translation.lin_vel.add_mut(translation.force.scale(translation.inv_mass()));
    }
}

#[inline]
pub fn integrate_all_ang_vel<const N: usize>(world: &mut world::World<N>) {
    for (_, rotation) in world.engine.rotation.iter_mut() {
        rotation.ang_vel += rotation.torque() * rotation.inv_inertia();
    }
}

// *** helpers wrappers ***

#[inline]
pub fn apply_lin_vel_axis<const N: usize>(
    world: &mut world::World<N>,
    entity: entities::Entity,
    lin_vel: f32,
    axis: math::Axis,
    limit: Option<f32>,
) -> Option<()> {
    world.engine.translation.get_mut(entity)?.apply_lin_vel_axis(lin_vel, axis, limit);

    Some(())
}

#[inline]
pub fn apply_lin_vel<const N: usize>(
    world: &mut world::World<N>,
    entity: entities::Entity,
    lin_vel: math::Vec2,
    limit: Option<f32>,
) -> Option<()> {
    world.engine.translation.get_mut(entity)?.apply_lin_vel(lin_vel, limit);

    Some(())
}

#[inline]
pub fn apply_force_axis<const N: usize>(
    world: &mut world::World<N>,
    entity: entities::Entity,
    force: f32,
    axis: math::Axis,
    limit: Option<f32>,
) -> Option<()> {
    world.engine.translation.get_mut(entity)?.apply_force_axis(force, axis, limit);

    Some(())
}

#[inline]
pub fn apply_force<const N: usize>(
    world: &mut world::World<N>,
    entity: entities::Entity,
    force: math::Vec2,
    limit: Option<f32>,
) -> Option<()> {
    world.engine.translation.get_mut(entity)?.apply_force(force, limit);

    Some(())
}

#[inline]
pub fn apply_ang_vel<const N: usize>(
    world: &mut world::World<N>,
    entity: entities::Entity,
    ang_vel: f32,
    limit: Option<f32>,
) -> Option<()> {
    world.engine.rotation.get_mut(entity)?.apply_ang_vel(ang_vel, limit);

    Some(())
}

#[inline]
pub fn apply_torque<const N: usize>(world: &mut world::World<N>, entity: entities::Entity, torque: f32, limit: Option<f32>) -> Option<()> {
    world.engine.rotation.get_mut(entity)?.apply_torque(torque, limit);

    Some(())
}

// *** extra helpers ***

#[inline]
pub fn apply_force_at_point_axis<const N: usize>(
    world: &mut world::World<N>,
    entity: entities::Entity,
    force: f32,
    axis: math::Axis,
    point: math::Vec2,
    lin_limit: Option<f32>,
    ang_limit: Option<f32>,
) -> Option<()> {
    let mass_center = world.engine.body.get(entity)?.centroid();
    let arm = point.sub(mass_center);

    apply_force_axis(world, entity, force, axis, lin_limit)?;

    let force_vector = match axis {
        math::Axis::X => math::Vec2::new(force, 0.0),
        math::Axis::Y => math::Vec2::new(0.0, force),
    };
    let torque = arm.cross(force_vector);
    let torque_2 = match axis {
        math::Axis::X => -arm.y * force,
        math::Axis::Y => arm.x * force,
    };
    assert_eq!(torque, torque_2);
    apply_torque(world, entity, torque, ang_limit)?;

    Some(())
}

#[inline]
pub fn apply_force_at_point<const N: usize>(
    world: &mut world::World<N>,
    entity: entities::Entity,
    force: math::Vec2,
    point: math::Vec2,
    lin_limit: Option<f32>,
    ang_limit: Option<f32>,
) -> Option<()> {
    let mass_center = world.engine.body.get(entity)?.centroid();
    let arm = point.sub(mass_center);

    apply_force(world, entity, force, lin_limit)?;

    let torque = arm.cross(force);
    apply_torque(world, entity, torque, ang_limit)?;

    Some(())
}
