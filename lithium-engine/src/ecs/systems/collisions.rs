use crate::{
    core::error,
    ecs::{components, entities, world::World},
    math::{self, ApplyTransformationVerts, ApplyTransformationVertsStep, EPS, EPS_SQR, Vec2},
};

use std::mem;

/// checks if 2 hitboxes are colliding using EPS to prevent false negatives
fn check_hitboxes(hitbox_1: &math::HitBox, hitbox_2: &math::HitBox) -> bool {
    !(hitbox_1.min_x > hitbox_2.max_x + EPS
        || hitbox_2.min_x > hitbox_1.max_x + EPS
        || hitbox_1.min_y > hitbox_2.max_y + EPS
        || hitbox_2.min_y > hitbox_1.max_y + EPS)
}

/// checks if 2 objects are colliding using SAT algorithm, returns the contact normal
fn check_sat(swept_shape_1: &math::SweptShape, swept_shape_2: &math::SweptShape) -> Option<math::Vec2> {
    fn add_sides(swept_shape: &math::SweptShape, sides: &mut Vec<math::Vec2>) {
        #[inline]
        fn add_polygon_sides(polygon: &math::Polygon, sides: &mut Vec<math::Vec2>) {
            let verts = &polygon.verts;
            let mut prev = *verts.last().unwrap();
            for &curr in verts {
                let side = curr.sub(prev);
                if side.square_mag() > EPS_SQR {
                    sides.push(side);
                }
                prev = curr;
            }
        }

        match swept_shape {
            math::SweptShape::Unchanged(shape) => match shape {
                math::Shape::Segment(segment) => {
                    let segment_side = segment.b.sub(segment.a);
                    if segment_side.square_mag() > EPS_SQR {
                        sides.push(segment_side)
                    }
                }
                math::Shape::Triangle(triangle) => {
                    let triangle_sides = [
                        triangle.b.sub(triangle.a),
                        triangle.c.sub(triangle.b),
                        triangle.a.sub(triangle.c),
                    ];
                    for triangle_side in triangle_sides {
                        if triangle_side.square_mag() > EPS_SQR {
                            sides.push(triangle_side);
                        }
                    }
                }
                math::Shape::Quad(quad) => {
                    let quad_sides = [
                        quad.b.sub(quad.a),
                        quad.c.sub(quad.b),
                        quad.d.sub(quad.c),
                        quad.a.sub(quad.d),
                    ];
                    for quad_side in quad_sides {
                        if quad_side.square_mag() > EPS_SQR {
                            sides.push(quad_side);
                        }
                    }
                }
                math::Shape::Polygon(polygon) => add_polygon_sides(polygon, sides),
                math::Shape::Circle(_) => unimplemented!(),
            },
            math::SweptShape::Changed(swept) => add_polygon_sides(swept, sides),
        }
    }

    fn project_shape(swept_shape: &math::SweptShape, axis: math::Vec2) -> (f32, f32) {
        #[inline]
        fn project_polygon(polygon: &math::Polygon, axis: math::Vec2) -> (f32, f32) {
            let mut min = polygon.verts[0].dot(axis);
            let mut max = min;

            for vert in polygon.verts.iter().skip(1) {
                let proj = vert.dot(axis);
                if proj < min {
                    min = proj;
                }
                if proj > max {
                    max = proj;
                }
            }
            (min, max)
        }

        match swept_shape {
            math::SweptShape::Unchanged(shape) => match shape {
                math::Shape::Segment(segment) => {
                    let (a_proj, b_proj) = (segment.a.dot(axis), segment.b.dot(axis));
                    (a_proj.min(b_proj), a_proj.max(b_proj))
                }
                math::Shape::Triangle(triangle) => {
                    let (a_proj, b_proj, c_proj) = (triangle.a.dot(axis), triangle.b.dot(axis), triangle.c.dot(axis));

                    (a_proj.min(b_proj).min(c_proj), a_proj.max(b_proj).max(c_proj))
                }
                math::Shape::Quad(quad) => {
                    let (a_proj, b_proj, c_proj, d_proj) =
                        (quad.a.dot(axis), quad.b.dot(axis), quad.c.dot(axis), quad.d.dot(axis));

                    (
                        a_proj.min(b_proj).min(c_proj).min(d_proj),
                        a_proj.max(b_proj).max(c_proj).max(d_proj),
                    )
                }
                math::Shape::Polygon(polygon) => project_polygon(polygon, axis),
                math::Shape::Circle(_) => unimplemented!(),
            },
            math::SweptShape::Changed(swept) => project_polygon(swept, axis),
        }
    }

    fn centroid(swept_shape: &math::SweptShape) -> math::Vec2 {
        #[inline]
        fn polygon_centroid(polygon: &math::Polygon) -> math::Vec2 {
            let mut sum = math::Vec2::new(0.0, 0.0);
            for vert in &polygon.verts {
                sum.add_mut(*vert);
            }
            sum.scale(1.0 / polygon.verts.len() as f32)
        }

        match swept_shape {
            math::SweptShape::Unchanged(shape) => match shape {
                math::Shape::Segment(segment) => segment.a.add(segment.b).scale(0.5),
                math::Shape::Triangle(triangle) => triangle.a.add(triangle.b.add(triangle.c)).scale(1.0 / 3.0),
                math::Shape::Quad(quad) => quad.a.add(quad.b.add(quad.c.add(quad.d))).scale(0.25),
                math::Shape::Polygon(polygon) => polygon_centroid(polygon),
                math::Shape::Circle(_) => unimplemented!(),
            },
            math::SweptShape::Changed(swept) => polygon_centroid(swept),
        }
    }

    fn check_axes(
        sides: &[math::Vec2],
        swept_shape_1: &math::SweptShape,
        swept_shape_2: &math::SweptShape,
        delta: Vec2,
        min_overlap: &mut f32,
        normal: &mut Vec2,
    ) -> Option<()> {
        for side in sides {
            let axis = side.perp_ccw().norm();

            let (min_1, max_1) = project_shape(swept_shape_1, axis);
            let (min_2, max_2) = project_shape(swept_shape_2, axis);

            if min_1 > max_2 + EPS || min_2 > max_1 + EPS {
                // not colliding
                return None;
            }

            let overlap = (max_1.min(max_2)) - (min_1.max(min_2));
            if overlap < *min_overlap {
                // update the normal data
                *min_overlap = overlap;
                *normal = if delta.dot(axis) < 0.0 { axis.neg() } else { axis }; // invert the normal direction if it is not from swept_shape_1 to swept_shape_2
            }
        }

        Some(())
    }

    // vector of sides
    let mut sides: Vec<math::Vec2> = Vec::with_capacity(swept_shape_1.sides() + swept_shape_2.sides());
    add_sides(swept_shape_1, &mut sides);

    // compute centroids
    let centroid_1 = centroid(swept_shape_1);
    let centroid_2 = centroid(swept_shape_2);
    let delta = centroid_2.sub(centroid_1); // points from swept_shape_1 to swept_shape_2

    // initialize normal data
    let mut min_overlap = f32::INFINITY;
    let mut normal = math::Vec2::new(0.0, 0.0); // minimum translation vector axis, the axis of the smallest vector to push one shape out of the other

    check_axes(
        &sides,
        swept_shape_1,
        swept_shape_2,
        delta,
        &mut min_overlap,
        &mut normal,
    )?;

    sides.clear();
    add_sides(swept_shape_2, &mut sides);

    check_axes(
        &sides,
        swept_shape_1,
        swept_shape_2,
        delta,
        &mut min_overlap,
        &mut normal,
    )?;

    Some(normal)
}

