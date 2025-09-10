use crate::{
    ecs::{
        components::{self, ToHitBox},
        entities,
        systems::physics::{EPS, EPS_SQR},
    },
    world::World,
};

/// checks if 2 hitboxes are colliding using EPS to prevent false negatives
fn check_hitboxes(hitbox_1: &components::HitBox, hitbox_2: &components::HitBox) -> bool {
    !(hitbox_1.min_x > hitbox_2.max_x + EPS
        || hitbox_2.min_x > hitbox_1.max_x + EPS
        || hitbox_1.min_y > hitbox_2.max_y + EPS
        || hitbox_2.min_y > hitbox_1.max_y + EPS)
}

/// checks if 2 objects are colliding using SAT algorithm, returns the mtv_axis
fn check_sat(
    swept_shape_1: &components::SweptShape,
    swept_shape_2: &components::SweptShape,
) -> Option<components::Vec2> {
    fn add_axes(swept_shape: &components::SweptShape, axes: &mut Vec<components::Vec2>) {
        fn add_polygon_axes(polygon: &components::Polygon, axes: &mut Vec<components::Vec2>) {
            let len = polygon.verts.len();

            for i in 0..len {
                let edge = polygon.verts[(i + 1) % len].sub(polygon.verts[i]);
                if edge.square_mag() > EPS_SQR {
                    axes.push(edge.perp().norm());
                }
            }
        }

        match swept_shape {
            components::SweptShape::Unmoved { shape, pos: _ } => match shape {
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
                components::Shape::Circle(_) => unimplemented!(),
                components::Shape::Polygon(polygon) => add_polygon_axes(polygon, axes),
            },
            components::SweptShape::AxisRect { swept: _, pos: _ } => {
                axes.push(components::Vec2::new(1.0, 0.0)); // add horizontal
                axes.push(components::Vec2::new(0.0, 1.0)); // add vertical
            }
            components::SweptShape::Moved { swept } => add_polygon_axes(swept, axes),
        }
    }

    fn project_shape(swept_shape: &components::SweptShape, axis: components::Vec2) -> (f32, f32) {
        fn project_rect(rect: &components::Rect, pos: components::Vec2, axis: components::Vec2) -> (f32, f32) {
            let a_proj = pos.dot(axis);
            let b_proj = pos.add_scalar(rect.width, 0.0).dot(axis);
            let c_proj = pos.add_scalar(0.0, rect.height).dot(axis);
            let d_proj = pos.add_scalar(rect.width, rect.height).dot(axis);

            (
                a_proj.min(b_proj.min(c_proj.min(d_proj))),
                a_proj.max(b_proj.max(c_proj.max(d_proj))),
            )
        }

        match swept_shape {
            components::SweptShape::Unmoved { shape, pos } => {
                // unmoved shapes have local positions

                match shape {
                    components::Shape::Segment(segment) => {
                        let a_proj = pos.add(segment.a).dot(axis);
                        let b_proj = pos.add(segment.b).dot(axis);

                        (a_proj.min(b_proj), a_proj.max(b_proj))
                    }
                    components::Shape::Triangle(triangle) => {
                        let a_proj = pos.add(triangle.a).dot(axis);
                        let b_proj = pos.add(triangle.b).dot(axis);
                        let c_proj = pos.add(triangle.c).dot(axis);

                        (a_proj.min(b_proj.min(c_proj)), a_proj.max(b_proj.max(c_proj)))
                    }
                    components::Shape::Rect(rect) => project_rect(rect, *pos, axis),
                    components::Shape::Circle(_) => unimplemented!(),
                    components::Shape::Polygon(polygon) => {
                        let mut min = f32::INFINITY;
                        let mut max = f32::NEG_INFINITY;

                        for vert in &polygon.verts {
                            let proj = pos.add(*vert).dot(axis);
                            min = min.min(proj);
                            max = max.max(proj);
                        }
                        (min, max)
                    }
                }
            }
            components::SweptShape::AxisRect { swept, pos } => project_rect(swept, *pos, axis), // axis-rect has local positions
            components::SweptShape::Moved { swept } => {
                // moved polygon has global positions

                let mut min = f32::INFINITY;
                let mut max = f32::NEG_INFINITY;

                for vert in &swept.verts {
                    let proj = vert.dot(axis); // I am not reusing this code because here we don't sum position, so it is simpler like this
                    if proj < min {
                        min = proj;
                    }
                    if proj > max {
                        max = proj;
                    }
                }
                (min, max)
            }
        }
    }

    #[inline]
    fn remove_duplicate_axes(axes: &[components::Vec2]) -> Vec<components::Vec2> {
        let mut unique: Vec<components::Vec2> = Vec::with_capacity(axes.len());
        for &axis in axes {
            for &u in &unique {
                if axis.dot(u).abs() >= 1.0 - EPS {
                    // axis are normalized
                    continue;
                }
            }
            unique.push(axis);
        }
        unique
    }

    fn centroid(swept_shape: &components::SweptShape) -> components::Vec2 {
        match swept_shape {
            components::SweptShape::Unmoved { shape, pos } => {
                // unmoved shapes have local positions

                match shape {
                    components::Shape::Segment(segment) => pos.add(segment.a.add(segment.b).scale(0.5)),
                    components::Shape::Triangle(triangle) => {
                        pos.add(triangle.a.add(triangle.b.add(triangle.c)).scale(1.0 / 3.0))
                    }
                    components::Shape::Rect(rect) => {
                        pos.add(components::Vec2::new(rect.width / 2.0, rect.height / 2.0))
                    }
                    components::Shape::Circle(_) => unimplemented!(),
                    components::Shape::Polygon(polygon) => {
                        let mut sum = components::Vec2::new(0.0, 0.0);
                        for vert in &polygon.verts {
                            sum.add_inplace(*vert);
                        }
                        pos.add(sum.scale(1.0 / polygon.verts.len() as f32))
                    }
                }
            }
            components::SweptShape::AxisRect { swept, pos } => {
                // axis-rect has local positions
                pos.add(components::Vec2::new(swept.width / 2.0, swept.height / 2.0))
            }
            components::SweptShape::Moved { swept } => {
                // moved polygon has global positions

                let mut sum = components::Vec2::new(0.0, 0.0);
                for vert in &swept.verts {
                    sum.add_inplace(*vert);
                }
                sum.scale(1.0 / swept.verts.len() as f32)
            }
        }
    }

    // vector of axes
    let mut axes: Vec<components::Vec2> = Vec::new();
    add_axes(swept_shape_1, &mut axes);
    add_axes(swept_shape_2, &mut axes);
    let axes = remove_duplicate_axes(&axes); // remove duplicates to avoid more axis checks

    // compute centroids for the 2 swept_shapes
    let centroid_1 = centroid(swept_shape_1);
    let centroid_2 = centroid(swept_shape_2);
    let delta = centroid_2.sub(centroid_1); // point from swept_shape_1 to swept_shape_2

    // initialize mtv data
    let mut min_overlap = f32::INFINITY;
    let mut mtv_axis = components::Vec2::new(0.0, 0.0); // minimum translation vector axis, the axis of the smallest vector to push one shape out of the other

    for axis in axes {
        let (min_1, max_1) = project_shape(swept_shape_1, axis);
        let (min_2, max_2) = project_shape(swept_shape_2, axis);

        if min_1 > max_2 + EPS || min_2 > max_1 + EPS {
            // not colliding
            return None;
        }

        let overlap = (max_1.min(max_2)) - (min_1.max(min_2));
        if overlap < min_overlap {
            // update the mtv data
            min_overlap = overlap;
            mtv_axis = if delta.dot(axis) < 0.0 { axis.neg() } else { axis }; // invert the mtv_axis direction if it is not from swept_shape_1 to swept_shape_2
        }
    }

    Some(mtv_axis)
}

