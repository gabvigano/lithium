use crate::{
    core::error,
    ecs::{components, entities, world::World},
    math::{self, ApplyTransformationVerts, ApplyTransformationVertsStep, Centroid, EPS, EPS_SQR},
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

    fn swept_centroid(swept_shape: &math::SweptShape) -> math::Vec2 {
        match swept_shape {
            math::SweptShape::Unchanged(shape) => shape.centroid(),
            math::SweptShape::Changed(swept) => swept.centroid(),
        }
    }

    fn check_axes(
        sides: &[math::Vec2],
        swept_shape_1: &math::SweptShape,
        swept_shape_2: &math::SweptShape,
        delta: math::Vec2,
        min_overlap: &mut f32,
        normal: &mut math::Vec2,
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
    let centroid_1 = swept_centroid(swept_shape_1);
    let centroid_2 = swept_centroid(swept_shape_2);
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
    rot_mat: Option<&components::RotationMatrix>,
    lin_vel: Option<math::Vec2>,
    ang_vel: Option<f32>,
    body: &components::Body,
) -> math::HitBox {
    let static_or_still = matches!(state, State::Static | State::Still);

    let pos_2 = match lin_vel {
        Some(lv) => pos.add(lv),
        None => pos,
    };

    let rot_mat_2: Option<&components::RotationMatrix> = match (rot_mat, ang_vel) {
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
                    Some(components::RotationMatrix { rot_mat: rm }) => &segment.apply_mat2x3_then_vec2(pos, rm),
                    None => &segment.apply_vec2(pos),
                };
                math::HitBox::from_verts_array(verts)
            } else {
                // if it is far or active use the step variant, which takes into account the movement between one frame and the next
                let verts = match (rot_mat, rot_mat_2) {
                    (
                        Some(components::RotationMatrix { rot_mat: rm }),
                        Some(components::RotationMatrix { rot_mat: rm_2 }),
                    ) => &segment.apply_mat2x3_then_vec2_step(pos, pos_2, rm, rm_2),
                    (Some(components::RotationMatrix { rot_mat: rm }), None) => {
                        &segment.apply_mat2x3_then_vec2_step(pos, pos_2, rm, rm)
                    }
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
                    Some(components::RotationMatrix { rot_mat: rm }) => &triangle.apply_mat2x3_then_vec2(pos, rm),
                    None => &triangle.apply_vec2(pos),
                };
                math::HitBox::from_verts_array(verts)
            } else {
                // if it is far or active use the step variant, which takes into account the movement between one frame and the next
                let verts = match (rot_mat, rot_mat_2) {
                    (
                        Some(components::RotationMatrix { rot_mat: rm }),
                        Some(components::RotationMatrix { rot_mat: rm_2 }),
                    ) => &triangle.apply_mat2x3_then_vec2_step(pos, pos_2, rm, rm_2),
                    (Some(components::RotationMatrix { rot_mat: rm }), None) => {
                        &triangle.apply_mat2x3_then_vec2_step(pos, pos_2, rm, rm)
                    }
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
                    Some(components::RotationMatrix { rot_mat: rm }) => &quad.apply_mat2x3_then_vec2(pos, rm),
                    None => &quad.apply_vec2(pos),
                };
                math::HitBox::from_verts_array(verts)
            } else {
                // if it is far or active use the step variant, which takes into account the movement between one frame and the next
                let verts = match (rot_mat, rot_mat_2) {
                    (
                        Some(components::RotationMatrix { rot_mat: rm }),
                        Some(components::RotationMatrix { rot_mat: rm_2 }),
                    ) => &quad.apply_mat2x3_then_vec2_step(pos, pos_2, rm, rm_2),
                    (Some(components::RotationMatrix { rot_mat: rm }), None) => {
                        &quad.apply_mat2x3_then_vec2_step(pos, pos_2, rm, rm)
                    }
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
                    Some(components::RotationMatrix { rot_mat: rm }) => &polygon.apply_mat2x3_then_vec2(pos, rm),
                    None => &polygon.apply_vec2(pos),
                };
                math::HitBox::from_verts_slice(verts)
            } else {
                // if it is far or active use the step variant, which takes into account the movement between one frame and the next
                let verts = match (rot_mat, rot_mat_2) {
                    (
                        Some(components::RotationMatrix { rot_mat: rm }),
                        Some(components::RotationMatrix { rot_mat: rm_2 }),
                    ) => &polygon.apply_mat2x3_then_vec2_step(pos, pos_2, rm, rm_2),
                    (Some(components::RotationMatrix { rot_mat: rm }), None) => {
                        &polygon.apply_mat2x3_then_vec2_step(pos, pos_2, rm, rm)
                    }
                    (None, Some(_)) => panic!("rot_mat_2 exists but there is not rot_mat"),
                    (None, None) => &polygon.apply_vec2_step(pos, pos_2),
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
    rot_mat: Option<&components::RotationMatrix>,
    lin_vel: Option<math::Vec2>,
    ang_vel: Option<f32>,
    body: &components::Body,
) -> math::SweptShape {
    let static_or_still = matches!(state, State::Static | State::Still);

    let pos_2 = match lin_vel {
        Some(v) => pos.add(v),
        None => pos,
    };

    let rot_mat_2: Option<&components::RotationMatrix> = match (rot_mat, ang_vel) {
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
                    Some(components::RotationMatrix { rot_mat: rm }) => segment.apply_mat2x3_then_vec2(pos, rm),
                    None => segment.apply_vec2(pos),
                };
                math::SweptShape::Unchanged(math::Shape::Segment(math::Segment::new_unchecked(a, b)))
            } else {
                // if it is far or active use the step variant, which takes into account the movement between one frame and the next
                let mut verts = match (rot_mat, rot_mat_2) {
                    (
                        Some(components::RotationMatrix { rot_mat: rm }),
                        Some(components::RotationMatrix { rot_mat: rm_2 }),
                    ) => segment.apply_mat2x3_then_vec2_step(pos, pos_2, rm, rm_2),
                    (Some(components::RotationMatrix { rot_mat: rm }), None) => {
                        segment.apply_mat2x3_then_vec2_step(pos, pos_2, rm, rm)
                    }
                    (None, Some(_)) => panic!("rot_mat_2 exists but there is not rot_mat"),
                    (None, None) => segment.apply_vec2_step(pos, pos_2),
                };
                math::SweptShape::Changed(convex_hull(&mut verts).unwrap())
            }
        }
        math::Shape::Triangle(triangle) => {
            if static_or_still {
                // if it is still or static, apply the global position and, if it exists, the rotation
                let [a, b, c] = match rot_mat {
                    Some(components::RotationMatrix { rot_mat: rm }) => triangle.apply_mat2x3_then_vec2(pos, rm),
                    None => triangle.apply_vec2(pos),
                };
                math::SweptShape::Unchanged(math::Shape::Triangle(math::Triangle::new_unchecked(a, b, c)))
            } else {
                // if it is far or active use the step variant, which takes into account the movement between one frame and the next
                let mut verts = match (rot_mat, rot_mat_2) {
                    (
                        Some(components::RotationMatrix { rot_mat: rm }),
                        Some(components::RotationMatrix { rot_mat: rm_2 }),
                    ) => triangle.apply_mat2x3_then_vec2_step(pos, pos_2, rm, rm_2),
                    (Some(components::RotationMatrix { rot_mat: rm }), None) => {
                        triangle.apply_mat2x3_then_vec2_step(pos, pos_2, rm, rm)
                    }
                    (None, Some(_)) => panic!("rot_mat_2 exists but there is not rot_mat"),
                    (None, None) => triangle.apply_vec2_step(pos, pos_2),
                };
                math::SweptShape::Changed(convex_hull(&mut verts).unwrap())
            }
        }
        math::Shape::Quad(quad) => {
            if static_or_still {
                // if it is still or static, apply the global position and, if it exists, the rotation
                let [a, b, c, d] = match rot_mat {
                    Some(components::RotationMatrix { rot_mat: rm }) => quad.apply_mat2x3_then_vec2(pos, rm),
                    None => quad.apply_vec2(pos),
                };
                math::SweptShape::Unchanged(math::Shape::Quad(math::Quad::new_unchecked(a, b, c, d)))
            } else {
                // if it is far or active use the step variant, which takes into account the movement between one frame and the next
                let mut verts = match (rot_mat, rot_mat_2) {
                    (
                        Some(components::RotationMatrix { rot_mat: rm }),
                        Some(components::RotationMatrix { rot_mat: rm_2 }),
                    ) => quad.apply_mat2x3_then_vec2_step(pos, pos_2, rm, rm_2),
                    (Some(components::RotationMatrix { rot_mat: rm }), None) => {
                        quad.apply_mat2x3_then_vec2_step(pos, pos_2, rm, rm)
                    }
                    (None, Some(_)) => panic!("rot_mat_2 exists but there is not rot_mat"),
                    (None, None) => quad.apply_vec2_step(pos, pos_2),
                };
                math::SweptShape::Changed(convex_hull(&mut verts).unwrap())
            }
        }
        math::Shape::Polygon(polygon) => {
            if static_or_still {
                // if it is still or static, apply the global position and, if it exists, the rotation
                let verts = match rot_mat {
                    Some(components::RotationMatrix { rot_mat: rm }) => polygon.apply_mat2x3_then_vec2(pos, rm),
                    None => polygon.apply_vec2(pos),
                };
                math::SweptShape::Unchanged(math::Shape::Polygon(math::Polygon::new_unchecked(verts)))
            } else {
                // if it is far or active use the step variant, which takes into account the movement between one frame and the next
                let mut verts = match (rot_mat, rot_mat_2) {
                    (
                        Some(components::RotationMatrix { rot_mat: rm }),
                        Some(components::RotationMatrix { rot_mat: rm_2 }),
                    ) => polygon.apply_mat2x3_then_vec2_step(pos, pos_2, rm, rm_2),
                    (Some(components::RotationMatrix { rot_mat: rm }), None) => {
                        polygon.apply_mat2x3_then_vec2_step(pos, pos_2, rm, rm)
                    }
                    (None, Some(_)) => panic!("rot_mat_2 exists but there is not rot_mat"),
                    (None, None) => polygon.apply_vec2_step(pos, pos_2),
                };
                math::SweptShape::Changed(convex_hull(&mut verts).unwrap())
            }
        }
        math::Shape::Circle(_) => unimplemented!(),
    }
}