/// computes hitbox of a swept shape, without computing the swept shape
fn compute_hitbox(
    state: State,
    pos: math::Vec2,
    lin_vel: Option<math::Vec2>,
    rot_mat: Option<&components::RotationMatrix>,
    shape: &math::Shape,
) -> math::HitBox {
    let static_or_still = matches!(state, State::Static | State::Still);

    let pos_2 = match lin_vel {
        Some(v) => pos.add(v),
        None => pos,
    };

    match shape {
        math::Shape::Segment(segment) => {
            if static_or_still {
                // if it is still or static, apply the global position and, if it exists, the rotation
                let verts = match rot_mat {
                    Some(components::RotationMatrix { prev, .. }) => &segment.apply_mat2x3_then_vec2(pos, prev),
                    None => &segment.apply_vec2(pos),
                };
                math::HitBox::from_verts_array(verts)
            } else {
                // if it is far or active use the step variant, which takes into account the movement between one frame and the next
                let verts = match rot_mat {
                    Some(components::RotationMatrix { prev, curr }) => {
                        &segment.apply_mat2x3_then_vec2_step(pos, pos_2, prev, curr)
                    }
                    None => &segment.apply_vec2_step(pos, pos_2),
                };
                math::HitBox::from_verts_array(verts)
            }
        }
        math::Shape::Triangle(triangle) => {
            if static_or_still {
                // if it is still or static, apply the global position and, if it exists, the rotation
                let verts = match rot_mat {
                    Some(components::RotationMatrix { prev, .. }) => &triangle.apply_mat2x3_then_vec2(pos, prev),
                    None => &triangle.apply_vec2(pos),
                };
                math::HitBox::from_verts_array(verts)
            } else {
                // if it is far or active use the step variant, which takes into account the movement between one frame and the next
                let verts = match rot_mat {
                    Some(components::RotationMatrix { prev, curr }) => {
                        &triangle.apply_mat2x3_then_vec2_step(pos, pos_2, prev, curr)
                    }
                    None => &triangle.apply_vec2_step(pos, pos_2),
                };
                math::HitBox::from_verts_array(verts)
            }
        }
        math::Shape::Quad(quad) => {
            if static_or_still {
                // if it is still or static, apply the global position and, if it exists, the rotation
                let verts = match rot_mat {
                    Some(components::RotationMatrix { prev, .. }) => &quad.apply_mat2x3_then_vec2(pos, prev),
                    None => &quad.apply_vec2(pos),
                };
                math::HitBox::from_verts_array(verts)
            } else {
                // if it is far or active use the step variant, which takes into account the movement between one frame and the next
                let verts = match rot_mat {
                    Some(components::RotationMatrix { prev, curr }) => {
                        &quad.apply_mat2x3_then_vec2_step(pos, pos_2, prev, curr)
                    }
                    None => &quad.apply_vec2_step(pos, pos_2),
                };
                math::HitBox::from_verts_array(verts)
            }
        }
        math::Shape::Polygon(polygon) => {
            if static_or_still {
                // if it is still or static, apply the global position and, if it exists, the rotation
                let verts = match rot_mat {
                    Some(components::RotationMatrix { prev, .. }) => &polygon.apply_mat2x3_then_vec2(pos, prev),
                    None => &polygon.apply_vec2(pos),
                };
                math::HitBox::from_verts_slice(verts)
            } else {
                // if it is far or active use the step variant, which takes into account the movement between one frame and the next
                let verts = match rot_mat {
                    Some(components::RotationMatrix { prev, curr }) => {
                        &polygon.apply_mat2x3_then_vec2_step(pos, pos_2, prev, curr)
                    }
                    None => &polygon.apply_vec2_step(pos, pos_2),
                };
                math::HitBox::from_verts_slice(verts)
            }
        }
        math::Shape::Circle(_) => unimplemented!(),
    }
}

