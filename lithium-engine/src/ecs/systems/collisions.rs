use crate::{
    ecs::{
        components::{self, ToHitBox},
        entities,
        systems::physics::{EPS, EPS_SQR},
    },
    world::World,
};

/// checks if 2 rectangles are colliding
fn check_rect_rect(
    pos_1: components::Vec2,
    shape_1: components::Rect,
    pos_2: components::Vec2,
    shape_2: components::Rect,
) -> bool {
    // extract data about rectangles
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

    // check aabb with EPS to avoid false negatives (since positives will be checked via SAT algorithm)
    !(left_1 > right_2 + EPS || left_2 > right_1 + EPS || top_1 > bottom_2 + EPS || top_2 > bottom_1 + EPS)

    // compute overlaps (rect_1 point of view) since they are colliding
    // let left_overlap = right_2 - left_1;
    // let right_overlap = right_1 - left_2;
    // let top_overlap = bottom_2 - top_1;
    // let bottom_overlap = bottom_1 - top_2;

    // if left_overlap.min(right_overlap) < top_overlap.min(bottom_overlap) {
    //     if right_overlap <= left_overlap {
    //         Some(components::Angle::new(0.0))
    //     } else {
    //         Some(components::Angle::new(std::f32::consts::PI))
    //     }
    // } else {
    //     if top_overlap <= bottom_overlap {
    //         Some(components::Angle::new(std::f32::consts::FRAC_PI_2))
    //     } else {
    //         Some(components::Angle::new(3.0 * std::f32::consts::FRAC_PI_2))
    //     }
    // }
}

/// checks if 2 circles are colliding
// pub fn check_circle_circle(
//     pos_1: components::Vec2,
//     shape_1: components::Circle,
//     pos_2: components::Vec2,
//     shape_2: components::Circle,
// ) -> bool {
//     // extract data about circles
//     let radius_1 = shape_1.radius;
//     let radius_2 = shape_2.radius;

//     let delta_x = (pos_2.x + radius_2) - (pos_1.x + radius_1); // difference between centres x
//     let delta_y = (pos_2.y + radius_2) - (pos_1.y + radius_1); // difference between centres y

//     let radius_sum = radius_1 + radius_2;

//     // check square distance and square radius with EPS to avoid false negatives (since positives will be checked via SAT algorithm)
//     delta_x * delta_x + delta_y * delta_y >= (radius_sum + EPS) * (radius_sum + EPS)
// }

/// checks if a circle and a rectangle are colliding and returns the collision angle
// pub fn check_circle_rect() {}

