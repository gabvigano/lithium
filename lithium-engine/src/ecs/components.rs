use crate::ecs::systems::physics::{EPS, pow2};

use serde::Deserialize;
use std::fmt;

#[derive(Copy, Clone, Deserialize, Debug)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}

impl Vec2 {
    #[inline]
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    #[inline]
    pub fn equal(self, vec2: Self) -> bool {
        self.x == vec2.x && self.y == vec2.y
    }

    #[inline]
    pub fn add(self, vec2: Self) -> Self {
        Self::new(self.x + vec2.x, self.y + vec2.y)
    }

    #[inline]
    pub fn add_mut(&mut self, vec2: Self) {
        self.x += vec2.x;
        self.y += vec2.y;
    }

    #[inline]
    pub fn add_scalar(self, x: f32, y: f32) -> Self {
        Self::new(self.x + x, self.y + y)
    }

    #[inline]
    pub fn add_scalar_mut(&mut self, x: f32, y: f32) {
        self.x += x;
        self.y += y;
    }

    #[inline]
    pub fn sub(self, vec2: Self) -> Self {
        Self::new(self.x - vec2.x, self.y - vec2.y)
    }

    #[inline]
    pub fn sub_mut(&mut self, vec2: Self) {
        self.x -= vec2.x;
        self.y -= vec2.y;
    }

    #[inline]
    pub fn sub_scalar(self, x: f32, y: f32) -> Self {
        Self::new(self.x - x, self.y - y)
    }

    #[inline]
    pub fn sub_scalar_mut(&mut self, x: f32, y: f32) {
        self.x -= x;
        self.y -= y;
    }

    #[inline]
    pub fn mul(self, vec2: Self) -> Self {
        Self::new(self.x * vec2.x, self.y * vec2.y)
    }

    #[inline]
    pub fn mul_mut(&mut self, vec2: Self) {
        self.x *= vec2.x;
        self.y *= vec2.y;
    }

    #[inline]
    pub fn div(self, vec2: Self) -> Self {
        Self::new(self.x / vec2.x, self.y / vec2.y)
    }

    #[inline]
    pub fn div_mut(&mut self, vec2: Self) {
        self.x /= vec2.x;
        self.y /= vec2.y;
    }

    #[inline]
    pub fn scale(self, scalar: f32) -> Self {
        Self::new(self.x * scalar, self.y * scalar)
    }

    #[inline]
    pub fn scale_mut(&mut self, scalar: f32) {
        self.x *= scalar;
        self.y *= scalar;
    }

    #[inline]
    pub fn norm(self) -> Self {
        let mag = self.mag();
        Self::new(self.x / mag, self.y / mag)
    }

    #[inline]
    pub fn norm_mut(&mut self) {
        let mag = self.mag();
        self.x /= mag;
        self.y /= mag;
    }

    #[inline]
    pub fn neg(self) -> Self {
        Self::new(-self.x, -self.y)
    }

    #[inline]
    pub fn neg_mut(&mut self) {
        self.x = -self.x;
        self.y = -self.y;
    }

    #[inline]
    pub fn abs(self) -> Self {
        Self::new(self.x.abs(), self.y.abs())
    }

    #[inline]
    pub fn abs_mut(&mut self) {
        self.x = self.x.abs();
        self.y = self.y.abs();
    }

    #[inline]
    pub fn perp_cw(self) -> Self {
        Self::new(self.y, -self.x)
    }

    #[inline]
    pub fn perp_cw_mut(&mut self) {
        let x = self.x;
        self.x = self.y;
        self.y = -x;
    }

    #[inline]
    pub fn perp_ccw(self) -> Self {
        Self::new(-self.y, self.x)
    }

    #[inline]
    pub fn perp_ccw_mut(&mut self) {
        let x = self.x;
        self.x = -self.y;
        self.y = x;
    }

    #[inline]
    pub fn dot(self, vec2: Self) -> f32 {
        self.x.mul_add(vec2.x, self.y * vec2.y)
    }

    #[inline]
    pub fn cross(self, vec2: Self) -> f32 {
        self.x * vec2.y - self.y * vec2.x
    }