/// dedup for slices that uses Vec2.approx_equal()
pub fn dedup_by_approx_equal(slice: &mut [math::Vec2]) -> &mut [math::Vec2] {
    let len = slice.len();
    if len <= 1 {
        return slice;
    }

    let mut read: usize = 1;
    let mut write: usize = 1;

    while read < len {
        if !slice[read].approx_equal(slice[write - 1]) {
            if read != write {
                slice[write] = slice[read];
            }
            write += 1;
        }
        read += 1;
    }

    &mut slice[..write]
}

/// generates a convex hull from a vector of points using monotone chain algorithm
pub fn convex_hull(mut verts: &mut [math::Vec2]) -> Result<math::Polygon, error::GeometryError> {
    if verts.len() < 3 {
        return Err(error::GeometryError::TooFewVertices(verts.len()));
    }

    // sort by x and, if x is the same by y
    verts.sort_unstable_by(|a, b| a.x.total_cmp(&b.x).then_with(|| a.y.total_cmp(&b.y)));

    // remove near-duplicates
    verts = dedup_by_approx_equal(verts);

    if verts.len() < 3 {
        return Err(error::GeometryError::TooFewVertices(verts.len()));
    }

    fn push_vert(boundary: &mut Vec<math::Vec2>, vert: math::Vec2, init_len: usize) {
        while boundary.len() >= init_len {
            let len = boundary.len();
            if (boundary[len - 2]).signed_area(boundary[len - 1], vert) >= 0.0 {
                boundary.pop();
            } else {
                break;
            }
        }
        boundary.push(vert);
    }

    let mut hull: Vec<math::Vec2> = Vec::with_capacity(verts.len() * 2);

    // compute top boundary (clockwise from leftmost to rightmost)
    for &v in verts.iter() {
        push_vert(&mut hull, v, 2)
    }

    if !hull.is_empty() {
        hull.pop();
    }

    let init_len = hull.len() + 2;

    // compute bottom boundary (clockwise from rightmost to leftmost)
    for &v in verts.iter().rev() {
        push_vert(&mut hull, v, init_len);
    }

    if !hull.is_empty() {
        hull.pop();
    }

    Ok(math::Polygon::new_unchecked(hull))
}

