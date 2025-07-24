use std::collections::HashSet;

use crate::{
    ecs::{components, entities},
    world::World,
};

pub fn check_rect_rect(
    pos_1: &components::Pos,
    shape_1: &components::Rect,
    pos_2: &components::Pos,
    shape_2: &components::Rect,
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
        let right_overlap = right_2 - left_1;
        let left_overlap = right_1 - left_2;
        let bottom_overlap = bottom_2 - top_1;
        let top_overlap = bottom_1 - top_2;

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

pub fn check_circle_circle(
    pos_1: &components::Pos,
    shape_1: &components::Circle,
    pos_2: &components::Pos,
    shape_2: &components::Circle,
) -> Option<components::Angle> {
    let radius_1 = shape_1.radius;
    let radius_2 = shape_2.radius;

    let delta_x = (pos_2.x + radius_2) - (pos_1.x + radius_1); // difference between centres x
    let delta_y = (pos_2.y + radius_2) - (pos_1.y + radius_1); // difference between centres y

    let radius_sum = radius_1 + radius_2;

    if delta_x * delta_x + delta_y * delta_y <= radius_sum * radius_sum {
        // compute square distance to improve performance
        // since it is colliding, compute angle
        Some(components::Angle {
            radians: delta_y.atan2(delta_x),
        })
    } else {
        // since is not colliding, return None
        None
    }
}

pub fn check_circle_rect(
    pos_circle: &components::Pos,
    shape_circle: &components::Circle,
    pos_rect: &components::Pos,
    shape_rect: &components::Rect,
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

    let (closest_x, closest_y) = if centre_x < left || centre_x > right || centre_y < top || centre_y > bottom {
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

    if delta_x * delta_x + delta_y * delta_y <= radius * radius {
        // compute square distance to improve performance
        // since it is colliding, compute angle
        Some(components::Angle {
            radians: delta_y.atan2(delta_x),
        })
    } else {
        // since is not colliding, return None
        None
    }
}

pub fn check_collision(
    pos_1: &components::Pos,
    shape_1: &components::Shape,
    pos_2: &components::Pos,
    shape_2: &components::Shape,
) -> Option<components::Angle> {
    match shape_1 {
        components::Shape::Rect(rect_1) => {
            match shape_2 {
                components::Shape::Rect(rect_2) => {
                    // rect-rect collision
                    check_rect_rect(pos_1, rect_1, pos_2, rect_2)
                }
                components::Shape::Circle(circle_2) => {
                    // rect-circle collision
                    check_circle_rect(pos_2, circle_2, pos_1, rect_1, false)
                }
            }
        }
        components::Shape::Circle(circle_1) => {
            match shape_2 {
                components::Shape::Rect(rect_2) => {
                    // circle-rect collision
                    check_circle_rect(pos_1, circle_1, pos_2, rect_2, true)
                }
                components::Shape::Circle(circle_2) => {
                    // circle-circle collision
                    check_circle_circle(pos_1, circle_1, pos_2, circle_2)
                }
            }
        }
    }
}

pub fn compute_collision(world: &mut World) -> Vec<(entities::Entity, entities::Entity, components::Angle)> {
    let mut collisions = Vec::new();
    let mut checks = HashSet::new();

    'vel_entities: for (entity_1, vel_1) in world.vel.iter() {
        // iterate over entities with velocity because the other ones won't move

        // check if entity_1 has position and shape
        let (Some(pos_1), Some(shape_1)) = (world.pos.get(entity_1), world.shape.get(entity_1)) else {
            continue 'vel_entities;
        };

        // compute next update position for entity_1
        let pos_1 = components::Pos {
            x: pos_1.x + vel_1.x,
            y: pos_1.y + vel_1.y,
        };

        'shape_entities: for (entity_2, shape_2) in world.shape.iter() {
            // iterate over the other entities with a shape

            // avoid checking collision with self
            if entity_1 == entity_2 {
                continue 'shape_entities;
            };

            // avoid rechecking the same pairs
            let pair = (entity_1.min(entity_2), entity_1.max(entity_2));

            if checks.contains(&pair) {
                continue 'shape_entities;
            }
            checks.insert(pair);

            // check if entity_2 has position
            let Some(pos_2) = world.pos.get(entity_2) else {
                continue 'shape_entities;
            };

            // compute next update position also for entity_2 if it has a velocity
            let pos_2 = if let Some(vel_2) = world.vel.get(entity_2) {
                components::Pos {
                    x: pos_2.x + vel_2.x,
                    y: pos_2.y + vel_2.y,
                }
            } else {
                pos_2.clone()
            };

            // check collision
            if let Some(angle) = check_collision(&pos_1, shape_1, &pos_2, shape_2) {
                // it will collide with something
                collisions.push((entity_1, entity_2, angle));
            };
        }
    }
    collisions
}

pub fn simulate_collisions(
    world: &mut World,
    collisions: Vec<(entities::Entity, entities::Entity, components::Angle)>,
) {
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

        let vel_1 = world.vel.get(entity_1).expect("missing velocity for a dynamic object");
        let vel_1_normal = vel_1.x * normal_x + vel_1.y * normal_y;

        let vel_1_tangent = vel_1.x * tangent_x + vel_1.y * tangent_y;

        if let Some(vel_2) = world.vel.get(entity_2) {
            // dynamic-dynamic collision
            let vel_2_normal = vel_2.x * normal_x + vel_2.y * normal_y;

            let vel_2_tangent = vel_2.x * tangent_x + vel_2.y * tangent_y;

            let mass_1 = world.mass.get(entity_1).expect("missing mass").0;
            let mass_2 = world.mass.get(entity_2).expect("missing mass").0;

            let vel_1_normal_post =
                ((vel_1_normal * (mass_1 - mass_2) + 2.0 * mass_2 * vel_2_normal) / (mass_1 + mass_2)) * elast;
            let vel_2_normal_post =
                ((vel_2_normal * (mass_2 - mass_1) + 2.0 * mass_1 * vel_1_normal) / (mass_1 + mass_2)) * elast;

            let new_vel_1 = components::Vel {
                x: vel_1_normal_post * normal_x + vel_1_tangent * tangent_x,
                y: mask_below_eps(
                    vel_1_normal_post * normal_y + vel_1_tangent * tangent_y,
                    vel_1.y,
                    rest_eps,
                ),
            };

            let new_vel_2 = components::Vel {
                x: vel_2_normal_post * normal_x + vel_2_tangent * tangent_x,
                y: mask_below_eps(
                    vel_2_normal_post * normal_y + vel_2_tangent * tangent_y,
                    vel_2.y,
                    rest_eps,
                ),
            };

            world.vel.set(entity_1, new_vel_1);
            world.vel.set(entity_2, new_vel_2);
        } else {
            // dynamic-static collision
            let vel_1_normal_post = -vel_1_normal * elast;

            let new_vel_1 = components::Vel {
                x: vel_1_normal_post * normal_x + vel_1_tangent * tangent_x,
                y: vel_1_normal_post * normal_y + vel_1_tangent * tangent_y,
            };

            world.vel.set(entity_1, new_vel_1);
        }
    }
}

fn mask_below_eps(value: f32, check: f32, eps: f32) -> f32 {
    if check.abs() >= eps { value } else { 0.0 }
}
