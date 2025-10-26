use crate::{
    core::{error, world::World},
    ecs::{components, entities},
    math::{self, ToHitBox},
    math::{EPS, EPS_SQR},
};

/// checks if 2 hitboxes are colliding using EPS to prevent false negatives
fn check_hitboxes(hitbox_1: &math::HitBox, hitbox_2: &math::HitBox) -> bool {
    !(hitbox_1.min_x > hitbox_2.max_x + EPS
        || hitbox_2.min_x > hitbox_1.max_x + EPS
        || hitbox_1.min_y > hitbox_2.max_y + EPS
        || hitbox_2.min_y > hitbox_1.max_y + EPS)
}

/// checks if 2 objects are colliding using SAT algorithm, returns the contact normal
fn check_sat(swept_shape_1: &math::SweptShape, swept_shape_2: &math::SweptShape) -> Option<math::Vec2> {
    fn add_axes(swept_shape: &math::SweptShape, axes: &mut Vec<math::Vec2>) {
        fn add_polygon_axes(polygon: &math::Polygon, axes: &mut Vec<math::Vec2>) {
            let len = polygon.verts.len();

            for i in 0..len {
                let edge = polygon.verts[(i + 1) % len].sub(polygon.verts[i]);
                if edge.square_mag() > EPS_SQR {
                    axes.push(edge.perp_ccw().norm());
                }
            }
        }

        match swept_shape {
            math::SweptShape::Unmoved { shape, pos: _ } => match shape {
                math::Shape::Segment(segment) => {
                    let edge = segment.b.sub(segment.a);
                    if edge.square_mag() > EPS_SQR {
                        axes.push(edge.perp_ccw().norm())
                    }
                }

                math::Shape::Triangle(triangle) => {
                    let edges = [
                        triangle.b.sub(triangle.a),
                        triangle.c.sub(triangle.b),
                        triangle.a.sub(triangle.c),
                    ];
                    for edge in edges {
                        if edge.square_mag() > EPS_SQR {
                            axes.push(edge.perp_ccw().norm());
                        }
                    }
                }
                math::Shape::Rect(_) => {
                    axes.push(math::Vec2::new(1.0, 0.0)); // add horizontal
                    axes.push(math::Vec2::new(0.0, 1.0)); // add vertical
                }
                math::Shape::Circle(_) => unimplemented!(),
                math::Shape::Polygon(polygon) => add_polygon_axes(polygon, axes),
            },
            math::SweptShape::AxisRect { swept: _, pos: _ } => {
                axes.push(math::Vec2::new(1.0, 0.0)); // add horizontal
                axes.push(math::Vec2::new(0.0, 1.0)); // add vertical
            }
            math::SweptShape::Moved { swept } => add_polygon_axes(swept, axes),
        }
    }

    fn project_shape(swept_shape: &math::SweptShape, axis: math::Vec2) -> (f32, f32) {
        fn project_rect(rect: &math::Rect, pos: math::Vec2, axis: math::Vec2) -> (f32, f32) {
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
            math::SweptShape::Unmoved { shape, pos } => {
                // unmoved shapes have local positions

                match shape {
                    math::Shape::Segment(segment) => {
                        let a_proj = pos.add(segment.a).dot(axis);
                        let b_proj = pos.add(segment.b).dot(axis);

                        (a_proj.min(b_proj), a_proj.max(b_proj))
                    }
                    math::Shape::Triangle(triangle) => {
                        let a_proj = pos.add(triangle.a).dot(axis);
                        let b_proj = pos.add(triangle.b).dot(axis);
                        let c_proj = pos.add(triangle.c).dot(axis);

                        (a_proj.min(b_proj.min(c_proj)), a_proj.max(b_proj.max(c_proj)))
                    }
                    math::Shape::Rect(rect) => project_rect(rect, *pos, axis),
                    math::Shape::Circle(_) => unimplemented!(),
                    math::Shape::Polygon(polygon) => {
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
            math::SweptShape::AxisRect { swept, pos } => project_rect(swept, *pos, axis), // axis-rect has local positions
            math::SweptShape::Moved { swept } => {
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
    fn remove_duplicate_axes(axes: &[math::Vec2]) -> Vec<math::Vec2> {
        let mut unique: Vec<math::Vec2> = Vec::with_capacity(axes.len());
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

    fn centroid(swept_shape: &math::SweptShape) -> math::Vec2 {
        match swept_shape {
            math::SweptShape::Unmoved { shape, pos } => {
                // unmoved shapes have local positions

                match shape {
                    math::Shape::Segment(segment) => pos.add(segment.a.add(segment.b).scale(0.5)),
                    math::Shape::Triangle(triangle) => {
                        pos.add(triangle.a.add(triangle.b.add(triangle.c)).scale(1.0 / 3.0))
                    }
                    math::Shape::Rect(rect) => pos.add(math::Vec2::new(rect.width / 2.0, rect.height / 2.0)),
                    math::Shape::Circle(_) => unimplemented!(),
                    math::Shape::Polygon(polygon) => {
                        let mut sum = math::Vec2::new(0.0, 0.0);
                        for vert in &polygon.verts {
                            sum.add_mut(*vert);
                        }
                        pos.add(sum.scale(1.0 / polygon.verts.len() as f32))
                    }
                }
            }
            math::SweptShape::AxisRect { swept, pos } => {
                // axis-rect has local positions
                pos.add(math::Vec2::new(swept.width / 2.0, swept.height / 2.0))
            }
            math::SweptShape::Moved { swept } => {
                // moved polygon has global positions

                let mut sum = math::Vec2::new(0.0, 0.0);
                for vert in &swept.verts {
                    sum.add_mut(*vert);
                }
                sum.scale(1.0 / swept.verts.len() as f32)
            }
        }
    }

    // vector of axes
    let mut axes: Vec<math::Vec2> = Vec::new();
    add_axes(swept_shape_1, &mut axes);
    add_axes(swept_shape_2, &mut axes);
    let axes = remove_duplicate_axes(&axes); // remove duplicates to avoid more axis checks

    // compute centroids for the 2 swept_shapes
    let centroid_1 = centroid(swept_shape_1);
    let centroid_2 = centroid(swept_shape_2);
    let delta = centroid_2.sub(centroid_1); // point from swept_shape_1 to swept_shape_2

    // initialize normal data
    let mut min_overlap = f32::INFINITY;
    let mut normal = math::Vec2::new(0.0, 0.0); // minimum translation vector axis, the axis of the smallest vector to push one shape out of the other

    for axis in axes {
        let (min_1, max_1) = project_shape(swept_shape_1, axis);
        let (min_2, max_2) = project_shape(swept_shape_2, axis);

        if min_1 > max_2 + EPS || min_2 > max_1 + EPS {
            // not colliding
            return None;
        }

        let overlap = (max_1.min(max_2)) - (min_1.max(min_2));
        if overlap < min_overlap {
            // update the normal data
            min_overlap = overlap;
            normal = if delta.dot(axis) < 0.0 { axis.neg() } else { axis }; // invert the normal direction if it is not from swept_shape_1 to swept_shape_2
        }
    }

    Some(normal)
}

/// checks if 2 objects are colliding and returns the contact normal
/// it prechecks using hitboxes and if the hitboxes are colliding it switches to SAT algorithm
fn check_collision(swept_shape_1: &math::SweptShape, swept_shape_2: &math::SweptShape) -> Option<math::Vec2> {
    let hitbox_1 = swept_shape_1.hitbox();
    let hitbox_2 = swept_shape_2.hitbox();

    if check_hitboxes(&hitbox_1, &hitbox_2) {
        // hitbox are colliding, check collision using SAT
        return check_sat(swept_shape_1, swept_shape_2);
    }
    None
}

/// generates a convex hull from a vector of points using monotone chain algorithm
pub fn convex_hull(mut verts: Vec<math::Vec2>) -> Result<math::Polygon, error::GeometryError> {
    // precheck for an early return if too few vertices are given, although this check will be
    // performed automatically when calling components::Polygon::new() at the end of this function
    if verts.len() < 3 {
        return Err(error::GeometryError::TooFewVertices(verts.len()));
    }

    // sort by x and, if x is the same, by y (reversed because low y = top and high y = bottom)
    verts.sort_unstable_by(|a, b| a.x.total_cmp(&b.x).then_with(|| b.y.total_cmp(&a.y)));

    fn walk(verts: &[math::Vec2]) -> Vec<math::Vec2> {
        let mut boundary: Vec<math::Vec2> = Vec::with_capacity(verts.len());

        for &v in verts {
            while boundary.len() >= 2 {
                let b = boundary.len();
                if (boundary[b - 2]).signed_area(boundary[b - 1], v) >= 0.0 {
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

    verts.reverse();

    // compute top boundary (counterclockwise from rightmost to leftmost)
    let mut top_boundary = walk(&verts);

    // drop lasts to avoid duplication
    bottom_boundary.pop();
    top_boundary.pop();

    // concat
    bottom_boundary.extend(top_boundary);

    math::Polygon::new(bottom_boundary)
}

/// generates a swept shape from a stationary or moving shape
fn generate_swept_shape(pos_1: math::Vec2, pos_2: math::Vec2, shape: &math::Shape) -> math::SweptShape<'_> {
    if pos_1.square_dist(pos_2) <= EPS_SQR {
        // the object is not moving
        math::SweptShape::Unmoved {
            shape: shape,
            pos: pos_1,
        }
    } else {
        // the object is moving
        match shape {
            math::Shape::Segment(segment) => {
                let mut verts = Vec::with_capacity(4);

                verts.push(pos_1.add(segment.a));
                verts.push(pos_1.add(segment.b));
                verts.push(pos_2.add(segment.a));
                verts.push(pos_2.add(segment.b));

                math::SweptShape::Moved {
                    swept: convex_hull(verts).expect("we passed more than 3 verts"),
                }
            }
            math::Shape::Triangle(triangle) => {
                let mut verts = Vec::with_capacity(6);

                verts.push(pos_1.add(triangle.a));
                verts.push(pos_1.add(triangle.b));
                verts.push(pos_1.add(triangle.c));
                verts.push(pos_2.add(triangle.a));
                verts.push(pos_2.add(triangle.b));
                verts.push(pos_2.add(triangle.c));

                math::SweptShape::Moved {
                    swept: convex_hull(verts).expect("we passed more than 3 verts"),
                }
            }
            math::Shape::Rect(rect) => {
                // check for axis optimization
                if (pos_1.x - pos_2.x).abs() <= EPS {
                    // vertical-only movement
                    let min_y = pos_1.y.min(pos_2.y);
                    let delta_y = (pos_1.y - pos_2.y).abs();

                    math::SweptShape::AxisRect {
                        swept: math::Rect::new(rect.width, delta_y + rect.height).expect(
                            "delta is always positive and the old rect is valid, so this should be always valid",
                        ),
                        pos: math::Vec2::new(pos_1.x, min_y),
                    }
                } else if (pos_1.y - pos_2.y).abs() <= EPS {
                    // horizontal-only movement
                    let min_x = pos_1.x.min(pos_2.x);
                    let delta_x = (pos_1.x - pos_2.x).abs();

                    math::SweptShape::AxisRect {
                        swept: math::Rect::new(delta_x + rect.width, rect.height).expect(
                            "delta is always positive and the old rect is valid, so this should be always valid",
                        ),
                        pos: math::Vec2::new(min_x, pos_1.y),
                    }
                } else {
                    let mut verts = Vec::with_capacity(8);

                    verts.push(pos_1);
                    verts.push(pos_1.add(math::Vec2::new(rect.width, 0.0)));
                    verts.push(pos_1.add(math::Vec2::new(0.0, rect.height)));
                    verts.push(pos_1.add(math::Vec2::new(rect.width, rect.height)));
                    verts.push(pos_2);
                    verts.push(pos_2.add_scalar(rect.width, 0.0));
                    verts.push(pos_2.add_scalar(0.0, rect.height));
                    verts.push(pos_2.add_scalar(rect.width, rect.height));

                    math::SweptShape::Moved {
                        swept: convex_hull(verts).expect("we passed more than 3 verts"),
                    }
                }
            }
            math::Shape::Circle(_) => unimplemented!(),
            math::Shape::Polygon(polygon) => {
                let mut verts = Vec::with_capacity(polygon.verts.len() * 2);

                for &v in &polygon.verts {
                    verts.push(pos_1.add(v));
                    verts.push(pos_2.add(v));
                }

                math::SweptShape::Moved {
                    swept: convex_hull(verts).expect("we passed more than 3 verts"),
                }
            }
        }
    }
}

/// updates 2 entities' linear velocity vector after they collide
fn compute_reaction(world: &mut World, entity_1: entities::Entity, entity_2: entities::Entity, normal: math::Vec2) {
    // update rest
    if normal.x.abs() <= 0.2 {
        // one is above the other
        if normal.y > 0.0
            && let Some(translation) = world.translation.get_mut(entity_1)
        {
            translation.rest = true;
        }

        if normal.y < 0.0
            && let Some(translation) = world.translation.get_mut(entity_2)
        {
            translation.rest = true;
        }
    }

    // compute elast and friction
    let surface_1 = world.surface.get(entity_1).expect("missing surface");
    let surface_2 = world.surface.get(entity_2).expect("missing surface");

    let elast = surface_1.elast.min(surface_2.elast);
    let static_friction = (surface_1.static_friction * surface_2.static_friction).sqrt();
    let kinetic_friction = (surface_1.kinetic_friction * surface_2.kinetic_friction).sqrt();

    // extract lin_vel and inv_mass
    let (lin_vel_1, inv_mass_1) = {
        let translation = world.translation.get(entity_1).expect("missing translation");
        (translation.lin_vel, translation.inv_mass())
    };

    let (lin_vel_2, inv_mass_2) = {
        if let Some(translation) = world.translation.get(entity_2) {
            (translation.lin_vel, translation.inv_mass())
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

    let translation_1 = world.translation.get_mut(entity_1).expect("missing translation");
    translation_1.lin_vel.sub_mut(impulse_vector.scale(inv_mass_1)); // here we subtract the delta_lin_vel (see above why)

    // round y linear velocity to 0 for object 1
    if translation_1.rest && translation_1.lin_vel.y.abs() <= 0.6 {
        translation_1.lin_vel.y = 0.0;
    }

    // recompute lin_vel_1
    let lin_vel_1 = translation_1.lin_vel;

    let lin_vel_2 = if let Some(translation_2) = world.translation.get_mut(entity_2) {
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

    let translation_1 = world.translation.get_mut(entity_1).expect("missing translation");
    translation_1.lin_vel.sub_mut(friction_impulse_vector.scale(inv_mass_1));

    if let Some(translation_2) = world.translation.get_mut(entity_2) {
        translation_2.lin_vel.add_mut(friction_impulse_vector.scale(inv_mass_2));
    }
}

/// resolves all collisions for a given object
fn resolve_obj_collisions(world: &mut World, entity_1: entities::Entity, ents: &Vec<entities::Entity>) -> bool {
    let mut solved = true;

    // checks entity_1 has all the components necessary for being a dynamic object and extracts its position
    let (Some(&components::Transform { pos: pos_1, .. }), Some(_), Some(_), Some(_)) = (
        world.transform.get(entity_1),
        world.translation.get(entity_1),
        world.surface.get(entity_1),
        world.shape.get(entity_1),
    ) else {
        // entity is not a dynamic object
        return true; // in this case it counts as solved
    };

    for &entity_2 in ents {
        // avoid checking collision with self
        if entity_1 == entity_2 {
            continue;
        };

        // checks entity_2 has all the components necessary for being at least a static object and extracts its position and shape
        let (Some(&components::Transform { pos: pos_2, .. }), Some(_), Some(shape_2)) = (
            world.transform.get(entity_2),
            world.surface.get(entity_2),
            world.shape.get(entity_2),
        ) else {
            continue;
        };

        // check if entity_2 is dynamic or static and extract its linear velocity
        let lin_vel_2 = world.translation.get(entity_2).map(|rb| rb.lin_vel);

        let normal = {
            // generate swept_shapes

            // re-extract lin_vel_1 and shape_1: lin_vel_1 because it may have changed by compute_reaction(), shape_1 because if it has not moved, swept_shape_1 will keep a reference to it,
            // and since we need to pass a mutable reference of world to compute_reaction() and world owns shape_1, we cannot have both a mutable and unmutable reference at the same time
            let (&components::Translation { lin_vel: lin_vel_1, .. }, shape_1) = (
                world.translation.get(entity_1).expect("missing translation"),
                world.shape.get(entity_1).expect("missing shape"),
            );
            let swept_shape_1 = generate_swept_shape(pos_1, pos_1.add(lin_vel_1), shape_1); // we are also recomputing the swept_shape at every iteration since its linear velocity may have changed

            let swept_shape_2 = if lin_vel_2.is_none() {
                // it is static, generate fixed swept_shape
                generate_swept_shape(pos_2, pos_2, shape_2)
            } else {
                // it is dynamic, generate swept_shape
                generate_swept_shape(pos_2, pos_2.add(lin_vel_2.expect("missing lin_vel")), shape_2)
            };

            // check collision
            check_collision(&swept_shape_1, &swept_shape_2)
        };

        if let Some(normal) = normal {
            solved = false;

            // they are colliding
            compute_reaction(world, entity_1, entity_2, normal);
        }
    }

    solved
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
pub fn resolve_collisions(world: &mut World, sort: bool, iters: usize) {
    let ents = if sort {
        // sort entities by y, with the highest (visually on the screen) being first, in order to optimize computations for objects resting on top of other objects
        // in addition, we can now iterate through this vector instead of calling a method multiple times to get the y
        sort_objs_by_y(world)
    } else {
        world.transform.get_ents()
    };

    for _ in 0..iters {
        let mut solved = true;
        for &entity in ents.iter() {
            if !resolve_obj_collisions(world, entity, &ents) {
                solved = false;
            }
        }

        if solved {
            break;
        }
    }
}