/// computes swept shape of a stationary or moving shape
fn compute_swept_shape(
    state: State,
    pos: math::Vec2,
    lin_vel: Option<math::Vec2>,
    rot_mat: Option<&components::RotationMatrix>,
    shape: &math::Shape,
) -> math::SweptShape {
    let static_or_still = matches!(state, State::Static | State::Still);

    let pos_2 = match lin_vel {
        Some(v) => pos.add(v),
        None => pos,
    };

    match shape {
        math::Shape::Segment(segment) => {
            if static_or_still {
                // if it is still or static, apply the global position and, if it exists, the rotation
                let [a, b] = match rot_mat {
                    Some(components::RotationMatrix { prev, .. }) => segment.apply_mat2x3_then_vec2(pos, prev),
                    None => segment.apply_vec2(pos),
                };
                math::SweptShape::Unchanged(math::Shape::Segment(math::Segment::new_unchecked(a, b)))
            } else {
                // if it is far or active use the step variant, which takes into account the movement between one frame and the next
                let mut verts = match rot_mat {
                    Some(components::RotationMatrix { prev, curr }) => {
                        segment.apply_mat2x3_then_vec2_step(pos, pos_2, prev, curr)
                    }
                    None => segment.apply_vec2_step(pos, pos_2),
                };
                math::SweptShape::Changed(convex_hull(&mut verts).unwrap())
            }
        }
        math::Shape::Triangle(triangle) => {
            if static_or_still {
                // if it is still or static, apply the global position and, if it exists, the rotation
                let [a, b, c] = match rot_mat {
                    Some(components::RotationMatrix { prev, .. }) => triangle.apply_mat2x3_then_vec2(pos, prev),
                    None => triangle.apply_vec2(pos),
                };
                math::SweptShape::Unchanged(math::Shape::Triangle(math::Triangle::new_unchecked(a, b, c)))
            } else {
                // if it is far or active use the step variant, which takes into account the movement between one frame and the next
                let mut verts = match rot_mat {
                    Some(components::RotationMatrix { prev, curr }) => {
                        triangle.apply_mat2x3_then_vec2_step(pos, pos_2, prev, curr)
                    }
                    None => triangle.apply_vec2_step(pos, pos_2),
                };
                math::SweptShape::Changed(convex_hull(&mut verts).unwrap())
            }
        }
        math::Shape::Quad(quad) => {
            if static_or_still {
                // if it is still or static, apply the global position and, if it exists, the rotation
                let [a, b, c, d] = match rot_mat {
                    Some(components::RotationMatrix { prev, .. }) => quad.apply_mat2x3_then_vec2(pos, prev),
                    None => quad.apply_vec2(pos),
                };
                math::SweptShape::Unchanged(math::Shape::Quad(math::Quad::new_unchecked(a, b, c, d)))
            } else {
                // if it is far or active use the step variant, which takes into account the movement between one frame and the next
                let mut verts = match rot_mat {
                    Some(components::RotationMatrix { prev, curr }) => {
                        quad.apply_mat2x3_then_vec2_step(pos, pos_2, prev, curr)
                    }
                    None => quad.apply_vec2_step(pos, pos_2),
                };
                math::SweptShape::Changed(convex_hull(&mut verts).unwrap())
            }
        }
        math::Shape::Polygon(polygon) => {
            if static_or_still {
                // if it is still or static, apply the global position and, if it exists, the rotation
                let verts = match rot_mat {
                    Some(components::RotationMatrix { prev, .. }) => polygon.apply_mat2x3_then_vec2(pos, prev),
                    None => polygon.apply_vec2(pos),
                };
                math::SweptShape::Unchanged(math::Shape::Polygon(math::Polygon::new_unchecked(verts)))
            } else {
                // if it is far or active use the step variant, which takes into account the movement between one frame and the next
                let mut verts = match rot_mat {
                    Some(components::RotationMatrix { prev, curr }) => {
                        polygon.apply_mat2x3_then_vec2_step(pos, pos_2, prev, curr)
                    }
                    None => polygon.apply_vec2_step(pos, pos_2),
                };
                math::SweptShape::Changed(convex_hull(&mut verts).unwrap())
            }
        }
        math::Shape::Circle(_) => unimplemented!(),
    }
}

