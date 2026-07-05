use crate::{ecs, math, physics};

use std::mem;

/// updates 2 entities' linear velocity vector after they collide
fn compute_reaction(
    normal: math::Vec2,
    contact_point: math::Vec2,
    mass_center_1: math::Vec2,
    mass_center_2: math::Vec2,
    mut translation_1: Option<&mut ecs::Translation>,
    mut translation_2: Option<&mut ecs::Translation>,
    mut rotation_1: Option<&mut ecs::Rotation>,
    mut rotation_2: Option<&mut ecs::Rotation>,
    surface_1: &ecs::Surface,
    surface_2: &ecs::Surface,
) {
    let compute_translation_reaction = translation_1.is_some() || translation_2.is_some();
    let compute_rotation_reaction = rotation_1.is_some() || rotation_2.is_some();

    // update rest
    if normal.x.abs() <= 0.5 {
        // one is above the other
        if normal.y > 0.0
            && let Some(translation_1) = translation_1.as_deref_mut()
        {
            translation_1.rest = true;
        }

        if normal.y < 0.0
            && let Some(translation_1) = translation_2.as_deref_mut()
        {
            translation_1.rest = true;
        }
    }

    // compute elast and friction
    let elast = surface_1.elast.min(surface_2.elast);
    let static_friction = (surface_1.static_friction * surface_2.static_friction).sqrt();
    let kinetic_friction = (surface_1.kinetic_friction * surface_2.kinetic_friction).sqrt();

    if compute_translation_reaction || compute_rotation_reaction {
        // extract lin_vel and inv_mass
        let (lin_vel_1, inv_mass_1) = {
            if let Some(translation_1) = translation_1.as_deref() {
                (translation_1.lin_vel, translation_1.inv_mass())
            } else {
                (math::Vec2::new(0.0, 0.0), 0.0)
            }
        };

        let (lin_vel_2, inv_mass_2) = {
            if let Some(translation_2) = translation_2.as_deref() {
                (translation_2.lin_vel, translation_2.inv_mass())
            } else {
                (math::Vec2::new(0.0, 0.0), 0.0)
            }
        };

        // compute lever arms
        let arm_1 = contact_point.sub(mass_center_1);
        let arm_2 = contact_point.sub(mass_center_2);

        // extract ang_vel and inv_inertia
        let (ang_vel_1, inv_inertia_1) = {
            if let Some(rotation_1) = rotation_1.as_deref() {
                (rotation_1.ang_vel, rotation_1.inv_inertia())
            } else {
                (0.0, 0.0)
            }
        };

        let (ang_vel_2, inv_inertia_2) = {
            if let Some(rotation_2) = rotation_2.as_deref() {
                (rotation_2.ang_vel, rotation_2.inv_inertia())
            } else {
                (0.0, 0.0)
            }
        };

        let vel_1 = lin_vel_1.add(arm_1.cross_scalar(ang_vel_1));
        let vel_2 = lin_vel_2.add(arm_2.cross_scalar(ang_vel_2));

        let normal_inv_mass_inertia =
            inv_mass_1 + inv_mass_2 + inv_inertia_1 * math::pow2(arm_1.cross(normal)) + inv_inertia_2 * math::pow2(arm_2.cross(normal));

        // relative velocity from shape_1 to shape_2, vector from vel_1 to vel_2
        let rel_vel = vel_2.sub(vel_1);
        // normal_rel_vel_mag is basically rel_vel projected on the normal axis
        // remember that normal is the unit vector perpendicular to the edge with minimum overlap
        let normal_rel_vel_mag = rel_vel.dot(normal);

        if normal_rel_vel_mag >= math::EPS {
            // object are not getting closer
            // careful here, since objects resting on other objects have a negative normal_rel_vel_mag very close to 0
            return;
        };

        // so here are the steps to compute impulse (not yet updated for angular velocity):
        //
        // 1) first of all we want to prove that after the impulse, we have:
        // lin_vel_1' = lin_vel_1 - J / mass_1    and    lin_vel_2' = lin_vel_2 + J / mass_2
        // where J is impulse
        //
        // since J = F * t, by Newton's third law we have opposite impulses on the 2 bodies:
        // -J_1 = J_2 = J
        //
        // and since impulse is the change in momentum, we have that:
        // P' = P + J
        // where P is the momentum
        //
        // so replacing into this formula we get:
        // P_1' = P_1 - J    and P_2' = P + J
        //
        // and if we divide by the mass we get:
        // lin_vel_1' = lin_vel_1 - J / mass_1    and    lin_vel_2' = lin_vel_2 + J / mass_2
        //
        // which is exactly what we were looking for
        //
        // 2) elast is definied as:
        // rel_lin_vel' = -elast * rel_lin_vel
        //
        // 3) the relative linear velocity is:
        // rel_lin_vel = lin_vel_2 - lin_vel_1
        //
        // so the new relative linear velocity is:
        // rel_lin_vel' = lin_vel_2' - lin_vel_1'
        //
        // 4) replacing, we have:
        // rel_lin_vel' = (lin_vel_2 + J / mass_2) - (lin_vel_1 - J / mass_1)
        // rel_lin_vel' = lin_vel_2 - lin_vel_1 + J / mass_2 + J / mass_1
        // rel_lin_vel' = rel_lin_vel + J * (1 / mass_2 + 1 / mass_1)
        // -elast * rel_lin_vel = rel_lin_vel + J * (1 / mass_2 + 1 / mass_1)
        // -elast * rel_lin_vel - rel_lin_vel = J * (1 / mass_2 + 1 / mass_1)
        // rel_lin_vel * (-elast - 1) = J * (1 / mass_2 + 1 / mass_1)
        // -rel_lin_vel * (elast + 1) = J * (1 / mass_2 + 1 / mass_1)
        // J = -rel_lin_vel * (elast + 1) / (1 / mass_2 + 1 / mass_1)
        //
        // and rearranging:
        // J = -((1 + elast) * normal_rel_lin_vel_mag / (inv_mass_1 + inv_mass_2))
        let impulse = -((1.0 + elast) * normal_rel_vel_mag / (normal_inv_mass_inertia));
        let impulse_vector = normal.scale(impulse);

        // what we will do with impulse is simply this:
        // since:
        // delta_P = delta_lin_vel * mass = J
        //
        // we get:
        // delta_lin_vel_n = J_n / mass_n
        //
        // so that is the magnitude of delta_lin_vel, the direction is simply the normal direction

        let lin_vel_1 = if let Some(translation_1) = translation_1.as_deref_mut() {
            translation_1.lin_vel.sub_mut(impulse_vector.scale(inv_mass_1)); // here we subtract the delta_lin_vel (see above why)

            // round linear velocity to 0 for object 1
            if translation_1.rest {
                if translation_1.lin_vel.x.abs() <= 0.1 {
                    translation_1.lin_vel.x = 0.0;
                }
                if translation_1.lin_vel.y.abs() <= 0.6 {
                    translation_1.lin_vel.y = 0.0;
                }
            }

            // recompute lin_vel_1
            translation_1.lin_vel
        } else {
            math::Vec2::new(0.0, 0.0)
        };

        let lin_vel_2 = if let Some(translation_2) = translation_2.as_deref_mut() {
            translation_2.lin_vel.add_mut(impulse_vector.scale(inv_mass_2)); // here we add the delta_lin_vel (see above why)

            // round linear velocity to 0 for object 2
            if translation_2.rest {
                if translation_2.lin_vel.x.abs() <= 0.1 {
                    translation_2.lin_vel.x = 0.0;
                }
                if translation_2.rest && translation_2.lin_vel.y.abs() <= 0.6 {
                    translation_2.lin_vel.y = 0.0;
                }
            }

            // recompute lin_vel_2
            translation_2.lin_vel
        } else {
            math::Vec2::new(0.0, 0.0)
        };

        // and for rotation:
        // delta_ang_vel_n = inv_inertia_n * cross(arm_n, impulse_vector)

        let ang_vel_1 = if let Some(rotation_1) = rotation_1.as_deref_mut() {
            rotation_1.ang_vel -= inv_inertia_1 * arm_1.cross(impulse_vector); // here we subtract the delta_ang_vel (see above why)

            // recompute ang_vel_1
            rotation_1.ang_vel
        } else {
            0.0
        };

        let ang_vel_2 = if let Some(rotation_2) = rotation_2.as_deref_mut() {
            rotation_2.ang_vel += inv_inertia_2 * arm_2.cross(impulse_vector); // here we add the delta_ang_vel (see above why)

            // recompute ang_vel_2
            rotation_2.ang_vel
        } else {
            0.0
        };

        // recompute rel_vel and normal_rel_vel_mag
        let vel_1 = lin_vel_1.add(arm_1.cross_scalar(ang_vel_1));
        let vel_2 = lin_vel_2.add(arm_2.cross_scalar(ang_vel_2));

        let rel_vel = vel_2.sub(vel_1);

        let normal_rel_vel_mag = rel_vel.dot(normal);

        // compute friction
        // tangent_rel_vel is the tangent component of rel_vel
        let tangent_rel_vel = rel_vel.sub(normal.scale(normal_rel_vel_mag));
        let tangent_rel_vel_mag = tangent_rel_vel.mag();

        if tangent_rel_vel_mag < math::EPS {
            // no tangential slip, so nothing to correct
            return;
        }

        // tangent_unit is tangent_rel_lin_vel normalized
        let tangent = tangent_rel_vel.scale(1.0 / tangent_rel_vel_mag); // I am not using .norm() because I've already computed the magnitude

        let tangent_inv_mass_inertia =
            inv_mass_1 + inv_mass_2 + inv_inertia_1 * math::pow2(arm_1.cross(tangent)) + inv_inertia_2 * math::pow2(arm_2.cross(tangent));

        let friction_impulse = -tangent_rel_vel_mag / (tangent_inv_mass_inertia); // impulse that would completely stop the objects
        let max_static = static_friction * impulse.abs(); // maximum impulse of static friction

        let friction_impulse = if friction_impulse.abs() <= max_static {
            // static friction cancels all slip
            friction_impulse
        } else {
            // dynamic friction
            -kinetic_friction * impulse.abs()
        };

        // compute the dynamic friction impulse
        let friction_impulse_vector = tangent.scale(friction_impulse);

        if let Some(translation_1) = translation_1.as_deref_mut() {
            translation_1.lin_vel.sub_mut(friction_impulse_vector.scale(inv_mass_1));
        }

        if let Some(translation_2) = translation_2.as_deref_mut() {
            translation_2.lin_vel.add_mut(friction_impulse_vector.scale(inv_mass_2));
        }

        if let Some(rotation_1) = rotation_1.as_deref_mut() {
            rotation_1.ang_vel -= inv_inertia_1 * arm_1.cross(friction_impulse_vector);
        };

        if let Some(rotation_2) = rotation_2.as_deref_mut() {
            rotation_2.ang_vel += inv_inertia_2 * arm_2.cross(friction_impulse_vector);
        };
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub enum State {
    Active,
    Far,
    Still,
    Static,
    Invalid,
}

/// detects collisions and computes reactions for every object
pub fn resolve_collisions<const N: usize>(world: &mut ecs::World<N>, iters: usize) {
    #[inline]
    fn get_state<const N: usize>(world: &ecs::World<N>, entity: ecs::Entity) -> State {
        let mut translation_is_zero = false; // true -> exists but is 0; false -> does not exist
        let mut rot_mat_is_zero = false;

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
            if ang_vel.abs() <= math::EPS {
                // entity is not rotating
                rot_mat_is_zero = true;
            } else {
                // entity is rotating
                return State::Active;
            }
        }

        if translation_is_zero || rot_mat_is_zero {
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
            // println!("entity {entity} marked as {:?}", states.last());
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

            // keep far state for object that are already far
            for idx in 0..len {
                if matches!(states[idx], State::Far) {
                    next_states[idx] = State::Far
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

            // initialize hitbox and swept shape cache
            let mut hitbox_1 = Some(physics::compute_hitbox(state_1, pos_1, rot_mat_1, lin_vel_1, ang_vel_1, body_1));

            let mut swept_shape_1: Option<math::SweptShape> = None;

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

                // broad phase
                if hitbox_1.is_none() {
                    // println!("recomputing hitbox_1 cache");
                    hitbox_1 = Some(physics::compute_hitbox(state_1, pos_1, rot_mat_1, lin_vel_1, ang_vel_1, body_1));
                }

                let hitbox_2 = physics::compute_hitbox(state_2, pos_2, rot_mat_2, lin_vel_2, ang_vel_2, body_2);

                // println!("{entity_1}-{entity_2} checking hitboxes...");
                if !physics::check_hitboxes(hitbox_1.as_ref().unwrap(), &hitbox_2) {
                    // hitboxes are not colliding
                    // println!("  -> hitboxes NOT colliding");
                    continue 'loop_2;
                }
                // println!("  -> hitboxes ARE colliding");

                // hitboxes are colliding, compute swept shapes
                if swept_shape_1.is_none() {
                    // println!("recomputing swept_shape_1 cache");
                    swept_shape_1 = Some(physics::compute_swept_shape(
                        state_1, pos_1, rot_mat_1, lin_vel_1, ang_vel_1, body_1,
                    ));
                }

                let swept_shape_2 = physics::compute_swept_shape(state_2, pos_2, rot_mat_2, lin_vel_2, ang_vel_2, body_2);

                // println!("{entity_1}-{entity_2} checking swept shapes...");
                let Some(normal) = physics::check_sat(swept_shape_1.as_ref().unwrap(), &swept_shape_2) else {
                    // swept shapes are not colliding, no need to compute reaction or invalidate cache
                    // println!("  -> swept shapes NOT colliding");
                    continue 'loop_2;
                };
                // println!("  -> swept shapes ARE colliding");

                // compute contact point and centers of mass
                let contact_point = physics::compute_contact_point(
                    normal, pos_1, pos_2, rot_mat_1, rot_mat_2, lin_vel_1, lin_vel_2, ang_vel_1, ang_vel_2, body_1, body_2,
                );

                let mass_center_1 = match rot_mat_1 {
                    Some(rot_mat) => rot_mat
                        .rot_mat
                        .pre_mul_vec2(body_1.centroid())
                        .add(pos_1)
                        .add(lin_vel_1.unwrap_or(math::Vec2::zero())),
                    None => body_1.centroid().add(pos_1).add(lin_vel_1.unwrap_or(math::Vec2::zero())),
                };
                let mass_center_2 = match rot_mat_2 {
                    Some(rot_mat) => rot_mat
                        .rot_mat
                        .pre_mul_vec2(body_2.centroid())
                        .add(pos_2)
                        .add(lin_vel_2.unwrap_or(math::Vec2::zero())),
                    None => body_2.centroid().add(pos_2).add(lin_vel_2.unwrap_or(math::Vec2::zero())),
                };

                // collision detected
                solved = false;
                entity_1_is_far = false;

                // invalidate cache since it will change with the reaction
                hitbox_1 = None;
                swept_shape_1 = None;

                let (translation_1, translation_2) = world.engine.translation.get2_mut(entity_1, entity_2);
                let (rotation_1, rotation_2) = world.engine.rotation.get2_mut(entity_1, entity_2);

                // println!("{entity_1}-{entity_2} computing reaction");
                compute_reaction(
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
                next_states[idx_1] = State::Far
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
}