enum Feature {
    Vertex(math::Vec2),
    Edge(math::Segment),
}

impl Feature {
    #[inline]
    fn apply_vec2_mut(&mut self, vec: math::Vec2) {
        match self {
            Self::Vertex(vertex) => vertex.add_mut(vec),
            Self::Edge(segment) => {
                segment.a.add_mut(vec);
                segment.b.add_mut(vec);
            }
        }
    }

    #[allow(dead_code)]
    #[inline]
    fn apply_mat2x3_mut(&mut self, mat: &math::Mat2x3) {
        match self {
            Self::Vertex(vertex) => *vertex = mat.pre_mul_vec2(*vertex),
            Self::Edge(segment) => {
                segment.a = mat.pre_mul_vec2(segment.a);
                segment.b = mat.pre_mul_vec2(segment.b);
            }
        }
    }
}

/// extimates the contact point of a collision
fn compute_contact_point(
    normal: math::Vec2,
    pos_1: math::Vec2,
    pos_2: math::Vec2,
    rot_mat_1: Option<&components::RotationMatrix>,
    rot_mat_2: Option<&components::RotationMatrix>,
    lin_vel_1: Option<math::Vec2>,
    lin_vel_2: Option<math::Vec2>,
    ang_vel_1: Option<f32>,
    ang_vel_2: Option<f32>,
    body_1: &components::Body,
    body_2: &components::Body,
) -> math::Vec2 {
    const FEATURE_MARGIN: f32 = 0.05;
    const PARALLEL_EPS: f32 = 0.02;

    fn find_support_feature(
        normal: math::Vec2,
        rot_mat: Option<&components::RotationMatrix>,
        shape: &math::Shape,
    ) -> Feature {
        fn find_support_feature_from_pairs(pairs: &[(math::Vec2, f32)]) -> Feature {
            let n_sides = pairs.len();

            let mut best_idx = 0;
            for i in 1..n_sides {
                if pairs[i].1 > pairs[best_idx].1 {
                    best_idx = i;
                }
            }

            let prev_idx = (best_idx + n_sides - 1) % n_sides;
            let next_idx = (best_idx + 1) % n_sides;

            let second_idx = if pairs[prev_idx].1 >= pairs[next_idx].1 {
                prev_idx
            } else {
                next_idx
            };

            if pairs[best_idx].1 - pairs[second_idx].1 <= FEATURE_MARGIN {
                Feature::Edge(math::Segment::new_unchecked(pairs[best_idx].0, pairs[second_idx].0))
            } else {
                Feature::Vertex(pairs[best_idx].0)
            }
        }
        match shape {
            math::Shape::Segment(segment) => {
                let [a, b] = match rot_mat {
                    Some(components::RotationMatrix { rot_mat: rm }) => segment.apply_mat2x3(rm),
                    None => [segment.a, segment.b],
                };

                let a_dot = a.dot(normal);
                let b_dot = b.dot(normal);

                if (a_dot - b_dot).abs() <= FEATURE_MARGIN {
                    Feature::Edge(math::Segment::new_unchecked(a, b))
                } else if a_dot >= b_dot {
                    Feature::Vertex(a)
                } else {
                    Feature::Vertex(b)
                }
            }
            math::Shape::Triangle(triangle) => {
                let [a, b, c] = match rot_mat {
                    Some(components::RotationMatrix { rot_mat: rm }) => triangle.apply_mat2x3(rm),
                    None => [triangle.a, triangle.b, triangle.c],
                };

                let pairs = [(a, a.dot(normal)), (b, b.dot(normal)), (c, c.dot(normal))];

                find_support_feature_from_pairs(&pairs)
            }
            math::Shape::Quad(quad) => {
                let [a, b, c, d] = match rot_mat {
                    Some(components::RotationMatrix { rot_mat: rm }) => quad.apply_mat2x3(rm),
                    None => [quad.a, quad.b, quad.c, quad.d],
                };

                let pairs = [
                    (a, a.dot(normal)),
                    (b, b.dot(normal)),
                    (c, c.dot(normal)),
                    (d, d.dot(normal)),
                ];

                find_support_feature_from_pairs(&pairs)
            }
            math::Shape::Polygon(polygon) => {
                let verts = match rot_mat {
                    Some(components::RotationMatrix { rot_mat: rm }) => &polygon.apply_mat2x3(rm),
                    None => &polygon.verts,
                };

                let n_sides = verts.len();

                let mut best_idx = 0;
                let mut best_dot = verts[0].dot(normal);

                for i in 1..n_sides {
                    let dot = verts[i].dot(normal);
                    if dot > best_dot {
                        best_dot = dot;
                        best_idx = i;
                    }
                }

                let prev_idx = (best_idx + n_sides - 1) % n_sides;
                let next_idx = (best_idx + 1) % n_sides;

                let prev_dot = verts[prev_idx].dot(normal);
                let next_dot = verts[next_idx].dot(normal);

                let (second_idx, second_dot) = if prev_dot >= next_dot {
                    (prev_idx, prev_dot)
                } else {
                    (next_idx, next_dot)
                };

                if best_dot - second_dot <= FEATURE_MARGIN {
                    Feature::Edge(math::Segment::new_unchecked(verts[best_idx], verts[second_idx]))
                } else {
                    Feature::Vertex(verts[best_idx])
                }
            }
            math::Shape::Circle(_) => unimplemented!(),
        }
    }

    fn project_vertex_on_segment(vertex: math::Vec2, segment: &math::Segment, to_clamp: bool) -> math::Vec2 {
        let edge = segment.get_vec2();
        let edge_len_sq = edge.square_mag();

        // project the vertex on the edge and clamp it
        if edge_len_sq == 0.0 {
            segment.a
        } else {
            let mut t = edge.dot(vertex.sub(segment.a)) / edge_len_sq;
            if to_clamp {
                t = t.clamp(0.0, 1.0);
            }
            segment.a.add(edge.scale(t))
        }
    }

    fn midpoint_of_min_dist_between_segments(segment_1: &math::Segment, segment_2: &math::Segment) -> math::Vec2 {
        let proj_1_a = project_vertex_on_segment(segment_1.a, segment_2, true);
        let proj_1_b = project_vertex_on_segment(segment_1.b, segment_2, true);
        let proj_2_a = project_vertex_on_segment(segment_2.a, segment_1, true);
        let proj_2_b = project_vertex_on_segment(segment_2.b, segment_1, true);

        let square_dist_1_a = segment_1.a.square_dist(proj_1_a);
        let square_dist_1_b = segment_1.b.square_dist(proj_1_b);
        let square_dist_2_a = segment_2.a.square_dist(proj_2_a);
        let square_dist_2_b = segment_2.b.square_dist(proj_2_b);

        let mut min_square_dist = square_dist_1_a;
        let mut best_on_1 = segment_1.a;
        let mut best_on_2 = proj_1_a;

        if square_dist_1_b < min_square_dist {
            min_square_dist = square_dist_1_b;
            best_on_1 = segment_1.b;
            best_on_2 = proj_1_b;
        }

        if square_dist_2_a < min_square_dist {
            min_square_dist = square_dist_2_a;
            best_on_1 = proj_2_a;
            best_on_2 = segment_2.a;
        }

        if square_dist_2_b < min_square_dist {
            best_on_1 = proj_2_b;
            best_on_2 = segment_2.b;
        }

        best_on_1.midpoint(best_on_2)
    }

    fn midpoint_of_min_dist_between_parallel_non_overlapping_segments(
        segment_1: &math::Segment,
        segment_2: &math::Segment,
    ) -> math::Vec2 {
        let square_dist_a_a = segment_1.a.square_dist(segment_2.a);
        let square_dist_a_b = segment_1.a.square_dist(segment_2.b);
        let square_dist_b_a = segment_1.b.square_dist(segment_2.a);
        let square_dist_b_b = segment_1.b.square_dist(segment_2.b);

        let mut min_square_dist = square_dist_a_a;
        let mut best_on_1 = segment_1.a;
        let mut best_on_2 = segment_2.a;

        if square_dist_a_b < min_square_dist {
            min_square_dist = square_dist_a_b;
            best_on_1 = segment_1.a;
            best_on_2 = segment_2.b;
        }

        if square_dist_b_a < min_square_dist {
            min_square_dist = square_dist_b_a;
            best_on_1 = segment_1.b;
            best_on_2 = segment_2.a;
        }

        if square_dist_b_b < min_square_dist {
            best_on_1 = segment_1.b;
            best_on_2 = segment_2.b;
        }

        best_on_1.midpoint(best_on_2)
    }

    fn check_overlap_and_find_midpoint(segment_1: &math::Segment, segment_2: &math::Segment) -> Option<math::Vec2> {
        let (a, b) = (segment_1.a, segment_1.b);
        let (c, d) = (segment_2.a, segment_2.b);
        let (c_proj, d_proj) = (
            project_vertex_on_segment(c, segment_1, false),
            project_vertex_on_segment(d, segment_1, false),
        );

        let mut pairs = [(a, a), (b, b), (c, c_proj), (d, d_proj)];

        if (a.x - b.x).abs() > EPS {
            // non-vertical lines
            let overlap_start_x = (a.x.min(b.x)).max(c_proj.x.min(d_proj.x));
            let overlap_end_x = (a.x.max(b.x)).min(c_proj.x.max(d_proj.x));

            if overlap_start_x > overlap_end_x + EPS {
                // segments dont overlap
                return None;
            }

            // sort points by x
            pairs.sort_by(|p1, p2| p1.1.x.partial_cmp(&p2.1.x).unwrap());
        } else {
            // vertical lines
            let overlap_start_y = (a.y.min(b.y)).max(c_proj.y.min(d_proj.y));
            let overlap_end_y = (a.y.max(b.y)).min(c_proj.y.max(d_proj.y));

            if overlap_start_y > overlap_end_y + EPS {
                // segments dont overlap
                return None;
            }

            // sort points by y
            pairs.sort_by(|p1, p2| p1.1.y.partial_cmp(&p2.1.y).unwrap());
        };

        let overlap_start_midpoint = pairs[1].0.midpoint(pairs[1].1);
        let overlap_end_midpoint = pairs[2].0.midpoint(pairs[2].1);
        Some(overlap_start_midpoint.midpoint(overlap_end_midpoint))
    }

    let global_pos_1 = match lin_vel_1 {
        Some(v) => pos_1.add(v),
        None => pos_1,
    };

    let global_pos_2 = match lin_vel_2 {
        Some(v) => pos_2.add(v),
        None => pos_2,
    };

    let global_rot_mat_1: Option<&components::RotationMatrix> = match (rot_mat_1, ang_vel_1) {
        (Some(rm), Some(av)) => Some(&rm.update(math::Radians(av), rm.rot_mat.pre_mul_vec2(body_1.centroid))),
        (Some(rm), None) => Some(rm),
        (None, Some(_)) => panic!("ang_vel exists but there is no rot_mat"),
        (None, None) => None,
    };

    let global_rot_mat_2: Option<&components::RotationMatrix> = match (rot_mat_2, ang_vel_2) {
        (Some(rm), Some(av)) => Some(&rm.update(math::Radians(av), rm.rot_mat.pre_mul_vec2(body_2.centroid))),
        (Some(rm), None) => Some(rm),
        (None, Some(_)) => panic!("ang_vel exists but there is no rot_mat"),
        (None, None) => None,
    };

    // find support vertices
    let mut support_1 = find_support_feature(normal, global_rot_mat_1, &body_1.shape);
    let mut support_2 = find_support_feature(normal.neg(), global_rot_mat_2, &body_2.shape);

    support_1.apply_vec2_mut(global_pos_1);
    support_2.apply_vec2_mut(global_pos_2);

    match (support_1, support_2) {
        (Feature::Vertex(vertex_1), Feature::Vertex(vertex_2)) => vertex_1.midpoint(vertex_2),
        (Feature::Vertex(vertex), Feature::Edge(segment)) | (Feature::Edge(segment), Feature::Vertex(vertex)) => {
            let proj = project_vertex_on_segment(vertex, &segment, true);

            // take midpoint between projection and vertex
            vertex.midpoint(proj)
        }
        (Feature::Edge(segment_1), Feature::Edge(segment_2)) => {
            let edge_1 = segment_1.get_vec2();
            let edge_2 = segment_2.get_vec2();

            let denominator = edge_1.cross(edge_2);

            if denominator.abs() >= PARALLEL_EPS {
                // edges are not parallel

                let delta = segment_2.a.sub(segment_1.a);

                // intersection is at:
                // start_1 + edge_1 * t = start_2 + edge_2 * u
                //
                // rearranging:
                // edge_1 * t = (start_2 - start_1) + edge_2 * u
                // edge_1 * t = delta + edge_2 * u
                //
                // we know that cross(v, v) = 0
                // so if we cross both sides with edge_2, we kill the u term
                //
                // 1) cross both sides with edge_2 and solve for t
                // cross(edge_1 * t, edge_2) = cross(delta + edge_2 * u, edge_2)
                // t * cross(edge_1, edge_2) = cross(delta, edge_2) + u * cross(edge_2, edge_2) <-- but this is 0, so
                // t * cross(edge_1, edge_2) = cross(delta, edge_2)
                // t = cross(delta, edge_2) / cross(edge_1, edge_2)
                //
                // 2) do the same solving for u
                // u = cross(delta, edge_1) / cross(edge_1, edge_2)
                //
                // so if we define:
                // denominator = cross(edge_1, edge_2)
                //
                // we have:
                // t = cross(delta, edge_2) / denominator
                // u = cross(delta, edge_1) / denominator

                let t = delta.cross(edge_2) / denominator;
                let u = delta.cross(edge_1) / denominator;

                if -EPS <= t && t <= 1.0 + EPS && -EPS <= u && u <= 1.0 + EPS {
                    // intersection is on both segments, return intersection
                    segment_1.a.add(edge_1.scale(t))
                } else {
                    // intersection is not on both segments
                    midpoint_of_min_dist_between_segments(&segment_1, &segment_2)
                }
            } else {
                // edges are parallel
                let overlap_midpoint = check_overlap_and_find_midpoint(&segment_1, &segment_2);

                match overlap_midpoint {
                    Some(midpoint) => midpoint,
                    None => midpoint_of_min_dist_between_parallel_non_overlapping_segments(&segment_1, &segment_2),
                }
            }
        }
    }
}

