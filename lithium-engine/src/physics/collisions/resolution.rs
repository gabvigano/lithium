use crate::{ecs, math};

/// updates 2 entities' linear velocity vector after they collide
pub(crate) fn compute_reaction(
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
                (math::ZERO_VEC2, 0.0)
            }
        };

        let (lin_vel_2, inv_mass_2) = {
            if let Some(translation_2) = translation_2.as_deref() {
                (translation_2.lin_vel, translation_2.inv_mass())
            } else {
                (math::ZERO_VEC2, 0.0)
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
            math::ZERO_VEC2
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
            math::ZERO_VEC2
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