/// checks if 2 objects are colliding using SAT algorithm, returns the mtv
/// for shapes with local vertices you need to pass them as global, while for shapes with just sizes (circles, rects) you don't need to
fn check_sat(
    pos_1: components::Vec2,
    shape_1: &components::Shape,
    pos_2: components::Vec2,
    shape_2: &components::Shape,
) -> Option<components::Vec2> {
    fn add_axes(shape: &components::Shape, axes: &mut Vec<components::Vec2>) {
        match shape {
            components::Shape::Segment(segment) => {
                let edge = segment.b.sub(segment.a);
                if edge.square_mag() > EPS_SQR {
                    axes.push(edge.perp().norm())
                }
            }
            components::Shape::Triangle(triangle) => {
                let edges = [
                    triangle.b.sub(triangle.a),
                    triangle.c.sub(triangle.b),
                    triangle.a.sub(triangle.c),
                ];
                for edge in edges {
                    if edge.square_mag() > EPS_SQR {
                        axes.push(edge.perp().norm());
                    }
                }
            }
            components::Shape::Rect(_) => {
                axes.push(components::Vec2::new(1.0, 0.0)); // add horizontal
                axes.push(components::Vec2::new(0.0, 1.0)); // add vertical
            }
            components::Shape::Circle(circle) => unimplemented!(),
            components::Shape::Polygon(polygon) => {
                let len = polygon.verts.len();

                for i in 0..len {
                    let edge = polygon.verts[(i + 1) % len].sub(polygon.verts[i]);
                    if edge.square_mag() > EPS_SQR {
                        axes.push(edge.perp().norm());
                    }
                }
            }
        }
    }

    fn project_shape(pos: components::Vec2, shape: &components::Shape, axis: components::Vec2) -> (f32, f32) {
        fn project_verts(verts: &[components::Vec2], axis: components::Vec2) -> (f32, f32) {
            let mut min = f32::INFINITY;
            let mut max = f32::NEG_INFINITY;

            for v in verts {
                let p = v.dot(axis);
                if p < min {
                    min = p;
                }
                if p > max {
                    max = p;
                }
            }
            (min, max)
        }

        match shape {
            components::Shape::Segment(segment) => {
                // for segment, vertices are already global

                let a_proj = segment.a.dot(axis);
                let b_proj = segment.b.dot(axis);

                if a_proj <= b_proj {
                    (a_proj, b_proj)
                } else {
                    (b_proj, a_proj)
                }
            }
            components::Shape::Triangle(triangle) => project_verts(&[triangle.a, triangle.b, triangle.c], axis), // for triangle, vertices are already global
            components::Shape::Rect(rect) => project_verts(
                // for rectangle, we have to add the global position
                &[
                    pos,
                    pos.add_scalar(rect.width, 0.0),
                    pos.add_scalar(0.0, rect.height),
                    pos.add_scalar(rect.width, rect.height),
                ],
                axis,
            ),
            components::Shape::Circle(circle) => unimplemented!(),
            components::Shape::Polygon(polygon) => project_verts(&polygon.verts, axis),
        }
    }

    fn remove_duplicates(axes: &[components::Vec2]) -> Vec<components::Vec2> {
        let mut unique = Vec::new();
        for &axis in axes {
            if !unique
                .iter()
                .any(|other_axis: &components::Vec2| other_axis.equal(axis))
            {
                unique.push(axis);
            }
        }
        unique
    }

    fn centroid(pos: components::Vec2, shape: &components::Shape) -> components::Vec2 {
        match shape {
            components::Shape::Segment(segment) => segment.a.add(segment.b).scale(0.5),
            components::Shape::Triangle(triangle) => triangle.a.add(triangle.b.add(triangle.c)).scale(1.0 / 3.0),
            components::Shape::Rect(rect) => pos.add(components::Vec2::new(rect.width / 2.0, rect.height / 2.0)), // for rectangle, we have to add the global position
            components::Shape::Circle(circle) => unimplemented!(),
            components::Shape::Polygon(polygon) => {
                let mut sum = components::Vec2::new(0.0, 0.0);
                for v in &polygon.verts {
                    sum.add_inplace(*v);
                }
                sum.scale(1.0 / polygon.verts.len() as f32)
            }
        }
    }

    // vector of axes, not normalized for performance
    let mut axes: Vec<components::Vec2> = Vec::new();
    add_axes(shape_1, &mut axes);
    add_axes(shape_2, &mut axes);
    let axes = remove_duplicates(&axes); // remove duplicates to avoid more axis checks

    // compute centroids for the 2 shapes
    let centroid_1 = centroid(pos_1, shape_1);
    let centroid_2 = centroid(pos_2, shape_2);
    let delta = centroid_2.sub(centroid_1); // point from shape_1 to shape_2

    // initialize mtv data
    let mut min_overlap = f32::INFINITY;
    let mut mtv_axis = components::Vec2::new(0.0, 0.0); // minimum translation vector, smallest vector to push one shape out of the other

    for axis in axes {
        let (min_1, max_1) = project_shape(pos_1, shape_1, axis);
        let (min_2, max_2) = project_shape(pos_2, shape_2, axis);

        if min_1 > max_2 + EPS || min_2 > max_1 + EPS {
            // not colliding
            return None;
        }

        let overlap = (max_1.min(max_2)) - (min_1.max(min_2));
        if overlap < min_overlap {
            // update the mtv
            min_overlap = overlap;
            mtv_axis = if delta.dot(axis) < 0.0 { axis.neg() } else { axis };
        }
    }

    Some(mtv_axis) //.scale(min_overlap))
}

/// checks if 2 objects are colliding and returns the mtv
/// it prechecks using hitboxes and if the hitboxes are colliding it switches to SAT algorithm
fn check_collision(
    pos_1: components::Vec2,
    shape_1: &components::Shape,
    pos_2: components::Vec2,
    shape_2: &components::Shape,
) -> Option<components::Vec2> {
    let hitbox_1 = shape_1.hitbox();
    let hitbox_2 = shape_2.hitbox();

    if check_rect_rect(pos_1, hitbox_1, pos_2, hitbox_2) {
        // hitbox are colliding, check collision using SAT
        return check_sat(pos_1, shape_1, pos_2, shape_2);
    }
    None
}

