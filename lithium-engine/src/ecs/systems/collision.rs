use std::{collections::HashSet, panic};

use crate::{
    ecs::{
        components, entities,
        systems::physics::{EPS, pow2},
    },
    world::World,
};

pub fn disc_rect_rect(
    pos_1: components::Vec2,
    shape_1: components::Rect,
    pos_2: components::Vec2,
    shape_2: components::Rect,
) -> Option<components::Angle> {
    let components::Rect {
        width: width_1,
        height: height_1,
    } = shape_1;
    let components::Rect {
        width: width_2,
        height: height_2,
    } = shape_2;

    let (left_1, right_1) = (pos_1.x, pos_1.x + width_1);
    let (top_1, bottom_1) = (pos_1.y, pos_1.y + height_1);

    let (left_2, right_2) = (pos_2.x, pos_2.x + width_2);
    let (top_2, bottom_2) = (pos_2.y, pos_2.y + height_2);

    if !(left_1 >= right_2 || left_2 >= right_1 || top_1 >= bottom_2 || top_2 >= bottom_1) {
        // it is colliding
        let left_overlap = right_1 - left_2;
        let right_overlap = right_2 - left_1;
        let top_overlap = bottom_1 - top_2;
        let bottom_overlap = bottom_2 - top_1;

        Some(if right_overlap.min(left_overlap) < bottom_overlap.min(top_overlap) {
            if right_overlap < left_overlap {
                components::Angle { radians: 0.0 }
            } else {
                components::Angle {
                    radians: std::f32::consts::PI,
                }
            }
        } else {
            if bottom_overlap < top_overlap {
                components::Angle {
                    radians: 3.0 * std::f32::consts::FRAC_PI_2,
                }
            } else {
                components::Angle {
                    radians: std::f32::consts::FRAC_PI_2,
                }
            }
        })
    } else {
        // since is not colliding, return None
        None
    }
}

pub fn disc_circle_circle(
    pos_1: components::Vec2,
    shape_1: components::Circle,
    pos_2: components::Vec2,
    shape_2: components::Circle,
) -> Option<components::Angle> {
    let radius_1 = shape_1.radius;
    let radius_2 = shape_2.radius;

    let delta_x = (pos_2.x + radius_2) - (pos_1.x + radius_1); // difference between centres x
    let delta_y = (pos_2.y + radius_2) - (pos_1.y + radius_1); // difference between centres y

    let radius_sum = radius_1 + radius_2;

    if delta_x * delta_x + delta_y * delta_y <= radius_sum * radius_sum {
        // compute square distance to improve performance
        // since it is colliding, compute angle
        Some(
            components::Angle {
                radians: delta_y.atan2(delta_x),
            }
            .norm(),
        )
    } else {
        // since is not colliding, return None
        None
    }
}

pub fn disc_circle_rect(
    pos_circle: components::Vec2,
    shape_circle: components::Circle,
    pos_rect: components::Vec2,
    shape_rect: components::Rect,
    from_circle_to_rect: bool,
) -> Option<components::Angle> {
    let radius = shape_circle.radius;
    let components::Rect { width, height } = shape_rect;

    let centre_x = pos_circle.x + radius;
    let centre_y = pos_circle.y + radius;

    let (left, right) = (pos_rect.x, pos_rect.x + width);
    let (top, bottom) = (pos_rect.y, pos_rect.y + height);

    let delta_left = centre_x - left;
    let delta_right = right - centre_x;
    let delta_top = centre_y - top;
    let delta_bottom = bottom - centre_y;

    let centre_outside = centre_x < left || centre_x > right || centre_y < top || centre_y > bottom;

    let (closest_x, closest_y) = if centre_outside {
        // centre is outside the rectangle, use standard clamp
        (centre_x.clamp(left, right), centre_y.clamp(top, bottom))
    } else {
        // centre is inside the rectangle, project to nearest edge
        if delta_left.min(delta_right) < delta_top.min(delta_bottom) {
            // closer horizontally
            let x = if delta_left < delta_right { left } else { right };
            (x, centre_y)
        } else {
            // closer vertically
            let y = if delta_top < delta_bottom { top } else { bottom };
            (centre_x, y)
        }
    };

    let (delta_x, delta_y) = if from_circle_to_rect {
        (closest_x - centre_x, closest_y - centre_y)
    } else {
        (centre_x - closest_x, centre_y - closest_y)
    };

    if !centre_outside || delta_x * delta_x + delta_y * delta_y <= radius * radius {
        // compute square distance to improve performance
        // since it is colliding, compute angle
        Some(
            components::Angle {
                radians: delta_y.atan2(delta_x),
            }
            .norm(),
        )
    } else {
        // since is not colliding, return None
        None
    }
}

