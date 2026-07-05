use crate::math::{ApplyTransformationVerts, Centroid, SatCompatible};
use crate::{ecs, math, physics};

/// checks if 2 hitboxes are colliding using EPS to prevent false negatives
pub fn check_hitboxes(hitbox_1: &math::HitBox, hitbox_2: &math::HitBox) -> bool {
    !(hitbox_1.min_x > hitbox_2.max_x + math::EPS
        || hitbox_2.min_x > hitbox_1.max_x + math::EPS
        || hitbox_1.min_y > hitbox_2.max_y + math::EPS
        || hitbox_2.min_y > hitbox_1.max_y + math::EPS)
}

/// checks if 2 objects are colliding using SAT algorithm, returns the contact normal
pub fn check_sat<T, U>(geometry_1: &T, geometry_2: &U) -> Option<math::Vec2>
where
    T: SatCompatible + Centroid,
    U: SatCompatible + Centroid,
{
    fn check_axes<T, U>(
        sides: &[math::Vec2],
        geometry_1: &T,
        geometry_2: &U,
        delta: math::Vec2,
        min_overlap: &mut f32,
        normal: &mut math::Vec2,
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
            if overlap < *min_overlap {
                // update the normal data
                *min_overlap = overlap;
                *normal = if delta.dot(axis) < 0.0 { axis.neg() } else { axis }; // invert the normal direction if it is not from swept_shape_1 to swept_shape_2
            }
        }

        Some(())
    }

    // vector of sides
    let mut sides: Vec<math::Vec2> = Vec::with_capacity(geometry_1.sides_number() + geometry_2.sides_number());
    geometry_1.append_sides(&mut sides);

    // compute centroids
    let centroid_1 = geometry_1.centroid();
    let centroid_2 = geometry_2.centroid();
    let delta = centroid_2.sub(centroid_1); // points from swept_shape_1 to swept_shape_2

    // initialize normal data
    let mut min_overlap = f32::INFINITY;
    let mut normal = math::Vec2::new(0.0, 0.0); // minimum translation vector axis, the axis of the smallest vector to push one shape out of the other

    check_axes(&sides, geometry_1, geometry_2, delta, &mut min_overlap, &mut normal)?;

    sides.clear();
    geometry_2.append_sides(&mut sides);

    check_axes(&sides, geometry_1, geometry_2, delta, &mut min_overlap, &mut normal)?;

    Some(normal)
}