/// generates a convec hull from a vector of points using monotone chain algorithm
pub fn convex_hull(mut verts: Vec<components::Vec2>) -> Option<components::Polygon> {
    if verts.len() < 3 {
        return None;
    }

    // sort by x and, if x is the same, by y
    verts.sort_by(|a, b| a.x.partial_cmp(&b.x).unwrap().then(a.y.partial_cmp(&b.y).unwrap()));

    fn walk<'a>(verts: impl IntoIterator<Item = &'a components::Vec2>) -> Vec<components::Vec2> {
        let mut boundary: Vec<components::Vec2> = Vec::new();

        for &v in verts {
            while boundary.len() >= 2 {
                let b = boundary.len();
                if (boundary[b - 2]).signed_area(boundary[b - 1], v) <= 0.0 {
                    boundary.pop();
                } else {
                    break;
                }
            }
            boundary.push(v);
        }

        boundary
    }

    // compute bottom boundary (counterclockwise from leftmost to rightmost)
    let mut bottom_boundary = walk(&verts);

    // compute top boundary (counterclockwise from rightmost to leftmost)
    let mut top_boundary = walk(verts.iter().rev());

    // drop lasts to avoid duplication
    bottom_boundary.pop();
    top_boundary.pop();

    // concat
    bottom_boundary.extend(top_boundary);

    Some(components::Polygon::new(bottom_boundary))
}

/// generates a swept shape for a moving shape
/// if the shape is not moving, the input shape is returned
/// notice how the shape returned are in global positions
fn generate_swept_shape(
    pos_1: components::Vec2,
    pos_2: components::Vec2,
    shape: &components::Shape,
) -> components::Shape {
    if pos_1.square_dist(pos_2) > EPS_SQR {
        match &shape {
            components::Shape::Segment(segment) => {
                // compute vertices
                let mut verts = Vec::with_capacity(4);

                verts.push(pos_1.add(segment.a));
                verts.push(pos_1.add(segment.b));
                verts.push(pos_2.add(segment.a));
                verts.push(pos_2.add(segment.b));

                components::Shape::Polygon(convex_hull(verts).unwrap()) // None is impossible since we are passing 4 verts
            }
            components::Shape::Triangle(triangle) => {
                // compute vertices
                let mut verts = Vec::with_capacity(6);

                verts.push(pos_1.add(triangle.a));
                verts.push(pos_1.add(triangle.b));
                verts.push(pos_1.add(triangle.c));
                verts.push(pos_2.add(triangle.a));
                verts.push(pos_2.add(triangle.b));
                verts.push(pos_2.add(triangle.c));

                components::Shape::Polygon(convex_hull(verts).unwrap()) // None is impossible since we are passing 6 verts
            }
            components::Shape::Rect(rect) => {
                // compute vertices
                let mut verts = Vec::with_capacity(8);

                verts.push(pos_1);
                verts.push(pos_1.add(components::Vec2::new(rect.width, 0.0)));
                verts.push(pos_1.add(components::Vec2::new(0.0, rect.height)));
                verts.push(pos_1.add(components::Vec2::new(rect.width, rect.height)));
                verts.push(pos_2);
                verts.push(pos_2.add_scalar(rect.width, 0.0));
                verts.push(pos_2.add_scalar(0.0, rect.height));
                verts.push(pos_2.add_scalar(rect.width, rect.height));

                components::Shape::Polygon(convex_hull(verts).unwrap()) // None is impossible since we are passing 4/8 verts
            }

            components::Shape::Circle(circle) => unimplemented!(),
            components::Shape::Polygon(polygon) => {
                // compute vertices
                let mut verts = Vec::with_capacity(polygon.verts.len() * 2);

                for &v in &polygon.verts {
                    verts.push(pos_1.add(v));
                    verts.push(pos_2.add(v));
                }

                components::Shape::Polygon(convex_hull(verts).unwrap()) // None is impossible since we are passing more than 6 verts
            }
        }
    } else {
        // since it is not moving, just return a shape clone
        shape.clone()
    }
}

const REST_X_EPS: f32 = 2.0;