/// updates 2 entities' linear velocity vector after they collide
fn compute_reaction(
    normal: math::Vec2,
    mut translation_1: Option<&mut components::Translation>,
    mut translation_2: Option<&mut components::Translation>,
    surface_1: &components::Surface,
    surface_2: &components::Surface,
) {
    let compute_translation_reaction = translation_1.is_some() || translation_2.is_some();

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

    if compute_translation_reaction {
        // extract lin_vel and inv_mass
        let (lin_vel_1, inv_mass_1) = {
            if let Some(translation_1) = translation_1.as_deref_mut() {
                (translation_1.lin_vel, translation_1.inv_mass())
            } else {
                (math::Vec2::new(0.0, 0.0), 0.0)
            }
        };

        let (lin_vel_2, inv_mass_2) = {
            if let Some(translation_2) = translation_2.as_deref_mut() {
                (translation_2.lin_vel, translation_2.inv_mass())
            } else {
                (math::Vec2::new(0.0, 0.0), 0.0)
            }
        };

        let inv_mass_sum = inv_mass_1 + inv_mass_2;

        // relative linear velocity from shape_1 to shape_2, vector from lin_vel_1 to lin_vel_2
        let rel_lin_vel = lin_vel_2.sub(lin_vel_1);
        // normal_rel_lin_vel_mag is basically rel_lin_vel projected on the normal axis
        // remember that normal is the unit vector perpendicular to the edge with minimum overlap
        let normal_rel_lin_vel_mag = rel_lin_vel.dot(normal);

        if normal_rel_lin_vel_mag >= EPS {
            // object are not getting closer
            // careful here, since objects resting on other objects have a negative normal_rel_lin_vel_mag very close to 0
            return;
        };

        // so here are the steps to compute impulse:
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
        let impulse = -((1.0 + elast) * normal_rel_lin_vel_mag / (inv_mass_sum));
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

            // round y linear velocity to 0 for object 1
            if translation_1.rest && translation_1.lin_vel.y.abs() <= 0.6 {
                translation_1.lin_vel.y = 0.0;
            }

            // recompute lin_vel_1
            translation_1.lin_vel
        } else {
            math::Vec2::new(0.0, 0.0)
        };

        let lin_vel_2 = if let Some(translation_2) = translation_2.as_deref_mut() {
            translation_2.lin_vel.add_mut(impulse_vector.scale(inv_mass_2)); // here we add the delta_lin_vel (see above why)

            // round y linear velocity to 0 for object 2
            if translation_2.rest && translation_2.lin_vel.y.abs() <= 0.6 {
                translation_2.lin_vel.y = 0.0;
            }

            // recompute lin_vel_2
            translation_2.lin_vel
        } else {
            math::Vec2::new(0.0, 0.0)
        };

        // recompute rel_lin_vel and normal_rel_lin_vel_mag
        let rel_lin_vel = lin_vel_2.sub(lin_vel_1);
        let normal_rel_lin_vel_mag = rel_lin_vel.dot(normal);

        // compute friction
        // tangent_rel_lin_vel is the tangent component of rel_lin_vel
        let tangent_rel_lin_vel = rel_lin_vel.sub(normal.scale(normal_rel_lin_vel_mag));
        let tangent_rel_lin_vel_mag = tangent_rel_lin_vel.mag();

        if tangent_rel_lin_vel_mag < EPS {
            // no tangential slip, so nothing to correct
            return;
        }

        // tangent_unit is tangent_rel_lin_vel normalized
        let tangent_unit = tangent_rel_lin_vel.scale(1.0 / tangent_rel_lin_vel_mag); // I am not using .norm() because I've already computed the magnitude

        let friction_impulse = -tangent_rel_lin_vel_mag / (inv_mass_sum); // impulse that would completely stop the objects
        let max_static = static_friction * impulse.abs(); // maximum impulse of static friction

        let friction_impulse = if friction_impulse.abs() <= max_static {
            // static friction cancels all slip
            friction_impulse
        } else {
            // dynamic friction
            -kinetic_friction * impulse.abs()
        };

        // compute the dynamic friction impulse
        let friction_impulse_vector = tangent_unit.scale(friction_impulse);

        if let Some(translation_1) = translation_1.as_deref_mut() {
            translation_1.lin_vel.sub_mut(friction_impulse_vector.scale(inv_mass_1));
        }

        if let Some(translation_2) = translation_2.as_deref_mut() {
            translation_2.lin_vel.add_mut(friction_impulse_vector.scale(inv_mass_2));
        }
    }
}

