use crate::math::ApplyTransformationVerts;
use crate::{ecs, math};

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
pub fn compute_contact_point(
    normal: math::Vec2,
    pos_1: math::Vec2,
    pos_2: math::Vec2,
    rot_mat_1: Option<&ecs::RotationMatrix>,
    rot_mat_2: Option<&ecs::RotationMatrix>,
    lin_vel_1: Option<math::Vec2>,
    lin_vel_2: Option<math::Vec2>,
    ang_vel_1: Option<f32>,
    ang_vel_2: Option<f32>,
    body_1: &ecs::Body,
    body_2: &ecs::Body,
) -> math::Vec2 {
    const FEATURE_MARGIN: f32 = 0.05;
    const PARALLEL_EPS: f32 = 0.02;

    fn find_support_feature(normal: math::Vec2, rot_mat: Option<&ecs::RotationMatrix>, shape: &math::Shape) -> Feature {
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
                    Some(ecs::RotationMatrix { rot_mat: rm }) => segment.apply_mat2x3(rm),
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
                    Some(ecs::RotationMatrix { rot_mat: rm }) => triangle.apply_mat2x3(rm),
                    None => [triangle.a, triangle.b, triangle.c],
                };

                let pairs = [(a, a.dot(normal)), (b, b.dot(normal)), (c, c.dot(normal))];

                find_support_feature_from_pairs(&pairs)
            }
            math::Shape::Quad(quad) => {
                let [a, b, c, d] = match rot_mat {
                    Some(ecs::RotationMatrix { rot_mat: rm }) => quad.apply_mat2x3(rm),
                    None => [quad.a, quad.b, quad.c, quad.d],
                };

                let pairs = [(a, a.dot(normal)), (b, b.dot(normal)), (c, c.dot(normal)), (d, d.dot(normal))];

                find_support_feature_from_pairs(&pairs)
            }
            math::Shape::Polygon(polygon) => {
                let verts = match rot_mat {
                    Some(ecs::RotationMatrix { rot_mat: rm }) => &polygon.apply_mat2x3(rm),
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

    fn midpoint_of_min_dist_between_parallel_non_overlapping_segments(segment_1: &math::Segment, segment_2: &math::Segment) -> math::Vec2 {
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

        if (a.x - b.x).abs() > math::EPS {
            // non-vertical lines
            let overlap_start_x = (a.x.min(b.x)).max(c_proj.x.min(d_proj.x));
            let overlap_end_x = (a.x.max(b.x)).min(c_proj.x.max(d_proj.x));

            if overlap_start_x > overlap_end_x + math::EPS {
                // segments dont overlap
                return None;
            }

            // sort points by x
            pairs.sort_by(|p1, p2| p1.1.x.partial_cmp(&p2.1.x).unwrap());
        } else {
            // vertical lines
            let overlap_start_y = (a.y.min(b.y)).max(c_proj.y.min(d_proj.y));
            let overlap_end_y = (a.y.max(b.y)).min(c_proj.y.max(d_proj.y));

            if overlap_start_y > overlap_end_y + math::EPS {
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

    let global_rot_mat_1: Option<&ecs::RotationMatrix> = match (rot_mat_1, ang_vel_1) {
        (Some(rm), Some(av)) => Some(&rm.update(math::Radians(av), rm.rot_mat.pre_mul_vec2(body_1.centroid))),
        (Some(rm), None) => Some(rm),
        (None, Some(_)) => panic!("ang_vel exists but there is no rot_mat"),
        (None, None) => None,
    };

    let global_rot_mat_2: Option<&ecs::RotationMatrix> = match (rot_mat_2, ang_vel_2) {
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

                if -math::EPS <= t && t <= 1.0 + math::EPS && -math::EPS <= u && u <= 1.0 + math::EPS {
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
