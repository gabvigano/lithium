use crate::math::{ApplyTransformationShape, Centroid, SatCompatible, ToHitBox};
use crate::{base, ecs, math, physics};

use std::mem;

/// checks if 2 hitboxes are colliding using EPS to prevent false negatives
#[inline]
pub fn check_hitboxes(hitbox_1: &math::HitBox, hitbox_2: &math::HitBox) -> bool {
    !(hitbox_1.min_x > hitbox_2.max_x + math::EPS
        || hitbox_2.min_x > hitbox_1.max_x + math::EPS
        || hitbox_1.min_y > hitbox_2.max_y + math::EPS
        || hitbox_2.min_y > hitbox_1.max_y + math::EPS)
}

/// checks if 2 convex geometries are colliding using SAT algorithm, returns the contact overlap and normal
fn check_sat_cvx<T, U>(geometry_1: &T, geometry_2: &U) -> Option<(f32, math::Vec2)>
where
    T: SatCompatible + Centroid,
    U: SatCompatible + Centroid,
{
    fn check_axes<T, U>(
        sides: &[math::Vec2],
        geometry_1: &T,
        geometry_2: &U,
        delta: math::Vec2,
        best_overlap: &mut f32,
        best_normal: &mut math::Vec2,
    ) -> Option<()>
    where
        T: SatCompatible,
        U: SatCompatible,
    {
        for side in sides {
            let axis = side.perp_ccw().norm();

            let (min_1, max_1) = geometry_1.project(axis);
            let (min_2, max_2) = geometry_2.project(axis);

            if min_1 > max_2 + math::EPS || min_2 > max_1 + math::EPS {
                // not colliding
                return None;
            }

            let overlap = (max_1.min(max_2)) - (min_1.max(min_2));
            if overlap < *best_overlap {
                // update the normal data
                *best_overlap = overlap;
                *best_normal = if delta.dot(axis) < 0.0 { axis.rev() } else { axis }; // invert the normal direction if it is not from geometry_1 to geometry_2
            }
        }

        Some(())
    }

    // compute centroids
    let centroid_1 = geometry_1.centroid();
    let centroid_2 = geometry_2.centroid();
    let delta = centroid_2.sub(centroid_1); // points from geometry_1 to geometry_2

    // initialize return data
    let mut best_overlap = f32::INFINITY;
    let mut best_normal = math::Vec2::ZERO; // minimum translation vector axis, the axis of the smallest vector to push one shape out of the other

    // vector of sides
    let mut sides: Vec<math::Vec2> = Vec::with_capacity(geometry_1.sides_number() + geometry_2.sides_number());
    geometry_1.append_sides(&mut sides);

    check_axes(&sides, geometry_1, geometry_2, delta, &mut best_overlap, &mut best_normal)?;

    sides.clear();
    geometry_2.append_sides(&mut sides);

    check_axes(&sides, geometry_1, geometry_2, delta, &mut best_overlap, &mut best_normal)?;

    Some((best_overlap, best_normal))
}

pub struct SatCollision {
    pub overlap: f32,
    pub normal: math::Vec2,     // points from geometry_1 to geometry_2
    pub cave_part_idx_1: usize, // only used for CavePoly
    pub cave_part_idx_2: usize, // only used for CavePoly
}