/// computes hitbox of a swept shape, without computing the swept shape
pub fn compute_hitbox(
    state: physics::State,
    pos: math::Vec2,
    rot_mat: Option<&ecs::RotationMatrix>,
    lin_vel: Option<math::Vec2>,
    ang_vel: Option<f32>,
    body: &ecs::Body,
) -> math::HitBox {
    let static_or_still = matches!(state, physics::State::Static | physics::State::Still);

    let pos_2 = match lin_vel {
        Some(lv) => pos.add(lv),
        None => pos,
    };

    let rot_mat_2: Option<&ecs::RotationMatrix> = match (rot_mat, ang_vel) {
        (Some(rm), Some(av)) => Some(&rm.update(math::Radians(av), rm.rot_mat.pre_mul_vec2(body.centroid))),
        (Some(rm), None) => Some(rm),
        (None, Some(_)) => panic!("ang_vel exists but there is no rot_mat"),
        (None, None) => None,
    };

    match &body.shape {
        math::Shape::Segment(segment) => {
            if static_or_still {
                // if it is still or static, apply the global position and, if it exists, the rotation
                let verts = match rot_mat {
                    Some(ecs::RotationMatrix { rot_mat: rm }) => &segment.apply_mat2x3_then_vec2(pos, rm),
                    None => &segment.apply_vec2(pos),
                };
                math::HitBox::from_verts_array(verts)
            } else {
                // if it is far or active use the step variant, which takes into account the movement between one frame and the next
                let verts = match (rot_mat, rot_mat_2) {
                    (Some(ecs::RotationMatrix { rot_mat: rm }), Some(ecs::RotationMatrix { rot_mat: rm_2 })) => {
                        &segment.apply_mat2x3_then_vec2_step(pos, pos_2, rm, rm_2)
                    }
                    (Some(ecs::RotationMatrix { rot_mat: rm }), None) => &segment.apply_mat2x3_then_vec2_step(pos, pos_2, rm, rm),
                    (None, Some(_)) => panic!("rot_mat_2 exists but there is not rot_mat"),
                    (None, None) => &segment.apply_vec2_step(pos, pos_2),
                };
                math::HitBox::from_verts_array(verts)
            }
        }
        math::Shape::Triangle(triangle) => {
            if static_or_still {
                // if it is still or static, apply the global position and, if it exists, the rotation
                let verts = match rot_mat {
                    Some(ecs::RotationMatrix { rot_mat: rm }) => &triangle.apply_mat2x3_then_vec2(pos, rm),
                    None => &triangle.apply_vec2(pos),
                };
                math::HitBox::from_verts_array(verts)
            } else {
                // if it is far or active use the step variant, which takes into account the movement between one frame and the next
                let verts = match (rot_mat, rot_mat_2) {
                    (Some(ecs::RotationMatrix { rot_mat: rm }), Some(ecs::RotationMatrix { rot_mat: rm_2 })) => {
                        &triangle.apply_mat2x3_then_vec2_step(pos, pos_2, rm, rm_2)
                    }
                    (Some(ecs::RotationMatrix { rot_mat: rm }), None) => &triangle.apply_mat2x3_then_vec2_step(pos, pos_2, rm, rm),
                    (None, Some(_)) => panic!("rot_mat_2 exists but there is not rot_mat"),
                    (None, None) => &triangle.apply_vec2_step(pos, pos_2),
                };
                math::HitBox::from_verts_array(verts)
            }
        }
        math::Shape::Quad(quad) => {
            if static_or_still {
                // if it is still or static, apply the global position and, if it exists, the rotation
                let verts = match rot_mat {
                    Some(ecs::RotationMatrix { rot_mat: rm }) => &quad.apply_mat2x3_then_vec2(pos, rm),
                    None => &quad.apply_vec2(pos),
                };
                math::HitBox::from_verts_array(verts)
            } else {
                // if it is far or active use the step variant, which takes into account the movement between one frame and the next
                let verts = match (rot_mat, rot_mat_2) {
                    (Some(ecs::RotationMatrix { rot_mat: rm }), Some(ecs::RotationMatrix { rot_mat: rm_2 })) => {
                        &quad.apply_mat2x3_then_vec2_step(pos, pos_2, rm, rm_2)
                    }
                    (Some(ecs::RotationMatrix { rot_mat: rm }), None) => &quad.apply_mat2x3_then_vec2_step(pos, pos_2, rm, rm),
                    (None, Some(_)) => panic!("rot_mat_2 exists but there is not rot_mat"),
                    (None, None) => &quad.apply_vec2_step(pos, pos_2),
                };
                math::HitBox::from_verts_array(verts)
            }
        }
        math::Shape::Polygon(polygon) => {
            if static_or_still {
                // if it is still or static, apply the global position and, if it exists, the rotation
                let verts = match rot_mat {
                    Some(ecs::RotationMatrix { rot_mat: rm }) => &polygon.apply_mat2x3_then_vec2(pos, rm),
                    None => &polygon.apply_vec2(pos),
                };
                math::HitBox::from_verts_slice(verts)
            } else {
                // if it is far or active use the step variant, which takes into account the movement between one frame and the next
                let verts = match (rot_mat, rot_mat_2) {
                    (Some(ecs::RotationMatrix { rot_mat: rm }), Some(ecs::RotationMatrix { rot_mat: rm_2 })) => {
                        &polygon.apply_mat2x3_then_vec2_step(pos, pos_2, rm, rm_2)
                    }
                    (Some(ecs::RotationMatrix { rot_mat: rm }), None) => &polygon.apply_mat2x3_then_vec2_step(pos, pos_2, rm, rm),
                    (None, Some(_)) => panic!("rot_mat_2 exists but there is not rot_mat"),
                    (None, None) => &polygon.apply_vec2_step(pos, pos_2),
                };
                math::HitBox::from_verts_slice(verts)
            }
        }
        math::Shape::Circle(_) => unimplemented!(),
    }
}

