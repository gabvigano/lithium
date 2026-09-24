use crate::math::geometry::ApplyTransformationShape;
use crate::{ecs, math};

// reset

#[inline]
pub fn reset_all_rest<const N: usize>(world: &mut ecs::World<N>) {
    for (_, translation) in world.engine.translation.iter_mut() {
        translation.rest = false;
    }
}

#[inline]
pub fn set_all_force<const N: usize>(world: &mut ecs::World<N>, force: math::Vec2) {
    for (_, translation) in world.engine.translation.iter_mut() {
        translation.force = force;
    }
}

#[inline]
pub fn set_all_lin_acc<const N: usize>(world: &mut ecs::World<N>, lin_acc: math::Vec2) {
    for (_, translation) in world.engine.translation.iter_mut() {
        translation.force = lin_acc.scale(translation.mass());
    }
}

#[inline]
pub fn set_all_torque<const N: usize>(world: &mut ecs::World<N>, torque: f32) {
    for (_, rotation) in world.engine.rotation.iter_mut() {
        rotation.torque = torque;
    }
}

#[inline]
pub fn set_all_ang_acc<const N: usize>(world: &mut ecs::World<N>, ang_acc: f32) {
    for (_, rotation) in world.engine.rotation.iter_mut() {
        rotation.torque = ang_acc * rotation.inertia();
    }
}

// integration

#[inline]
pub fn integrate_all_pos<const N: usize>(world: &mut ecs::World<N>, step: f32) {
    for (entity, transform) in world.engine.transform.iter_mut() {
        if let Some(ecs::Translation { lin_vel, .. }) = world.engine.translation.get(entity) {
            transform.pos.add_mut(lin_vel.scale(step));
        }
    }
}

#[inline]
pub fn integrate_all_rot_mat<const N: usize>(world: &mut ecs::World<N>, step: f32) {
    for (entity, rot_mat) in world.engine.rotation_matrix.iter_mut() {
        if let Some(ecs::Rotation { ang_vel, .. }) = world.engine.rotation.get(entity)
            && let Some(ecs::Body { centroid, .. }) = world.engine.body.get(entity)
        {
            _ = rot_mat.update_mut(math::Radians(ang_vel * step), rot_mat.rot_mat.pre_mul_vec2(*centroid));
        }
    }
}

#[inline]
pub fn integrate_all_lin_vel<const N: usize>(world: &mut ecs::World<N>, step: f32) {
    for (_, translation) in world.engine.translation.iter_mut() {
        translation.lin_vel.add_mut(translation.force.scale(translation.inv_mass() * step));
    }
}

#[inline]
pub fn integrate_all_ang_vel<const N: usize>(world: &mut ecs::World<N>, step: f32) {
    for (_, rotation) in world.engine.rotation.iter_mut() {
        rotation.ang_vel += rotation.torque() * rotation.inv_inertia() * step;
    }
}

// helpers wrappers

#[inline]
pub fn apply_lin_vel<const N: usize>(world: &mut ecs::World<N>, entity: ecs::Entity, lin_vel: math::Vec2) -> Option<()> {
    world.engine.translation.get_mut(entity)?.apply_lin_vel(lin_vel);

    Some(())
}

#[inline]
pub fn apply_lin_vel_axis<const N: usize>(world: &mut ecs::World<N>, entity: ecs::Entity, lin_vel: f32, axis: math::Axis) -> Option<()> {
    world.engine.translation.get_mut(entity)?.apply_lin_vel_axis(lin_vel, axis);

    Some(())
}

#[inline]
pub fn apply_force<const N: usize>(world: &mut ecs::World<N>, entity: ecs::Entity, force: math::Vec2) -> Option<()> {
    world.engine.translation.get_mut(entity)?.apply_force(force);

    Some(())
}