/// updates 2 entities' velocity vector after they collide
fn compute_reaction(world: &mut World, entity_1: entities::Entity, entity_2: entities::Entity, mtv: components::Vec2) {
    // update rest
    if mtv.x.abs() <= REST_X_EPS {
        // one is above the other
        if mtv.y > 0.0
            && let Some(rigid_body) = world.rigid_body.get_mut(entity_1)
        {
            rigid_body.rest = true;
        }

        if mtv.y < 0.0
            && let Some(rigid_body) = world.rigid_body.get_mut(entity_2)
        {
            rigid_body.rest = true;
        }
    }

    let elast = {
        let elast_1 = world.collider.get(entity_1).expect("missing collider").elast;
        let elast_2 = world.collider.get(entity_2).expect("missing collider").elast;
        elast_1.min(elast_2)
    };

    let (vel_1, inv_mass_1) = {
        let rigid_body = world.rigid_body.get(entity_1).expect("missing rigid body");
        if rigid_body.mass <= 0.0 {
            panic!("mass must be positive")
        };
        (rigid_body.vel, 1.0 / rigid_body.mass)
    };

    let (vel_2, inv_mass_2) = {
        if let Some(rigid_body) = world.rigid_body.get(entity_2) {
            if rigid_body.mass <= 0.0 {
                panic!("mass must be positive")
            };
            (rigid_body.vel, 1.0 / rigid_body.mass)
        } else {
            (components::Vec2::new(0.0, 0.0), 0.0)
        }
    };

    let rel_vel = vel_2.sub(vel_1).dot(mtv); // relative velocity from shape_1 to shape_2

    if rel_vel >= -EPS {
        // object are not getting closer
        return;
    };

    let impulse = -((1.0 + elast) * rel_vel / (inv_mass_1 + inv_mass_2));

    let rigid_body_1 = world.rigid_body.get_mut(entity_1).expect("missing rigid body");
    rigid_body_1.vel.sub_inplace(mtv.scale(impulse * inv_mass_1));

    // round velocity to 0 for object 1
    if rigid_body_1.vel.x.abs() <= 0.6 {
        rigid_body_1.vel.x = 0.0;
    }
    if rigid_body_1.vel.y.abs() <= 0.6 {
        rigid_body_1.vel.y = 0.0;
    }

    if let Some(rigid_body_2) = world.rigid_body.get_mut(entity_2) {
        rigid_body_2.vel.add_inplace(mtv.scale(impulse * inv_mass_2));

        // round velocity to 0 for object 2
        if rigid_body_2.vel.x.abs() <= 0.6 {
            rigid_body_2.vel.x = 0.0;
        }
        if rigid_body_2.vel.y.abs() <= 0.6 {
            rigid_body_2.vel.y = 0.0;
        }
    }
}

/// recursively resolves all collisions for a given object
fn resolve_obj_collisions(world: &mut World, entity_1: entities::Entity, ents: &Vec<entities::Entity>) {
    // checks the presence and extracts the position, velocity and shape of entity_1
    let (
        Some(&components::Transform { pos: pos_1, .. }),
        Some(&components::RigidBody { vel: vel_1, .. }),
        Some(shape_1),
    ) = (
        world.transform.get(entity_1),
        world.rigid_body.get(entity_1),
        world.shape.get(entity_1),
    )
    else {
        // entity is not a dynamic object
        return;
    };

    // generate swept_shape and hitbox for shape_1
    let swept_shape_1 = generate_swept_shape(pos_1, pos_1.add(vel_1), shape_1);

    for &entity_2 in ents {
        // avoid checking collision with self
        if entity_1 == entity_2 {
            continue;
        };

        // checks the presence and extracts the position and shape of entity_2
        let (Some(&components::Transform { pos: pos_2, .. }), Some(shape_2)) =
            (world.transform.get(entity_2), world.shape.get(entity_2))
        else {
            continue;
        };

        // check if entity_2 is dynamic or static and extract its velocity
        let vel_2 = world.rigid_body.get(entity_2).map(|rb| rb.vel);

        let swept_shape_2 = if vel_2.is_none() {
            // it is static, generate fixed swept_shape since we still need the vertices to be updated to global position
            generate_swept_shape(pos_2, pos_2, shape_2)
        } else {
            // it is dynamic, generate swept_shape
            generate_swept_shape(pos_2, pos_2.add(vel_2.unwrap()), shape_2)
        };

        // check collision
        if let Some(mtv) = check_collision(pos_1, &swept_shape_1, pos_2, &swept_shape_2) {
            compute_reaction(world, entity_1, entity_2, mtv);

            // resolve_obj_collisions(world, entity_1, &ents);
            // resolve_obj_collisions(world, entity_2, &ents);
        }
    }
}

/// sorts by y all the objects that own a position, from minimum to maximum
fn sort_objs_by_y(world: &mut World) -> Vec<entities::Entity> {
    // get reference of the transform vector
    let transform = world.transform.get_ref();

    // exctract copies of y from each transform
    let ys: Vec<f32> = transform.iter().map(|r| r.pos.y).collect();

    // copy entities implementing transform
    let ents = world.transform.get_ents();

    // zip vector toghether
    let mut pairs: Vec<(f32, u32)> = ys.into_iter().zip(ents).collect();

    // sort by y
    pairs.sort_by(|(a, _), (b, _)| a.total_cmp(b));

    // extract sorted entities
    let (_, ents): (Vec<f32>, Vec<u32>) = pairs.into_iter().unzip();

    ents
}

/// launches resolve_obj_collisions for each object
pub fn resolve_collisions(world: &mut World, sort: bool) {
    let ents = if sort {
        // sort entities by y, with the highest being first, in order to optimize computations for objects resting on top of other objects
        // in addition, we can now iterate through this vector instead of calling a method multiple times
        sort_objs_by_y(world)
    } else {
        world.transform.get_ents()
    };

    for &entity in ents.iter() {
        resolve_obj_collisions(world, entity, &ents);
    }
}