    #[inline]
    pub fn signed_area(self, q: Self, r: Self) -> f32 {
        (q.sub(self)).cross(r.sub(self))
    }

    #[inline]
    pub fn vec_dist(self, point: Self) -> Self {
        Self::new(point.x - self.x, point.y - self.y)
    }

    #[inline]
    pub fn dist(self, point: Self) -> f32 {
        self.square_dist(point).sqrt()
    }

    #[inline]
    pub fn square_dist(self, point: Self) -> f32 {
        pow2(point.x - self.x) + pow2(point.y - self.y)
    }

    #[inline]
    pub fn mag(self) -> f32 {
        self.square_mag().sqrt()
    }

    #[inline]
    pub fn square_mag(self) -> f32 {
        pow2(self.x) + pow2(self.y)
    }
}

impl fmt::Display for Vec2 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({:.4} ; {:.4})", self.x, self.y)
    }
}

#[derive(Deserialize)]
pub struct TransformSpec {
    pub spawn: Vec2,
    pub angle: Angle,
}

#[derive(Clone, Debug)]
pub struct Transform {
    pub spawn: Vec2,
    pub pos: Vec2,
    pub angle: Angle,
}

impl Transform {
    #[inline]
    pub fn new(spawn: Vec2, pos: Vec2, angle: Angle) -> Self {
        Self { spawn, pos, angle }
    }

    #[inline]
    pub fn reset_pos(&mut self) {
        self.pos = self.spawn
    }
}

impl fmt::Display for Transform {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "transform (spawn: {}, pos: {}, angle: {})",
            self.spawn, self.pos, self.angle
        )
    }
}

impl From<TransformSpec> for Transform {
    fn from(spec: TransformSpec) -> Self {
        Self::new(spec.spawn, spec.spawn, spec.angle)
    }
}

#[derive(Copy, Clone, Debug)]
pub enum Axis {
    Horizontal,
    Vertical,
}

#[derive(Copy, Clone, Deserialize, Debug)]
pub struct Angle {
    pub radians: f32,
}

impl Angle {
    #[inline]
    pub fn new(radians: f32) -> Self {
        Self { radians }
    }

    #[inline]
    pub fn norm(mut self) -> Self {
        self.radians = self.radians.rem_euclid(std::f32::consts::PI * 2.0);
        self
    }
}

impl fmt::Display for Angle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:.4} rad", self.radians)
    }
}

#[derive(Deserialize)]
pub struct RigidBodySpec {
    pub vel: Vec2,
    pub force: Vec2,
    pub mass: f32,
}

#[derive(Clone, Debug)]
pub struct RigidBody {
    pub vel: Vec2,
    pub force: Vec2,
    mass: f32,
    inv_mass: f32,
    pub rest: bool,
}

impl RigidBody {
    #[inline]
    pub fn new(vel: Vec2, force: Vec2, mass: f32, rest: bool) -> Self {
        if mass <= 0.0 {
            panic!("mass must be positive");
        }

        Self {
            vel,
            force,
            mass,
            inv_mass: 1.0 / mass,
            rest,
        }
    }

    #[inline]
    pub fn reset_vel(&mut self, new_vel: Vec2) {
        self.vel = new_vel;
    }

    #[inline]
    pub fn reset_force(&mut self, new_force: Vec2) {
        self.force = new_force;
    }

    pub fn mass(&self) -> f32 {
        self.mass
    }
    pub fn inv_mass(&self) -> f32 {
        self.inv_mass
    }

    pub fn set_mass(&mut self, mass: f32) {
        self.mass = mass;
        self.inv_mass = 1.0 / mass;
    }
}

impl fmt::Display for RigidBody {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "rigid_body (vel: {:.4}, force: {:.4}), mass: {:.4}, rest: {}",
            self.vel, self.force, self.mass, self.rest
        )
    }
}

impl From<RigidBodySpec> for RigidBody {
    fn from(spec: RigidBodySpec) -> Self {
        Self::new(spec.vel, spec.force, spec.mass, false)
    }
}

#[derive(Deserialize)]
pub struct SurfaceSpec {
    pub elast: f32,
    pub static_friction: f32,
    pub kinetic_friction: f32,
}