pub fn check_disc_coll(
    pos_1: components::Vec2,
    shape_1: &components::Shape,
    pos_2: components::Vec2,
    shape_2: &components::Shape,
) -> Option<components::Angle> {
    match shape_1 {
        components::Shape::Rect(rect_1) => {
            match shape_2 {
                components::Shape::Rect(rect_2) => {
                    // rect-rect collision
                    disc_rect_rect(pos_1, *rect_1, pos_2, *rect_2)
                }
                components::Shape::Circle(circle_2) => {
                    // rect-circle collision
                    disc_circle_rect(pos_2, *circle_2, pos_1, *rect_1, false)
                }
            }
        }
        components::Shape::Circle(circle_1) => {
            match shape_2 {
                components::Shape::Rect(rect_2) => {
                    // circle-rect collision
                    disc_circle_rect(pos_1, *circle_1, pos_2, *rect_2, true)
                }
                components::Shape::Circle(circle_2) => {
                    // circle-circle collision
                    disc_circle_circle(pos_1, *circle_1, pos_2, *circle_2)
                }
            }
        }
    }
}

pub fn cont_line_line(
    line_1: &components::LineCache,
    line_2: &components::LineCache,
) -> Option<(components::Vec2, components::Angle)> {
    unimplemented!()
}

/// uses Liang–Barsky algorithm to determine which side of the rectangle was hit by the line, then computes the angle of collision
pub fn cont_line_rect(
    line: &components::LineCache,
    pos_rect: components::Vec2,
    shape_rect: components::Rect,
) -> Option<(components::Vec2, components::Angle)> {
    let (left, right, top, bottom) = (
        pos_rect.x,
        pos_rect.x + shape_rect.width,
        pos_rect.y,
        pos_rect.y + shape_rect.height,
    );

    // line starts from inside the rect
    if line.a.x > left && line.a.x < right && line.a.y > top && line.a.y < bottom {
        // resolve exit
        println!("line starts inside the rectangle");
        let left_overlap = line.a.x - left;
        let right_overlap = right - line.a.x;
        let top_overlap = line.a.y - top;
        let bottom_overlap = bottom - line.a.y;

        return Some(if right_overlap.min(left_overlap) < bottom_overlap.min(top_overlap) {
            if right_overlap < left_overlap {
                (
                    components::Vec2::new(left, line.a.y),
                    components::Angle { radians: 0.0 },
                )
            } else {
                (
                    components::Vec2::new(right, line.a.y),
                    components::Angle {
                        radians: std::f32::consts::PI,
                    },
                )
            }
        } else {
            if bottom_overlap < top_overlap {
                (
                    components::Vec2::new(line.a.x, top),
                    components::Angle {
                        radians: 3.0 * std::f32::consts::FRAC_PI_2,
                    },
                )
            } else {
                (
                    components::Vec2::new(line.a.x, bottom),
                    components::Angle {
                        radians: std::f32::consts::FRAC_PI_2,
                    },
                )
            }
        });
    }

    let mut t_enter = 0.0; // start of the line
    let mut t_exit: f32 = 1.0; // end of the line
    let mut side_id = 0; // 0 -> no collision; 1 -> left; 2 -> right; 3 -> top; 4 -> bottom

    let inv_delta_x = if line.delta_x.abs() > EPS {
        Some(1.0 / line.delta_x)
    } else {
        None
    };
    let inv_delta_y = if line.delta_y.abs() > EPS {
        Some(1.0 / line.delta_y)
    } else {
        None
    };

    #[inline]
    fn clip(
        candidate_t: f32,
        entering: bool,
        this_side_id: u8,
        t_entry: &mut f32,
        t_exit: &mut f32,
        side_id: &mut u8,
    ) -> bool {
        if entering {
            // entering edges raise the lower bound
            if candidate_t > *t_exit + EPS {
                return false;
            } // entry after exit ⇒ reject
            if candidate_t > *t_entry || (*side_id == 0 && (candidate_t - *t_entry).abs() <= EPS) {
                // tighten entry and remember side
                *t_entry = candidate_t;
                *side_id = this_side_id;
            }
        } else {
            // exiting edges lower the upper bound
            if candidate_t < *t_entry - EPS {
                return false;
            } // exit before entry ⇒ reject
            if candidate_t < *t_exit {
                // tighten exit
                *t_exit = candidate_t;
            }
        }
        true
    }

    // x-axis constraints (left and right edges)
    if let Some(inv_dx) = inv_delta_x {
        // meeting t for left edge; entering if we move right -> left (delta_x < 0)
        let t_left: f32 = (left - line.a.x) * inv_dx; // solves (-delta_x)*t <= point_a.x-left
        if !clip(t_left, line.delta_x > 0.0, 1, &mut t_enter, &mut t_exit, &mut side_id) {
            return None;
        }
        // meeting t for right edge; entering if we move left -> right (delta_x > 0)
        let t_right: f32 = (right - line.a.x) * inv_dx; // solves ( delta_x)*t <= right-point_a.x
        if !clip(t_right, line.delta_x < 0.0, 2, &mut t_enter, &mut t_exit, &mut side_id) {
            return None;
        }
    } else {
        // segment is parallel to vertical edges; reject if x is outside the slab
        if line.a.x < left - EPS || line.a.x > right + EPS {
            return None;
        }
    }

    // y-axis constraints (top and bottom edges)
    if let Some(inv_dy) = inv_delta_y {
        // meeting t for top edge; entering if we move down→up (delta_y < 0)
        let t_top: f32 = (top - line.a.y) * inv_dy; // solves (-delta_y)*t <= point_a.y-top
        if !clip(t_top, line.delta_y > 0.0, 3, &mut t_enter, &mut t_exit, &mut side_id) {
            return None;
        }
        // meeting t for bottom edge; entering if we move up→down (delta_y > 0)
        let t_bottom: f32 = (bottom - line.a.y) * inv_dy; // solves ( delta_y)*t <= bottom-point_a.y
        if !clip(t_bottom, line.delta_y < 0.0, 4, &mut t_enter, &mut t_exit, &mut side_id) {
            return None;
        }
    } else {
        // segment is parallel to horizontal edges; reject if y is outside the slab
        if line.a.y < top - EPS || line.a.y > bottom + EPS {
            return None;
        }
    }

    if side_id == 0 || t_enter > t_exit + EPS {
        return None;
    }

    // compute cut position
    let contact_point = components::Vec2::new(line.a.x + t_enter * line.delta_x, line.a.y + t_enter * line.delta_y);

    // match side to get angle
    let contact_angle = match side_id {
        1 => components::Angle { radians: 0.0 },
        2 => components::Angle {
            radians: std::f32::consts::PI,
        },
        3 => components::Angle {
            radians: std::f32::consts::FRAC_PI_2,
        },
        4 => components::Angle {
            radians: 3.0 * std::f32::consts::FRAC_PI_2,
        },
        _ => panic!("non-existing side"),
    };

    Some((contact_point, contact_angle))
}