/// computes swept shape of a stationary or moving shape
pub fn compute_swept_shape(
    state: physics::State,
    pos: math::Vec2,
    rot_mat: Option<&ecs::RotationMatrix>,
    lin_vel: Option<math::Vec2>,
    ang_vel: Option<f32>,
    body: &ecs::Body,
) -> math::SweptShape {
    let static_or_still = matches!(state, physics::State::Static | physics::State::Still);

    let pos_2 = match lin_vel {
        Some(v) => pos.add(v),
        None => pos,
    };

    let rot_mat_2: Option<&ecs::RotationMatrix> = match (rot_mat, ang_vel) {
        (Some(rm), Some(av)) => Some(&rm.update(math::Radians(av), rm.rot_mat.pre_mul_vec2(body.centroid))),
        (Some(rm), None) => Some(rm),
        (None, Some(_)) => panic!("ang_vel exists but there is no rot_mat"),
        (None, None) => None,
    };

    match &body.shape {
        math::Shape::Segment(segment) => {
            if static_or_still {
                // if it is still or static, apply the global position and, if it exists, the rotation
                let [a, b] = match rot_mat {
                    Some(ecs::RotationMatrix { rot_mat: rm }) => segment.apply_mat2x3_then_vec2(pos, rm),
                    None => segment.apply_vec2(pos),
                };
                math::SweptShape::Unchanged(math::Shape::Segment(math::Segment::new_unchecked(a, b)))
            } else {
                // if it is far or active use the step variant, which takes into account the movement between one frame and the next
                let mut verts = match (rot_mat, rot_mat_2) {
                    (Some(ecs::RotationMatrix { rot_mat: rm }), Some(ecs::RotationMatrix { rot_mat: rm_2 })) => {
                        segment.apply_mat2x3_then_vec2_step(pos, pos_2, rm, rm_2)
                    }
                    (Some(ecs::RotationMatrix { rot_mat: rm }), None) => segment.apply_mat2x3_then_vec2_step(pos, pos_2, rm, rm),
                    (None, Some(_)) => panic!("rot_mat_2 exists but there is not rot_mat"),
                    (None, None) => segment.apply_vec2_step(pos, pos_2),
                };
                math::SweptShape::Changed(math::convex_hull(&mut verts).unwrap())
            }
        }
        math::Shape::Triangle(triangle) => {
            if static_or_still {
                // if it is still or static, apply the global position and, if it exists, the rotation
                let [a, b, c] = match rot_mat {
                    Some(ecs::RotationMatrix { rot_mat: rm }) => triangle.apply_mat2x3_then_vec2(pos, rm),
                    None => triangle.apply_vec2(pos),
                };
                math::SweptShape::Unchanged(math::Shape::Triangle(math::Triangle::new_unchecked(a, b, c)))
            } else {
                // if it is far or active use the step variant, which takes into account the movement between one frame and the next
                let mut verts = match (rot_mat, rot_mat_2) {
                    (Some(ecs::RotationMatrix { rot_mat: rm }), Some(ecs::RotationMatrix { rot_mat: rm_2 })) => {
                        triangle.apply_mat2x3_then_vec2_step(pos, pos_2, rm, rm_2)
                    }
                    (Some(ecs::RotationMatrix { rot_mat: rm }), None) => triangle.apply_mat2x3_then_vec2_step(pos, pos_2, rm, rm),
                    (None, Some(_)) => panic!("rot_mat_2 exists but there is not rot_mat"),
                    (None, None) => triangle.apply_vec2_step(pos, pos_2),
                };
                math::SweptShape::Changed(math::convex_hull(&mut verts).unwrap())
            }
        }
        math::Shape::Quad(quad) => {
            if static_or_still {
                // if it is still or static, apply the global position and, if it exists, the rotation
                let [a, b, c, d] = match rot_mat {
                    Some(ecs::RotationMatrix { rot_mat: rm }) => quad.apply_mat2x3_then_vec2(pos, rm),
                    None => quad.apply_vec2(pos),
                };
                math::SweptShape::Unchanged(math::Shape::Quad(math::Quad::new_unchecked(a, b, c, d)))
            } else {
                // if it is far or active use the step variant, which takes into account the movement between one frame and the next
                let mut verts = match (rot_mat, rot_mat_2) {
                    (Some(ecs::RotationMatrix { rot_mat: rm }), Some(ecs::RotationMatrix { rot_mat: rm_2 })) => {
                        quad.apply_mat2x3_then_vec2_step(pos, pos_2, rm, rm_2)
                    }
                    (Some(ecs::RotationMatrix { rot_mat: rm }), None) => quad.apply_mat2x3_then_vec2_step(pos, pos_2, rm, rm),
                    (None, Some(_)) => panic!("rot_mat_2 exists but there is not rot_mat"),
                    (None, None) => quad.apply_vec2_step(pos, pos_2),
                };
                math::SweptShape::Changed(math::convex_hull(&mut verts).unwrap())
            }
        }
        math::Shape::Polygon(polygon) => {
            if static_or_still {
                // if it is still or static, apply the global position and, if it exists, the rotation
                let verts = match rot_mat {
                    Some(ecs::RotationMatrix { rot_mat: rm }) => polygon.apply_mat2x3_then_vec2(pos, rm),
                    None => polygon.apply_vec2(pos),
                };
                math::SweptShape::Unchanged(math::Shape::Polygon(math::Polygon::new_unchecked(verts)))
            } else {
                // if it is far or active use the step variant, which takes into account the movement between one frame and the next
                let mut verts = match (rot_mat, rot_mat_2) {
                    (Some(ecs::RotationMatrix { rot_mat: rm }), Some(ecs::RotationMatrix { rot_mat: rm_2 })) => {
                        polygon.apply_mat2x3_then_vec2_step(pos, pos_2, rm, rm_2)
                    }
                    (Some(ecs::RotationMatrix { rot_mat: rm }), None) => polygon.apply_mat2x3_then_vec2_step(pos, pos_2, rm, rm),
                    (None, Some(_)) => panic!("rot_mat_2 exists but there is not rot_mat"),
                    (None, None) => polygon.apply_vec2_step(pos, pos_2),
                };
                math::SweptShape::Changed(math::convex_hull(&mut verts).unwrap())
            }
        }
        math::Shape::Circle(_) => unimplemented!(),
    }
}