#[derive(Clone, Debug)]
pub struct Surface {
    pub elast: f32,
    pub static_friction: f32,
    pub kinetic_friction: f32,
}

impl Surface {
    #[inline]
    pub fn new(elast: f32, static_friction: f32, kinetic_friction: f32) -> Self {
        Self {
            elast,
            static_friction,
            kinetic_friction,
        }
    }
}

impl fmt::Display for Surface {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "surface (elast: {:.4})", self.elast)
    }
}

impl From<SurfaceSpec> for Surface {
    fn from(spec: SurfaceSpec) -> Self {
        Self::new(spec.elast, spec.static_friction, spec.kinetic_friction)
    }
}

#[derive(Deserialize)]
pub struct MaterialSpec {
    pub color: Color,
    pub layer: usize,
    pub show: bool,
}

#[derive(Clone, Debug)]
pub struct Material {
    pub color: Color,
    pub layer: usize,
    pub show: bool,
}

impl Material {
    #[inline]
    pub fn new(color: Color, layer: usize, show: bool) -> Self {
        Self { color, layer, show }
    }
}

impl fmt::Display for Material {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "material (color: {}, layer: {}, show: {})",
            self.color, self.layer, self.show
        )
    }
}

impl From<MaterialSpec> for Material {
    fn from(spec: MaterialSpec) -> Self {
        Self::new(spec.color, spec.layer, spec.show)
    }
}

#[derive(Copy, Clone, Deserialize, Debug)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Color {
    #[inline]
    pub fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }
}

impl fmt::Display for Color {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "rgba ({}, {}, {}, {})", self.r, self.g, self.b, self.a)
    }
}

#[derive(Clone, Debug)]
pub struct HitBox {
    pub min_x: f32,
    pub min_y: f32,
    pub max_x: f32,
    pub max_y: f32,
}

impl HitBox {
    #[inline]
    pub fn new(min_x: f32, min_y: f32, max_x: f32, max_y: f32) -> Self {
        Self {
            min_x,
            min_y,
            max_x,
            max_y,
        }
    }

    #[inline]
    pub fn add_pos(self, pos: Vec2) -> Self {
        Self::new(
            self.min_x + pos.x,
            self.min_y + pos.y,
            self.max_x + pos.x,
            self.max_y + pos.y,
        )
    }

    #[inline]
    pub fn add_pos_mut(&mut self, pos: Vec2) {
        self.min_x += pos.x;
        self.min_y += pos.y;
        self.max_x += pos.x;
        self.max_y += pos.y;
    }
}

impl fmt::Display for HitBox {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "hitbox ({:.4}, {:.4}, {:.4}, {:.4})",
            self.min_x, self.min_y, self.max_x, self.max_y
        )
    }
}

pub trait ToHitBox {
    fn hitbox(&self) -> HitBox;
}

#[derive(Clone, Debug)]
pub enum SweptShape<'a> {
    Unmoved { shape: &'a Shape, pos: Vec2 },
    AxisRect { swept: Rect, pos: Vec2 },
    Moved { swept: Polygon },
}

impl ToHitBox for SweptShape<'_> {
    fn hitbox(&self) -> HitBox {
        match self {
            SweptShape::Unmoved { shape, pos } => {
                // this is a local position, global position must be added
                shape.hitbox().add_pos(*pos)
            }
            SweptShape::AxisRect { swept, pos } => {
                // this is a local position, global position must be added
                swept.hitbox().add_pos(*pos)
            }
            SweptShape::Moved { swept } => {
                // this is already a global position
                swept.hitbox()
            }
        }
    }
}

#[derive(Clone, Deserialize, Debug)]
pub enum Shape {
    Segment(Segment),
    Triangle(Triangle),
    Rect(Rect),
    Circle(Circle),
    Polygon(Polygon),
}

impl ToHitBox for Shape {
    fn hitbox(&self) -> HitBox {
        match self {
            Shape::Segment(segment) => segment.hitbox(),
            Shape::Triangle(triangle) => triangle.hitbox(),
            Shape::Rect(rect) => rect.hitbox(),
            Shape::Circle(circle) => circle.hitbox(),
            Shape::Polygon(polygon) => polygon.hitbox(),
        }
    }
}