#[inline]
pub fn apply_force_axis<const N: usize>(world: &mut ecs::World<N>, entity: ecs::Entity, force: f32, axis: math::Axis) -> Option<()> {
    world.engine.translation.get_mut(entity)?.apply_force_axis(force, axis);

    Some(())
}

#[inline]
pub fn apply_ang_vel<const N: usize>(world: &mut ecs::World<N>, entity: ecs::Entity, ang_vel: f32) -> Option<()> {
    world.engine.rotation.get_mut(entity)?.apply_ang_vel(ang_vel);

    Some(())
}

#[inline]
pub fn apply_torque<const N: usize>(world: &mut ecs::World<N>, entity: ecs::Entity, torque: f32) -> Option<()> {
    world.engine.rotation.get_mut(entity)?.apply_torque(torque);

    Some(())
}

// extra helpers

#[inline]
pub fn local_point_to_world<const N: usize>(world: &ecs::World<N>, entity: ecs::Entity, local_point: math::Vec2) -> math::Vec2 {
    let mut world_point = match world.engine.rotation_matrix.get(entity) {
        Some(rotation_matrix) => rotation_matrix.rot_mat().pre_mul_vec2(local_point),
        None => local_point,
    };

    if let Some(transform) = world.engine.transform.get(entity) {
        world_point.add_mut(transform.pos());
    }

    world_point
}

#[inline]
pub fn apply_force_at_point<const N: usize>(
    world: &mut ecs::World<N>,
    entity: ecs::Entity,
    force: math::Vec2,
    point: math::Vec2,
) -> Option<()> {
    let local_mass_center = world.engine.body.get(entity)?.centroid();
    let mass_center = local_point_to_world(world, entity, local_mass_center);

    let arm = point.sub(mass_center);
    let torque = arm.cross(force);

    let translation = world.engine.translation.get_mut(entity)?;
    let rotation = world.engine.rotation.get_mut(entity)?;

    translation.apply_force(force);
    rotation.apply_torque(torque);

    Some(())
}

#[inline]
pub fn apply_force_at_point_axis<const N: usize>(
    world: &mut ecs::World<N>,
    entity: ecs::Entity,
    force: f32,
    axis: math::Axis,
    point: math::Vec2,
) -> Option<()> {
    let force_world = match axis {
        math::Axis::X => math::Vec2::new(force, 0.0),
        math::Axis::Y => math::Vec2::new(0.0, force),
    };

    apply_force_at_point(world, entity, force_world, point)
}

#[inline]
pub fn apply_force_at_local_point<const N: usize>(
    world: &mut ecs::World<N>,
    entity: ecs::Entity,
    force: math::Vec2,
    local_point: math::Vec2,
) -> Option<()> {
    let point = local_point_to_world(world, entity, local_point);

    apply_force_at_point(world, entity, force, point)
}

#[inline]
pub fn apply_force_at_local_point_axis<const N: usize>(
    world: &mut ecs::World<N>,
    entity: ecs::Entity,
    force: f32,
    axis: math::Axis,
    local_point: math::Vec2,
) -> Option<()> {
    let point = local_point_to_world(world, entity, local_point);

    apply_force_at_point_axis(world, entity, force, axis, point)
}

#[inline]
pub fn apply_all_trans<const N: usize>(world: &mut ecs::World<N>, entity: ecs::Entity) -> Option<math::Shape> {
    let shape = &world.engine.body.get(entity)?.shape;

    let transform = world.engine.transform.get(entity);
    let rotation_matrix = world.engine.rotation_matrix.get(entity);

    Some(match (transform, rotation_matrix) {
        (Some(transform), Some(rotation_matrix)) => shape.apply_mat2x3_then_vec2_unchecked(transform.pos, &rotation_matrix.rot_mat),
        (Some(transform), None) => shape.apply_vec2_unchecked(transform.pos),
        (None, Some(rotation)) => shape.apply_mat2x3_unchecked(&rotation.rot_mat),
        (None, None) => shape.clone(),
    })
}