/// checks if 2 geometries are colliding using SAT algorithm with check_sat_cvx()
pub fn check_sat<T, U>(geometry_1: &T, geometry_2: &U) -> Result<Option<SatCollision>, base::GeometryError>
where
    T: SatCompatible + Centroid,
    U: SatCompatible + Centroid,
{
    let cave_parts_1 = geometry_1.split_cave()?;
    let cave_parts_2 = geometry_2.split_cave()?;

    fn test_against_cave<V>(cvx_geometry: &V, cave_parts: &[math::CvxPoly]) -> Option<(f32, math::Vec2, usize)>
    where
        V: SatCompatible + Centroid,
    {
        let mut best_overlap = f32::INFINITY;
        let mut best_normal = math::Vec2::ZERO;
        let mut cave_part_idx = 0;
        let mut collided = false;

        for (idx, cvx_poly) in cave_parts.iter().enumerate() {
            let Some((overlap, normal)) = check_sat_cvx(cvx_geometry, cvx_poly) else {
                continue;
            };

            if overlap < best_overlap {
                best_overlap = overlap;
                best_normal = normal;
                cave_part_idx = idx;
                collided = true;
            }
        }

        if collided {
            Some((best_overlap, best_normal, cave_part_idx))
        } else {
            None
        }
    }

    // todo: here, for CavePoly we should only take results from sat if the edge tested is external in the original CavePoly

    Ok(match (cave_parts_1, cave_parts_2) {
        (None, None) => check_sat_cvx(geometry_1, geometry_2).map(|(overlap, normal)| SatCollision {
            overlap,
            normal,
            cave_part_idx_1: 0,
            cave_part_idx_2: 0,
        }),
        (None, Some(cvx_polys)) => test_against_cave(geometry_1, cvx_polys).map(|(overlap, normal, cave_part_idx)| SatCollision {
            overlap,
            normal,
            cave_part_idx_1: 0,
            cave_part_idx_2: cave_part_idx,
        }),
        (Some(cvx_polys), None) => test_against_cave(geometry_2, cvx_polys).map(|(overlap, normal, cave_part_idx)| SatCollision {
            overlap,
            normal: normal.rev(),
            cave_part_idx_1: cave_part_idx,
            cave_part_idx_2: 0,
        }),
        (Some(cvx_polys_1), Some(cvx_polys_2)) => {
            let mut best_overlap = f32::INFINITY;
            let mut best_normal = math::Vec2::ZERO;
            let mut cave_part_idx_1 = 0;
            let mut cave_part_idx_2 = 0;
            let mut collided = false;

            for (idx_1, cvx_poly_1) in cvx_polys_1.iter().enumerate() {
                for (idx_2, cvx_poly_2) in cvx_polys_2.iter().enumerate() {
                    let Some((overlap, normal)) = check_sat_cvx(cvx_poly_1, cvx_poly_2) else {
                        continue;
                    };

                    if overlap < best_overlap {
                        best_overlap = overlap;
                        best_normal = normal;
                        cave_part_idx_1 = idx_1;
                        cave_part_idx_2 = idx_2;
                        collided = true;
                    }
                }
            }

            if collided {
                Some(SatCollision {
                    overlap: best_overlap,
                    normal: best_normal,
                    cave_part_idx_1,
                    cave_part_idx_2,
                })
            } else {
                None
            }
        }
    })
}