impl fmt::Display for Shape {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Shape::Segment(segment) => write!(f, "{}", segment),
            Shape::Triangle(triangle) => write!(f, "{}", triangle),
            Shape::Rect(rect) => write!(f, "{}", rect),
            Shape::Circle(circle) => write!(f, "{}", circle),
            Shape::Polygon(polygon) => write!(f, "{}", polygon),
        }
    }
}

/// notice that a and b are local positions, you may need to manually integrate them with a position
#[derive(Clone, Deserialize, Debug)]
pub struct Segment {
    pub a: Vec2,
    pub b: Vec2,
}

impl Segment {
    #[inline]
    pub fn new(a: Vec2, b: Vec2) -> Self {
        Self { a, b }
    }

    #[inline]
    pub fn eval_x(&self, x: f32) -> Option<f32> {
        if x < self.a.x.min(self.b.x) - EPS || x > self.a.x.max(self.b.x) + EPS {
            // out of range
            return None;
        };

        let delta_x = self.b.x - self.a.x;
        let delta_y = self.b.y - self.a.y;

        if delta_x.abs() <= EPS {
            // vertical line
            return None;
        }

        let m = delta_y / delta_x;
        let q = self.a.y - m * self.a.x;

        Some(x.mul_add(m, q))
    }

    #[inline]
    pub fn eval_y(&self, y: f32) -> Option<f32> {
        if y < self.a.y.min(self.b.y) - EPS || y > self.a.y.max(self.b.y) + EPS {
            // out of range
            return None;
        };

        let delta_x = self.b.x - self.a.x;
        let delta_y = self.b.y - self.a.y;

        if delta_x.abs() <= EPS {
            // vertical line
            return Some(self.a.x);
        }

        let m = delta_y / delta_x;
        let q = self.a.y - m * self.a.x;

        Some((y - q) / m)
    }
}

impl ToHitBox for Segment {
    #[inline]
    fn hitbox(&self) -> HitBox {
        let min_x = self.a.x.min(self.b.x);
        let min_y = self.a.y.min(self.b.y);
        let max_x = self.a.x.max(self.b.x);
        let max_y = self.a.y.max(self.b.y);

        HitBox::new(min_x, min_y, max_x, max_y)
    }
}

impl fmt::Display for Segment {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "segment ({:.4}, {:.4})", self.a, self.b)
    }
}

// #[derive(Clone, Debug)]
// pub struct SegmentCache {
//     pub a: Vec2,
//     pub b: Vec2,
//     pub delta_x: f32,
//     pub delta_y: f32,
//     pub m: Option<f32>,
//     pub q: Option<f32>,
//     pub c: f32,
//     pub pow_sum: f32,
// }

// impl SegmentCache {
//     #[inline]
//     pub fn new(a: Vec2, b: Vec2) -> Self {
//         let delta_x = b.x - a.x;
//         let delta_y = b.y - a.y;

//         let (m, q) = if delta_x.abs() <= EPS {
//             (None, None)
//         } else {
//             let m = delta_y / delta_x;
//             (Some(m), Some(a.y - m * a.x))
//         };

//         Self {
//             a: a,
//             b: b,
//             delta_x: delta_x,
//             delta_y: delta_y,
//             m: m,
//             q: q,
//             c: (b.x * a.y - b.y * a.x),
//             pow_sum: (pow2(delta_x) + pow2(delta_y)),
//         }
//     }

//     #[inline]
//     pub fn from(segment: Segment) -> Self {
//         Self::new(segment.a, segment.b)
//     }

//     #[inline]
//     pub fn eval_x(&self, x: f32) -> Option<f32> {
//         if x < self.a.x.min(self.b.x) - EPS || x > self.a.x.max(self.b.x) + EPS {
//             // out of range
//             return None;
//         };

//         match (self.m, self.q) {
//             (Some(m), Some(q)) => Some(x.mul_add(m, q)),
//             _ => None,
//         }
//     }