pub fn cont_line_circle(
    line: &components::LineCache,
    pos_circle: components::Vec2,
    shape_circle: components::Circle,
) -> Option<(components::Vec2, components::Angle)> {
    let square_dist = line.circle_square_dist(pos_circle, shape_circle);
    let square_radius = pow2(shape_circle.radius);

    if square_dist > square_radius {
        // the line is not cutting the circle
        None
    } else {
        // the line is cutting the circle

        // temporary store the circle's centre position, because all the formulas assume circle is centered
        // so we need to subtract it from the line equation
        let cx = pos_circle.x + shape_circle.radius;
        let cy = pos_circle.y + shape_circle.radius;

        // check if the line is vertical (or almost vertical)
        let circum = if line.delta_x.abs() <= EPS {
            // the line is vertical

            // just compute the y, since the x will be the same as the line
            let circum_y = (square_radius - pow2(line.a.x - cx)).sqrt();

            // choose the solution based on the direction of the line
            let circum_y = if line.delta_y >= 0.0 { circum_y } else { -circum_y };

            components::Vec2::new(line.a.x, circum_y + cy)
        } else {
            // the line is not vertical

            // compute the line equation to solve the system with the circle
            let m = line.m.expect("line shouldn't be vertical here");
            let d = (-m).mul_add(line.a.x - cx, line.a.y - cy);

            // solve the equation
            let eq_a = 1.0 + pow2(m);
            let eq_b = 2.0 * m * d;
            let eq_c = pow2(d) - square_radius;
            let eq_delta = eq_b.mul_add(eq_b, -4.0 * eq_a * eq_c);

            let e = 1.0 / (2.0 * eq_a);

            // compute an eps to accept as 0
            let eps = 1e-6 * (eq_a.abs() + eq_b.abs() + eq_c.abs()).max(1.0);

            if eq_delta < -eps {
                // this should never happen
                return None;
            } else if eq_delta <= 0.0 {
                // tangent case, only 1 solution
                let circum_x = -eq_b * e;

                // put x into the line to get y
                let circum_y = m.mul_add(circum_x, d);

                // compute intersection position, summing again the circle's centre position
                components::Vec2::new(circum_x + cx, circum_y + cy)
            } else {
                // normal case
                let eq_delta_sqrt = eq_delta.sqrt();

                let circum_x_1 = (-eq_b + eq_delta_sqrt) * e;
                let circum_x_2 = (-eq_b - eq_delta_sqrt) * e;

                // put x into the line to get y
                let circum_y_1 = m.mul_add(circum_x_1, d);
                let circum_y_2 = m.mul_add(circum_x_2, d);

                // compute intersections positions, summing again the circle's centre position
                let circum_1 = components::Vec2::new(circum_x_1 + cx, circum_y_1 + cy);
                let circum_2 = components::Vec2::new(circum_x_2 + cx, circum_y_2 + cy);

                // compute square distance from A
                let square_dist_1 = circum_1.square_dist(line.a);
                let square_dist_2 = circum_2.square_dist(line.a);

                // take the smaller distance
                if square_dist_1 <= square_dist_2 {
                    circum_1
                } else {
                    circum_2
                }
            }
        };

        let delta_x = cx - circum.x; // difference between centres x
        let delta_y = cy - circum.y; // difference between centres y

        Some((
            circum,
            components::Angle {
                radians: delta_y.atan2(delta_x),
            }
            .norm(),
        ))
    }
}