/// updates 2 entities' linear velocity vector after they collide
fn compute_reaction(
    normal: math::Vec2,
    contact_point: math::Vec2,
    mass_center_1: math::Vec2,
    mass_center_2: math::Vec2,
    mut translation_1: Option<&mut components::Translation>,
    mut translation_2: Option<&mut components::Translation>,
    mut rotation_1: Option<&mut components::Rotation>,
    mut rotation_2: Option<&mut components::Rotation>,
    surface_1: &components::Surface,
    surface_2: &components::Surface,
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

        let normal_inv_mass_inertia = inv_mass_1
            + inv_mass_2
            + inv_inertia_1 * math::pow2(arm_1.cross(normal))
            + inv_inertia_2 * math::pow2(arm_2.cross(normal));

        // relative velocity from shape_1 to shape_2, vector from vel_1 to vel_2
        let rel_vel = vel_2.sub(vel_1);
        // normal_rel_vel_mag is basically rel_vel projected on the normal axis
        // remember that normal is the unit vector perpendicular to the edge with minimum overlap
        let normal_rel_vel_mag = rel_vel.dot(normal);

        if normal_rel_vel_mag >= EPS {
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

        if tangent_rel_vel_mag < EPS {
            // no tangential slip, so nothing to correct
            return;
        }

        // tangent_unit is tangent_rel_lin_vel normalized
        let tangent = tangent_rel_vel.scale(1.0 / tangent_rel_vel_mag); // I am not using .norm() because I've already computed the magnitude

        let tangent_inv_mass_inertia = inv_mass_1
            + inv_mass_2
            + inv_inertia_1 * math::pow2(arm_1.cross(tangent))
            + inv_inertia_2 * math::pow2(arm_2.cross(tangent));

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

        if let Some(&components::Rotation { ang_vel, .. }) = world.engine.rotation.get(entity) {
            // entity can rotate
            if ang_vel.abs() <= EPS {
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
            let mut hitbox_1 = Some(compute_hitbox(state_1, pos_1, rot_mat_1, lin_vel_1, ang_vel_1, body_1));

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
                    hitbox_1 = Some(compute_hitbox(state_1, pos_1, rot_mat_1, lin_vel_1, ang_vel_1, body_1));
                }

                let hitbox_2 = compute_hitbox(state_2, pos_2, rot_mat_2, lin_vel_2, ang_vel_2, body_2);

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
                    swept_shape_1 = Some(compute_swept_shape(
                        state_1, pos_1, rot_mat_1, lin_vel_1, ang_vel_1, body_1,
                    ));
                }

                let swept_shape_2 = compute_swept_shape(state_2, pos_2, rot_mat_2, lin_vel_2, ang_vel_2, body_2);

                // println!("{entity_1}-{entity_2} checking swept shapes...");
                let Some(normal) = check_sat(swept_shape_1.as_ref().unwrap(), &swept_shape_2) else {
                    // swept shapes are not colliding, no need to compute reaction or invalidate cache
                    // println!("  -> swept shapes NOT colliding");
                    continue 'loop_2;
                };
                // println!("  -> swept shapes ARE colliding");

                // compute contact point and centers of mass
                let contact_point = compute_contact_point(
                    normal, pos_1, pos_2, rot_mat_1, rot_mat_2, lin_vel_1, lin_vel_2, ang_vel_1, ang_vel_2, body_1,
                    body_2,
                );

                let mass_center_1 = match rot_mat_1 {
                    Some(rot_mat) => rot_mat
                        .rot_mat
                        .pre_mul_vec2(body_1.centroid())
                        .add(pos_1)
                        .add(lin_vel_1.unwrap_or(math::Vec2::zero())),
                    None => body_1
                        .centroid()
                        .add(pos_1)
                        .add(lin_vel_1.unwrap_or(math::Vec2::zero())),
                };
                let mass_center_2 = match rot_mat_2 {
                    Some(rot_mat) => rot_mat
                        .rot_mat
                        .pre_mul_vec2(body_2.centroid())
                        .add(pos_2)
                        .add(lin_vel_2.unwrap_or(math::Vec2::zero())),
                    None => body_2
                        .centroid()
                        .add(pos_2)
                        .add(lin_vel_2.unwrap_or(math::Vec2::zero())),
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