//     #[inline]
//     pub fn eval_y(&self, y: f32) -> Option<f32> {
//         if y < self.a.y.min(self.b.y) - EPS || y > self.a.y.max(self.b.y) + EPS {
//             // out of range
//             return None;
//         };

//         match (self.m, self.q) {
//             (Some(m), Some(q)) if m.abs() > EPS => Some((y - q) / m),
//             _ => None,
//         }
//     }

//     #[inline]
//     pub fn point_square_dist(&self, point: Vec2) -> f32 {
//         // vector from A to the point
//         let ap = Vec2::new(point.x - self.a.x, point.y - self.a.y);

//         // squared length of the segment
//         if self.pow_sum <= EPS_SQR {
//             // segment is a single point at A
//             return ap.square_len();
//         }

//         // projection factor of AP onto AB, normalized by |AB|^2
//         let t = ap.dot(Vec2::new(self.delta_x, self.delta_y)) / self.pow_sum;

//         if t < 0.0 {
//             // projection is before A → closest point is A
//             ap.square_len()
//         } else if t > 1.0 {
//             // projection is past B → closest point is B
//             Vec2::new(point.x - self.b.x, point.y - self.b.y).square_len()
//         } else {
//             // projection falls on the segment → use perpendicular distance
//             pow2(self.delta_y * point.x - self.delta_x * point.y + self.c) / self.pow_sum
//         }
//     }

//     #[inline]
//     pub fn rect_square_dist(&self, pos: Vec2, rect: Rect) -> (f32, f32, f32, f32) {
//         let tl = self.point_square_dist(pos);
//         let tr = self.point_square_dist(Vec2::new(pos.x + rect.width, pos.y));
//         let bl = self.point_square_dist(Vec2::new(pos.x, pos.y + rect.height));
//         let br = self.point_square_dist(Vec2::new(pos.x + rect.width, pos.y + rect.height));

//         (tl, tr, bl, br)
//     }

//     #[inline]
//     pub fn circle_square_dist(&self, pos: Vec2, circle: Circle) -> f32 {
//         self.point_square_dist(Vec2::new(pos.x + circle.radius, pos.y + circle.radius))
//     }
// }

/// notice that a, b and c are local positions, you may need to manually integrate them with a position
#[derive(Clone, Deserialize, Debug)]
pub struct Triangle {
    pub a: Vec2,
    pub b: Vec2,
    pub c: Vec2,
}

impl Triangle {
    #[inline]
    pub fn new(a: Vec2, b: Vec2, c: Vec2) -> Self {
        Self { a, b, c }
    }
}

impl ToHitBox for Triangle {
    #[inline]
    fn hitbox(&self) -> HitBox {
        let min_x = self.a.x.min(self.b.x.min(self.c.x));
        let min_y = self.a.y.min(self.b.y.min(self.c.y));
        let max_x = self.a.x.max(self.b.x.max(self.c.x));
        let max_y = self.a.y.max(self.b.y.max(self.c.y));

        HitBox::new(min_x, min_y, max_x, max_y)
    }
}

impl fmt::Display for Triangle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "triangle ({:.4}, {:.4}, {:.4})", self.a, self.b, self.c)
    }
}

#[derive(Clone, Deserialize, Debug)]
pub struct Rect {
    pub width: f32,
    pub height: f32,
}

impl Rect {
    #[inline]
    pub fn new(width: f32, height: f32) -> Self {
        if width <= 0.0 {
            panic!("width must be greater than 0")
        }

        if height <= 0.0 {
            panic!("height must be greater than 0")
        }

        Self { width, height }
    }
}

impl ToHitBox for Rect {
    #[inline]
    fn hitbox(&self) -> HitBox {
        HitBox::new(0.0, 0.0, self.width, self.height)
    }
}

impl fmt::Display for Rect {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "rectangle ({:.4}, {:.4})", self.width, self.height)
    }
}

#[derive(Clone, Deserialize, Debug)]
pub struct Circle {
    pub radius: f32,
}

impl Circle {
    #[inline]
    pub fn new(radius: f32) -> Self {
        if radius <= 0.0 {
            panic!("radius must be greater than 0")
        }

        Self { radius }
    }
}