/// continuosly checks if a given object is colliding with another object
pub fn check_cont_coll(
    line_1: &components::LineCache,
    pos_2: components::Vec2,
    shape_2: &components::Shape,
) -> Option<(components::Vec2, components::Angle)> {
    if line_1.delta_x == 0.0 && line_1.delta_y == 0.0 {
        return None; // null movement line
    }

    match shape_2 {
        components::Shape::Rect(rect_2) => cont_line_rect(line_1, pos_2, *rect_2),
        components::Shape::Circle(circle_2) => cont_line_circle(line_1, pos_2, *circle_2),
    }
}

/// uses both continuos and discrete collision detection algorithms to detect all the collisions in the world
/// returns a vector of tuple containing the ids of the 2 colliding entities and the angle of collision
pub fn detect_collisions(world: &mut World) -> Vec<(entities::Entity, entities::Entity, components::Angle)> {
    let mut collisions = Vec::new();
    let mut disc_checks = HashSet::new();

    // iterate over entities with velocity because the other ones won't move
    'vel_entities: for (entity_1, vel_1) in world.vel.iter() {
        // check if entity_1 has position and shape
        let (Some(pos_1), Some(shape_1)) = (world.pos.get(entity_1), world.shape.get(entity_1)) else {
            continue 'vel_entities;
        };

        // compute next update position for entity_1
        let new_pos_1 = components::Vec2 {
            x: pos_1.x + vel_1.x,
            y: pos_1.y + vel_1.y,
        };

        // compute movement line for entity_1 from the center of the shape
        let line_1 = match shape_1 {
            components::Shape::Circle(components::Circle { radius }) => components::LineCache::new(
                new_pos_1.add_scalar(*radius, *radius),
                (*pos_1).add_scalar(*radius, *radius),
            ),
            components::Shape::Rect(components::Rect { width, height }) => components::LineCache::new(
                new_pos_1.add_scalar(width / 2.0, height / 2.0),
                (*pos_1).add_scalar(width / 2.0, height / 2.0),
            ),
        };

        let mut cont_colls = Vec::new();

        // iterate over the other entities with a shape
        'shape_entities: for (entity_2, shape_2) in world.shape.iter() {
            // avoid checking collision with self
            if entity_1 == entity_2 {
                continue 'shape_entities;
            };

            // check if entity_2 has position
            let Some(pos_2) = world.pos.get(entity_2) else {
                continue 'shape_entities;
            };

            // compute next update position for entity_2 if it has a velocity
            let new_pos_2 = if let Some(vel_2) = world.vel.get(entity_2) {
                components::Vec2 {
                    x: pos_2.x + vel_2.x,
                    y: pos_2.y + vel_2.y,
                }
            } else {
                *pos_2
            };

            // compute movement line for entity_2 from the center of the shape
            // let line_2 = match shape_2 {
            //     &components::Shape::Circle(components::Circle { radius }) => components::Line::new(
            //         new_pos_2.add_scalar(radius, radius),
            //         (*pos_2).add_scalar(radius, radius),
            //     ),
            //     _ => unimplemented!(),
            // };

            // check continuos collision
            if let Some((pos, angle)) = check_cont_coll(&line_1, new_pos_2, shape_2) {
                // it will collide with something
                cont_colls.push((entity_2, pos, angle));
            };
        }

        // check if there were continuos collisions
        if cont_colls.len() > 0 {
            // extract the collision with closer distance from A, so the first collision done by the movement line
            let target = line_1.a;

            if let Some((cut_entity, _pos, angle)) = cont_colls
                .iter()
                .min_by(|(_, p1, _), (_, p2, _)| p1.square_dist(target).total_cmp(&p2.square_dist(target)))
            {
                collisions.push((entity_1, *cut_entity, *angle));
                // println!("found cont, using that")
            }
        } else {
            // no continuos collisions, use discrete collision detection

            // iterate over the other entities with a shape
            'shape_entities: for (entity_2, shape_2) in world.shape.iter() {
                // avoid checking collision with self
                if entity_1 == entity_2 {
                    continue 'shape_entities;
                };

                // avoid rechecking the same pairs
                let pair = (entity_1.min(entity_2), entity_1.max(entity_2));

                if disc_checks.contains(&pair) {
                    continue 'shape_entities;
                }
                disc_checks.insert(pair);

                // check if entity_2 has position
                let Some(pos_2) = world.pos.get(entity_2) else {
                    continue 'shape_entities;
                };

                // compute next update position also for entity_2 if it has a velocity
                let new_pos_2 = if let Some(vel_2) = world.vel.get(entity_2) {
                    components::Vec2 {
                        x: pos_2.x + vel_2.x,
                        y: pos_2.y + vel_2.y,
                    }
                } else {
                    *pos_2
                };

                // check discrete collision
                if let Some(angle) = check_disc_coll(new_pos_1, shape_1, new_pos_2, shape_2) {
                    // it will collide with something
                    collisions.push((entity_1, entity_2, angle));
                    // println!("found no cont, using disc")
                };
            }
        }
    }
    collisions
}