#[repr(u8)]
#[derive(Copy, Clone, Debug)]
enum State {
    Active,
    Far,
    Still,
    Static,
    Invalid,
}

/// detects collisions and computes reactions for every object
pub fn resolve_collisions<const N: usize>(world: &mut World<N>, iters: usize) {
    #[inline]
    fn get_state<const N: usize>(world: &World<N>, entity: entities::Entity) -> State {
        let mut translation_is_zero = false; // true -> exists but is 0; false -> does not exist
        let mut rot_mat_is_zero = false;

        if let Some(&components::Translation { lin_vel, .. }) = world.engine.translation.get(entity) {
            // entity can move
            if lin_vel.approx_equal_zero() {
                // entity is not moving
                translation_is_zero = true;
            } else {
                // entity is moving
                return State::Active;
            }
        }

        if let Some(rot_mat) = world.engine.rotation_matrix.get(entity) {
            // entity can rotate
            if rot_mat.curr_approx_equal_prev() {
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
            world.engine.shape.get(entity),
        ) {
            // entity is a valid object
            states.push(get_state(world, entity));
            // let a = get_state(world, entity);
            // states.push(a);
            // println!("entity {entity} marked as {a:?}");
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

            let (&components::Transform { pos: pos_1, .. }, mut rot_mat_1, mut lin_vel_1, surface_1, shape_1) = (
                world.engine.transform.get(entity_1).expect("missing transform"),
                world.engine.rotation_matrix.get(entity_1),
                world.engine.translation.get(entity_1).map(|t| t.lin_vel), // extract lin_vel
                world.engine.surface.get(entity_1).expect("missing surface"),
                world.engine.shape.get(entity_1).expect("missing shape"),
            );

            // initialize hitbox and swept shape cache
            let mut hitbox_1 = Some(compute_hitbox(state_1, pos_1, lin_vel_1, rot_mat_1, shape_1));

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

                let (&components::Transform { pos: pos_2, .. }, rot_mat_2, lin_vel_2, surface_2, shape_2) = (
                    world.engine.transform.get(entity_2).expect("missing transform"),
                    world.engine.rotation_matrix.get(entity_2),
                    world.engine.translation.get(entity_2).map(|t| t.lin_vel), // extract lin_vel
                    world.engine.surface.get(entity_2).expect("missing surface"),
                    world.engine.shape.get(entity_2).expect("missing shape"),
                );

                // broad phase
                if hitbox_1.is_none() {
                    // println!("recomputing hitbox_1 cache");
                    hitbox_1 = Some(compute_hitbox(state_1, pos_1, lin_vel_1, rot_mat_1, shape_1));
                }

                let hitbox_2 = compute_hitbox(state_2, pos_2, lin_vel_2, rot_mat_2, shape_2);

                // println!("{entity_1}-{entity_2} checking hitboxes...");
                if !check_hitboxes(hitbox_1.as_ref().unwrap(), &hitbox_2) {
                    // hitboxes are not colliding
                    // println!("  -> hitboxes NOT colliding");
                    continue 'loop_2;
                }
                // println!("  -> hitboxes ARE colliding");

                // hitboxes are colliding, compute swept shapes
                if swept_shape_1.is_none() {
                    // println!("recomputing swept_shape_1 cache");
                    swept_shape_1 = Some(compute_swept_shape(state_1, pos_1, lin_vel_1, rot_mat_1, &shape_1));
                }

                let swept_shape_2 = compute_swept_shape(state_2, pos_2, lin_vel_2, rot_mat_2, &shape_2);

                // println!("{entity_1}-{entity_2} checking swept shapes...");
                let Some(normal) = check_sat(swept_shape_1.as_ref().unwrap(), &swept_shape_2) else {
                    // swept shapes are not colliding, no need to compute reaction or invalidate cache
                    // println!("  -> swept shapes NOT colliding");
                    continue 'loop_2;
                };
                // println!("  -> swept shapes ARE colliding");

                // collision detected
                solved = false;
                entity_1_is_far = false;

                // invalidate cache since it will change with the reaction
                hitbox_1 = None;
                swept_shape_1 = None;

                let (translation_1, translation_2) = world.engine.translation.get2_mut(entity_1, entity_2);

                // println!("{entity_1}-{entity_2} computing reaction");
                compute_reaction(normal, translation_1, translation_2, surface_1, surface_2);

                // update rot_mat and lin_vel for entity_1 since they are cached for the duration of the inner loop
                // note how we can't just set them to None, since the solver would then treat them as non-existing
                // components, and it may even conflict with the states
                rot_mat_1 = world.engine.rotation_matrix.get(entity_1);
                lin_vel_1 = world.engine.translation.get(entity_1).map(|t| t.lin_vel);

                // here active state means "the entity did collide"; later, it is rechecked to see if the actual state is different,
                // but we need this to:
                // - make sure a previously far en{entity_1}-{entity_2} swept shapes not colliding{entity_1}-{entity_2} swept shapes not colliding{entity_1}-{entity_2} swept shapes not collidingtity that collided as entity_2 is not kept far
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