impl ToHitBox for Circle {
    #[inline]
    fn hitbox(&self) -> HitBox {
        let diameter = self.radius * 2.0;
        HitBox::new(0.0, 0.0, diameter, diameter)
    }
}

impl fmt::Display for Circle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "circle ({:.4})", self.radius)
    }
}

/// polygons must be convex, vertices must be stored counterclockwise, and there must be no collinear edges
/// notice that vertices are local positions, you may need to manually integrate them with a position
#[derive(Clone, Deserialize, Debug)]
pub struct Polygon {
    pub verts: Vec<Vec2>,
}

impl Polygon {
    #[inline]
    pub fn new(verts: Vec<Vec2>) -> Self {
        let polygon = Self { verts };
        if !polygon.is_valid() {
            panic!("polygon is not valid");
        }

        polygon
    }

    pub fn is_valid(&self) -> bool {
        let verts_len = self.verts.len();

        if verts_len < 3 {
            panic!("cannot build a polygon with only {} vertices", verts_len);
        } else if verts_len == 3 {
            panic!("use Shape::Triangle as it is more efficient");
        }

        // check duplicates vertices
        // this uses squares instead of circles (as in square distance) for performance
        for i in 0..verts_len {
            for j in (i + 1)..verts_len {
                if (self.verts[i].x - self.verts[j].x).abs() < EPS && (self.verts[i].y - self.verts[j].y).abs() < EPS {
                    panic!("near-duplicate points");
                }
            }
        }

        // check if the polygon is convex
        for i in 0..verts_len {
            let i1 = (i + 1) % verts_len; // use modulo indexing to restart when the end is reached
            let i2 = (i + 2) % verts_len;

            let a = self.verts[i1].sub(self.verts[i]);
            let b = self.verts[i2].sub(self.verts[i1]);
            let cross = a.cross(b);

            if cross <= EPS {
                panic!(
                    "({}-{}) and ({}-{}) are collinear or clockwise but they must be counterclockwise",
                    i, i1, i1, i2
                );
            }
        }
        true
    }
}

impl ToHitBox for Polygon {
    #[inline]
    fn hitbox(&self) -> HitBox {
        // initialize extremes
        let mut min_x = f32::MAX;
        let mut min_y = f32::MAX;
        let mut max_x = f32::MIN;
        let mut max_y = f32::MIN;

        // update extremes
        for vert in &self.verts {
            min_x = min_x.min(vert.x);
            min_y = min_y.min(vert.y);
            max_x = max_x.max(vert.x);
            max_y = max_y.max(vert.y);
        }

        HitBox::new(min_x, min_y, max_x, max_y)
    }
}

impl fmt::Display for Polygon {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "polygon (")?;
        for (i, vert) in self.verts.iter().enumerate() {
            if i > 0 {
                write!(f, ", ")?;
            }
            write!(f, "{}", vert)?;
        }
        write!(f, ")")
    }
}

#[derive(Copy, Clone, Deserialize, Debug)]
pub struct Line {
    pub m: f32,
    pub q: f32,
}

impl Line {
    #[inline]
    pub fn new(m: f32, q: f32) -> Self {
        Self { m, q }
    }

    #[inline]
    pub fn from(segment: Segment) -> Self {
        let delta_x = segment.b.x - segment.a.x;
        let delta_y = segment.b.y - segment.a.y;

        let (m, q) = if delta_x.abs() <= EPS {
            (None, None)
        } else {
            let m = delta_y / delta_x;
            (Some(m), Some(segment.a.y - m * segment.a.x))
        };

        Self {
            m: m.expect("m is None"),
            q: q.expect("q is None"),
        }
    }
}

// OBJECTS

#[derive(Deserialize)]
pub struct StaticSpec {
    pub transform: TransformSpec,
    pub surface: SurfaceSpec,
    pub shape: Shape,
    pub material: MaterialSpec,
}

#[derive(Deserialize)]
pub struct DynamicSpec {
    pub transform: TransformSpec,
    pub rigid_body: RigidBodySpec,
    pub surface: SurfaceSpec,
    pub shape: Shape,
    pub material: MaterialSpec,
}