pub fn compute_collisions(world: &mut World, collisions: Vec<(entities::Entity, entities::Entity, components::Angle)>) {
    for (entity_1, entity_2, angle) in collisions {
        // update rest
        let rest_delta = 0.3;
        let rest_eps = 0.6;

        if world.rest.get(entity_1).is_some()
            && angle.radians >= std::f32::consts::FRAC_PI_2 - rest_delta
            && angle.radians <= std::f32::consts::FRAC_PI_2 + rest_delta
        {
            world.rest.set(entity_1, true);
        } else if world.rest.get(entity_2).is_some()
            && angle.radians >= 3.0 * std::f32::consts::FRAC_PI_2 - rest_delta
            && angle.radians <= 3.0 * std::f32::consts::FRAC_PI_2 + rest_delta
        {
            world.rest.set(entity_2, true);
        }

        let normal_x = angle.radians.cos();
        let normal_y = angle.radians.sin();

        let tangent_x = -normal_y;
        let tangent_y = normal_x;

        let elast = world
            .elast
            .get(entity_1)
            .expect("missing restitution")
            .0
            .min(world.elast.get(entity_2).expect("missing restitution").0);

        let vel_1 = *world.vel.get(entity_1).expect("missing velocity for a dynamic object");

        let vel_1_normal = vel_1.x * normal_x + vel_1.y * normal_y;
        let vel_1_tangent = vel_1.x * tangent_x + vel_1.y * tangent_y;

        let vel_1_normal_post = if let Some(vel_2) = world.vel.get(entity_2) {
            // dynamic-dynamic collision
            let vel_2_normal = vel_2.x * normal_x + vel_2.y * normal_y;
            let vel_2_tangent = vel_2.x * tangent_x + vel_2.y * tangent_y;

            let mass_1 = world.mass.get(entity_1).expect("missing mass").0;
            let mass_2 = world.mass.get(entity_2).expect("missing mass").0;

            let vel_1_normal_post =
                (vel_1_normal * (mass_1 - elast * mass_2) + (1.0 + elast) * mass_2 * vel_2_normal) / (mass_1 + mass_2);
            let vel_2_normal_post =
                (vel_2_normal * (mass_2 - elast * mass_1) + (1.0 + elast) * mass_1 * vel_1_normal) / (mass_1 + mass_2);

            let new_vel_2 = components::Vec2 {
                x: mask_below_eps(
                    vel_2_normal_post * normal_x + vel_2_tangent * tangent_x,
                    vel_2.x,
                    rest_eps,
                ),
                y: mask_below_eps(
                    vel_2_normal_post * normal_y + vel_2_tangent * tangent_y,
                    vel_2.y,
                    rest_eps,
                ),
            };

            world.vel.set(entity_2, new_vel_2);
            vel_1_normal_post
        } else {
            // dynamic-static collision
            let vel_1_normal_post = -vel_1_normal * elast;
            vel_1_normal_post
        };

        let new_vel_1 = components::Vec2 {
            x: mask_below_eps(
                vel_1_normal_post * normal_x + vel_1_tangent * tangent_x,
                vel_1.x,
                rest_eps,
            ),
            y: mask_below_eps(
                vel_1_normal_post * normal_y + vel_1_tangent * tangent_y,
                vel_1.y,
                rest_eps,
            ),
        };

        world.vel.set(entity_1, new_vel_1);
    }
}

fn mask_below_eps(value: f32, check: f32, eps: f32) -> f32 {
    if check.abs() >= eps { value } else { 0.0 }
}