/// checks if 2 objects are colliding and returns the mtv
/// it prechecks using hitboxes and if the hitboxes are colliding it switches to SAT algorithm
fn check_collision(
    swept_shape_1: &components::SweptShape,
    swept_shape_2: &components::SweptShape,
) -> Option<components::Vec2> {
    let hitbox_1 = swept_shape_1.hitbox();
    let hitbox_2 = swept_shape_2.hitbox();

    if check_hitboxes(&hitbox_1, &hitbox_2) {
        // hitbox are colliding, check collision using SAT
        return check_sat(swept_shape_1, swept_shape_2);
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

/// generates a swept shape from a moving shape
fn generate_swept_shape(
    pos_1: components::Vec2,
    pos_2: components::Vec2,
    shape: &components::Shape,
) -> components::SweptShape<'_> {
    if pos_1.square_dist(pos_2) <= EPS_SQR {
        // the object is not moving
        components::SweptShape::Unmoved {
            shape: shape,
            pos: pos_1,
        }
    } else {
        // the object is moving
        match shape {
            components::Shape::Segment(segment) => {
                let mut verts = Vec::with_capacity(4);

                verts.push(pos_1.add(segment.a));
                verts.push(pos_1.add(segment.b));
                verts.push(pos_2.add(segment.a));
                verts.push(pos_2.add(segment.b));

                components::SweptShape::Moved {
                    swept: convex_hull(verts).unwrap(),
                }
            }
            components::Shape::Triangle(triangle) => {
                let mut verts = Vec::with_capacity(6);

                verts.push(pos_1.add(triangle.a));
                verts.push(pos_1.add(triangle.b));
                verts.push(pos_1.add(triangle.c));
                verts.push(pos_2.add(triangle.a));
                verts.push(pos_2.add(triangle.b));
                verts.push(pos_2.add(triangle.c));

                components::SweptShape::Moved {
                    swept: convex_hull(verts).unwrap(),
                }
            }
            components::Shape::Rect(rect) => {
                // check for axis optimization
                if (pos_1.x - pos_2.x).abs() <= EPS {
                    // vertical-only movement
                    let min_y = pos_1.y.min(pos_2.y);
                    let delta_y = (pos_1.y - pos_2.y).abs();

                    components::SweptShape::AxisRect {
                        swept: components::Rect::new(rect.width, delta_y + rect.height),
                        pos: components::Vec2::new(pos_1.x, min_y),
                    }
                } else if (pos_1.y - pos_2.y).abs() <= EPS {
                    // horizontal-only movement
                    let min_x = pos_1.x.min(pos_2.x);
                    let delta_x = (pos_1.x - pos_2.x).abs();

                    components::SweptShape::AxisRect {
                        swept: components::Rect::new(delta_x + rect.width, rect.height),
                        pos: components::Vec2::new(min_x, pos_1.y),
                    }
                } else {
                    let mut verts = Vec::with_capacity(8);

                    verts.push(pos_1);
                    verts.push(pos_1.add(components::Vec2::new(rect.width, 0.0)));
                    verts.push(pos_1.add(components::Vec2::new(0.0, rect.height)));
                    verts.push(pos_1.add(components::Vec2::new(rect.width, rect.height)));
                    verts.push(pos_2);
                    verts.push(pos_2.add_scalar(rect.width, 0.0));
                    verts.push(pos_2.add_scalar(0.0, rect.height));
                    verts.push(pos_2.add_scalar(rect.width, rect.height));

                    components::SweptShape::Moved {
                        swept: convex_hull(verts).unwrap(),
                    }
                }
            }
            components::Shape::Circle(_) => unimplemented!(),
            components::Shape::Polygon(polygon) => {
                let mut verts = Vec::with_capacity(polygon.verts.len() * 2);

                for &v in &polygon.verts {
                    verts.push(pos_1.add(v));
                    verts.push(pos_2.add(v));
                }

                components::SweptShape::Moved {
                    swept: convex_hull(verts).unwrap(),
                }
            }
        }
    }
}

/// updates 2 entities' velocity vector after they collide
fn compute_reaction(
    world: &mut World,
    entity_1: entities::Entity,
    entity_2: entities::Entity,
    mtv_axis: components::Vec2,
) {
    // update rest
    if mtv_axis.x.abs() <= 2.0 {
        // one is above the other
        if mtv_axis.y > 0.0
            && let Some(rigid_body) = world.rigid_body.get_mut(entity_1)
        {
            rigid_body.rest = true;
        }

        if mtv_axis.y < 0.0
            && let Some(rigid_body) = world.rigid_body.get_mut(entity_2)
        {
            rigid_body.rest = true;
        }
    }

    let elast = {
        let elast_1 = world.surface.get(entity_1).expect("missing surface").elast;
        let elast_2 = world.surface.get(entity_2).expect("missing surface").elast;
        elast_1.min(elast_2)
    };

    let (vel_1, inv_mass_1) = {
        let rigid_body = world.rigid_body.get(entity_1).expect("missing rigid_body");
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

    // relative velocity from shape_1 to shape_2
    // it is basically the vector from vel_1 to vel_2 projected on the mtv_axis
    // remember that mtv_axis is the unit vector perpendicular to the edge with minimum overlap
    let rel_vel = vel_2.sub(vel_1).dot(mtv_axis);

    if rel_vel >= -EPS {
        // object are not getting closer
        return;
    };

    // so here are the steps to compute impulse:
    //
    // 1) first of all we want to prove that after the impulse, we have:
    // vel_1' = vel_1 - J / mass_1    and    vel_2' = vel_2 + J / mass_2
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
    // vel_1' = vel_1 - J / mass_1    and    vel_2' = vel_2 + J / mass_2
    //
    // which is exactly what we were looking for
    //
    // 2) elast is definied as:
    // vel_rel' = -elast * vel_rel
    //
    // 3) the relative velocity is:
    // vel_rel = vel_2 - vel_1
    //
    // so the new relative velocity is:
    // vel_rel' = vel_2' - vel_1'
    //
    // 4) replacing, we have:
    // vel_rel' = (vel_2 + J / mass_2) - (vel_1 - J / mass_1)
    // vel_rel' = vel_2 - vel_1 + J / mass_2 + J / mass_1
    // vel_rel' = vel_rel + J * (1 / mass_2 + 1 / mass_1)
    // -elast * vel_rel = vel_rel + J * (1 / mass_2 + 1 / mass_1)
    // -elast * vel_rel - vel_rel = J * (1 / mass_2 + 1 / mass_1)
    // vel_rel * (-elast - 1) = J * (1 / mass_2 + 1 / mass_1)
    // -vel_rel * (elast + 1) = J * (1 / mass_2 + 1 / mass_1)
    // J = -vel_rel * (elast + 1) / (1 / mass_2 + 1 / mass_1)
    //
    // and rearranging:
    // J = -((1 + elast) * rel_vel / (inv_mass_1 + inv_mass_2))
    let impulse = -((1.0 + elast) * rel_vel / (inv_mass_1 + inv_mass_2));

    // what we will do with impulse is simply this:
    // since:
    // delta_P = delta_vel * mass = J
    //
    // we get:
    // delta_vel_n = J_n / mass_n
    //
    // so that is the magnitude of delta_vel, the direction is simply the mtv_axis direction

    let rigid_body_1 = world.rigid_body.get_mut(entity_1).expect("missing rigid body");
    rigid_body_1.vel.sub_inplace(mtv_axis.scale(impulse * inv_mass_1)); // here we subtract the delta_vel (see above why)

    // round velocity to 0 for object 1
    if rigid_body_1.vel.x.abs() <= 0.6 {
        rigid_body_1.vel.x = 0.0;
    }
    if rigid_body_1.vel.y.abs() <= 0.6 {
        rigid_body_1.vel.y = 0.0;
    }

    if let Some(rigid_body_2) = world.rigid_body.get_mut(entity_2) {
        rigid_body_2.vel.add_inplace(mtv_axis.scale(impulse * inv_mass_2)); // here we add the delta_vel (see above why)

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
    let (Some(&components::Transform { pos: pos_1, .. }), Some(&components::RigidBody { vel: vel_1, .. }), Some(_)) = (
        world.transform.get(entity_1),
        world.rigid_body.get(entity_1),
        world.shape.get(entity_1),
    ) else {
        // entity is not a dynamic object
        return;
    };

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

        let mtv_axis = {
            // generate swept_shapes
            let shape_1 = world.shape.get(entity_1).unwrap();
            let swept_shape_1 = generate_swept_shape(pos_1, pos_1.add(vel_1), shape_1);

            let swept_shape_2 = if vel_2.is_none() {
                // it is static, generate fixed swept_shape since we still need the vertices to be updated to global position
                generate_swept_shape(pos_2, pos_2, shape_2)
            } else {
                // it is dynamic, generate swept_shape
                generate_swept_shape(pos_2, pos_2.add(vel_2.unwrap()), shape_2)
            };

            // check collision
            check_collision(&swept_shape_1, &swept_shape_2)
        };

        if let Some(mtv_axis) = mtv_axis {
            // they are colliding
            compute_reaction(world, entity_1, entity_2, mtv_axis);

            resolve_obj_collisions(world, entity_1, &ents);
            resolve_obj_collisions(world, entity_2, &ents);
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