pub fn compute_global_shape(
    state: State,
    mut pos: math::Vec2,
    rot_mat: Option<&ecs::RotationMatrix>,
    lin_vel: Option<math::Vec2>,
    ang_vel: Option<f32>,
    body: &ecs::Body,
    step: f32,
) -> math::Shape {
    let rot_mat = if matches!(state, State::Static | State::Still) {
        None
    } else {
        pos = match lin_vel {
            Some(lv) => pos.add(lv.scale(step)),
            None => pos,
        };

        match (rot_mat, ang_vel) {
            (Some(rm), Some(av)) => Some(&rm.update(math::Radians(av * step), rm.rot_mat.pre_mul_vec2(body.centroid))),
            (Some(rm), None) => Some(rm),
            (None, Some(_)) => panic!("ang_vel exists but rot_mat does not"), // <- thanks Lyla for fixing the error message
            (None, None) => None,
        }
    };

    match rot_mat {
        Some(ecs::RotationMatrix { rot_mat: rm }) => body.shape.apply_mat2x3_then_vec2_unchecked(pos, rm),
        None => body.shape.apply_vec2_unchecked(pos),
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub enum State {
    Active, // entity can collide
    Far,    // entity has not been involved in a collision for an entire iteration
    Still,  // entity has translation or rotation components, but they are currently zero
    Static, // entity does not have translation or rotation components
    Invalid,
}

/// detects collisions and computes reactions for every object
pub fn resolve_collisions<const N: usize>(world: &mut ecs::World<N>, iters: usize, step: f32) -> Result<(), base::GeometryError> {
    #[inline]
    fn get_state<const N: usize>(world: &ecs::World<N>, entity: ecs::Entity) -> State {
        let mut translation_is_zero = false; // true -> exists but is 0; false -> does not exist
        let mut rotation_is_zero = false;

        if let Some(&ecs::Translation { lin_vel, .. }) = world.engine.translation.get(entity) {
            // entity can move
            if lin_vel.approx_equal_zero() {
                // entity is not moving
                translation_is_zero = true;
            } else {
                // entity is moving
                return State::Active;
            }
        }

        if let Some(&ecs::Rotation { ang_vel, .. }) = world.engine.rotation.get(entity) {
            // entity can rotate
            if ang_vel.abs() < math::EPS {
                // entity is not rotating
                rotation_is_zero = true;
            } else {
                // entity is rotating
                return State::Active;
            }
        }

        if translation_is_zero || rotation_is_zero {
            // entity can move or rotate but it is still
            return State::Still;
        } else {
            // entity is static
            return State::Static;
        }
    }

    let ents = world.engine.transform.get_ents();
    let len = ents.len();
    let mut states: Vec<State> = Vec::with_capacity(len);
    let mut next_states = vec![State::Invalid; len];

    // println!("marking...");

    for &entity in ents.iter() {
        if let (Some(_), Some(_), Some(_)) = (
            world.engine.transform.get(entity),
            world.engine.surface.get(entity),
            world.engine.body.get(entity),
        ) {
            // entity is a valid object
            states.push(get_state(world, entity));
            // println!("entity {entity} marked as {:?}", states.last().unwrap());
        } else {
            // entity is not a valid object
            states.push(State::Invalid);
            // println!("entity {entity} marked as Invalid");
        }
    }

    for i in 0..iters {
        let mut solved = true;

        if i > 0 {
            mem::swap(&mut states, &mut next_states);
            next_states.fill(State::Invalid);

            // preserve far state for objects that are already far
            for idx in 0..len {
                if matches!(states[idx], State::Far) {
                    next_states[idx] = State::Far;
                }
            }
        }

        // println!("\niteration: {i}");
        // println!("\n{ents:?}");
        // println!("{states:?}");
        // println!("{next_states:?}");
        // println!("\nsolving...");

        'loop_1: for idx_1 in 0..len {
            let state_1 = states[idx_1];

            if !matches!(state_1, State::Active) {
                // entity is not active
                // println!("{}-* (1) skipped because {state_1:?}", ents[idx_1]);
                continue 'loop_1;
            }

            let entity_1 = ents[idx_1];
            let mut entity_1_is_far = true;

            let (pos_1, rot_mat_1, mut lin_vel_1, mut ang_vel_1, surface_1, body_1) = (
                world.engine.transform.get(entity_1).map(|t| t.pos), // extract pos
                world.engine.rotation_matrix.get(entity_1),
                world.engine.translation.get(entity_1).map(|t| t.lin_vel), // extract lin_vel
                world.engine.rotation.get(entity_1).map(|t| t.ang_vel),    // extract ang_vel
                world.engine.surface.get(entity_1),
                world.engine.body.get(entity_1),
            );

            let Some(pos_1) = pos_1 else {
                continue 'loop_1;
            };

            let Some(surface_1) = surface_1 else {
                continue 'loop_1;
            };

            let Some(body_1) = body_1 else {
                continue 'loop_1;
            };

            // initialize global shape and hitbox cache
            let mut global_shape_1 = None;
            let mut hitbox_1 = None;

            'loop_2: for idx_2 in 0..len {
                let state_2 = states[idx_2];

                if idx_1 == idx_2 || matches!(state_2, State::Invalid) {
                    // avoid self-check and skip invalid entities
                    // println!(
                    //     "{entity_1}-{} (2) skipped because same as entity_1 or {state_2:?}",
                    //     ents[idx_2]
                    // );
                    continue 'loop_2;
                }

                if matches!(state_2, State::Active) && idx_1 > idx_2 {
                    // both entities are active and it is under the diagonal of the matrix, so it has already been checked
                    // println!(
                    //     "{entity_1}-{} (2) skipped because already checked {entity_1}-{}",
                    //     ents[idx_2], ents[idx_2]
                    // );
                    continue 'loop_2;
                }

                let entity_2 = ents[idx_2];

                let (pos_2, rot_mat_2, lin_vel_2, ang_vel_2, surface_2, body_2) = (
                    world.engine.transform.get(entity_2).map(|t| t.pos), // extract pos
                    world.engine.rotation_matrix.get(entity_2),
                    world.engine.translation.get(entity_2).map(|t| t.lin_vel), // extract lin_vel
                    world.engine.rotation.get(entity_2).map(|t| t.ang_vel),    // extract ang_vel
                    world.engine.surface.get(entity_2),
                    world.engine.body.get(entity_2),
                );

                let Some(pos_2) = pos_2 else {
                    continue 'loop_2;
                };

                let Some(surface_2) = surface_2 else {
                    continue 'loop_2;
                };

                let Some(body_2) = body_2 else {
                    continue 'loop_2;
                };

                // compute global shapes
                if global_shape_1.is_none() {
                    // println!("recomputing global_shape_1 cache");
                    global_shape_1 = Some(compute_global_shape(state_1, pos_1, rot_mat_1, lin_vel_1, ang_vel_1, body_1, step));
                }

                let global_shape_2 = compute_global_shape(state_2, pos_2, rot_mat_2, lin_vel_2, ang_vel_2, body_2, step);

                // broad phase, compute hitbox
                if hitbox_1.is_none() {
                    // println!("recomputing hitbox_1 cache");
                    hitbox_1 = Some(global_shape_1.as_ref().unwrap().to_hitbox());
                }
                let hitbox_2 = global_shape_2.to_hitbox();

                // println!("{entity_1}-{entity_2} checking hitboxes...");
                if !check_hitboxes(hitbox_1.as_ref().unwrap(), &hitbox_2) {
                    // hitboxes are not colliding
                    // println!("  -> hitboxes NOT colliding");
                    continue 'loop_2;
                }
                // println!("  -> hitboxes ARE colliding");

                // narrow phase
                // println!("{entity_1}-{entity_2} checking global shapes...");
                let Some(SatCollision {
                    overlap: _,
                    normal,
                    cave_part_idx_1,
                    cave_part_idx_2,
                }) = check_sat(global_shape_1.as_ref().unwrap(), &global_shape_2)?
                else {
                    // global shapes are not colliding, no need to compute reaction or invalidate cache
                    // println!("  -> global shapes NOT colliding");
                    continue 'loop_2;
                };
                // println!("  -> global shapes ARE colliding");

                // compute centers of mass and contact point
                let mass_center_1 = global_shape_1.as_ref().unwrap().centroid();
                let mass_center_2 = global_shape_2.centroid();

                let contact_point = physics::compute_contact_point(
                    normal,
                    pos_1,
                    pos_2,
                    rot_mat_1,
                    rot_mat_2,
                    lin_vel_1,
                    lin_vel_2,
                    ang_vel_1,
                    ang_vel_2,
                    body_1,
                    body_2,
                    cave_part_idx_1,
                    cave_part_idx_2,
                    step,
                )?;

                // collision detected
                solved = false;
                entity_1_is_far = false;

                // invalidate cache since it will change with the reaction
                hitbox_1 = None;
                global_shape_1 = None;

                let (translation_1, translation_2) = world.engine.translation.get2_mut(entity_1, entity_2);
                let (rotation_1, rotation_2) = world.engine.rotation.get2_mut(entity_1, entity_2);

                // println!("{entity_1}-{entity_2} computing reaction");
                physics::compute_reaction(
                    normal,
                    contact_point,
                    mass_center_1,
                    mass_center_2,
                    translation_1,
                    translation_2,
                    rotation_1,
                    rotation_2,
                    surface_1,
                    surface_2,
                );

                // update lin_vel and ang_vel for entity_1 since they are cached for the duration of the inner loop
                // note how we can't just set them to None, since the solver would then treat them as non-existing
                // components, and it may even conflict with the states
                lin_vel_1 = world.engine.translation.get(entity_1).map(|t| t.lin_vel);
                ang_vel_1 = world.engine.rotation.get(entity_1).map(|t| t.ang_vel);

                // here active state means "the entity did collide"; later, it is rechecked to see if the actual state is different,
                // but we need this to:
                // - make sure a previously far entity that collided as entity_2 is not kept far
                // - make sure an entity that would be far otherwise (never collides as entity_1) is not set to far if it collides as entity_2
                next_states[idx_2] = State::Active;
            }

            // handle far state for entity_1
            if !matches!(next_states[idx_1], State::Active) && entity_1_is_far {
                // if it is active, it means it was in a collision as entity_2, so it can't be far
                next_states[idx_1] = State::Far;
            }
        }

        if solved {
            break;
        } else {
            // compute next_states
            for idx in 0..len {
                if matches!(states[idx], State::Static) {
                    // static entities always stay static
                    next_states[idx] = State::Static;
                    continue;
                }

                next_states[idx] = match (next_states[idx], get_state(world, ents[idx])) {
                    (State::Far, State::Active) => State::Far, // if it's far but it's not static or still, keep far
                    (_, state) => state,
                };
            }
        }
    }

    Ok(())
}
