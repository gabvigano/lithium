use serde::Deserialize;

use crate::ecs::systems::physics::{EPS, pow2};

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
    pub fn add(self, vec2: Self) -> Self {
        Self::new(self.x + vec2.x, self.y + vec2.y)
    }

    #[inline]
    pub fn add_inplace(&mut self, vec2: Self) {
        self.x += vec2.x;
        self.y += vec2.y;
    }

    #[inline]
    pub fn add_scalar(self, x: f32, y: f32) -> Self {
        Self::new(self.x + x, self.y + y)
    }

    #[inline]
    pub fn add_scalar_inplace(&mut self, x: f32, y: f32) {
        self.x += x;
        self.y += y;
    }

    #[inline]
    pub fn sub(self, vec2: Self) -> Self {
        Self::new(self.x - vec2.x, self.y - vec2.y)
    }

    #[inline]
    pub fn sub_inplace(&mut self, vec2: Self) {
        self.x -= vec2.x;
        self.y -= vec2.y;
    }

    #[inline]
    pub fn sub_scalar(self, x: f32, y: f32) -> Self {
        Self::new(self.x - x, self.y - y)
    }

    #[inline]
    pub fn sub_scalar_inplace(&mut self, x: f32, y: f32) {
        self.x -= x;
        self.y -= y;
    }

    #[inline]
    pub fn mul(self, vec2: Self) -> Self {
        Self::new(self.x * vec2.x, self.y * vec2.y)
    }

    #[inline]
    pub fn mul_inplace(&mut self, vec2: Self) {
        self.x *= vec2.x;
        self.y *= vec2.y;
    }

    #[inline]
    pub fn div(self, vec2: Self) -> Self {
        Self::new(self.x / vec2.x, self.y / vec2.y)
    }

    #[inline]
    pub fn div_inplace(&mut self, vec2: Self) {
        self.x /= vec2.x;
        self.y /= vec2.y;
    }

    #[inline]
    pub fn scale(self, scalar: f32) -> Self {
        Self::new(self.x * scalar, self.y * scalar)
    }

    #[inline]
    pub fn scale_inplace(&mut self, scalar: f32) {
        self.x *= scalar;
        self.y *= scalar;
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
    pub fn len(self) -> f32 {
        self.square_len().sqrt()
    }

    #[inline]
    pub fn square_len(self) -> f32 {
        pow2(self.x) + pow2(self.y)
    }
}

#[derive(Clone, Deserialize, Debug)]
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

#[derive(Clone, Deserialize, Debug)]
pub enum Dir {
    Axis(Axis),
    Angle(Angle),
}

#[derive(Copy, Clone, Deserialize, Debug)]
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

#[derive(Clone, Deserialize, Debug)]
pub struct RigidBody {
    pub vel: Vec2,
    pub acc: Vec2,
    pub mass: f32,
    pub rest: bool,
}

impl RigidBody {
    #[inline]
    pub fn new(vel: Vec2, acc: Vec2, mass: f32, rest: bool) -> Self {
        Self { vel, acc, mass, rest }
    }

    #[inline]
    pub fn reset_vel(&mut self, new_vel: Vec2) {
        self.vel = new_vel;
    }

    #[inline]
    pub fn reset_acc(&mut self, new_acc: Vec2) {
        self.acc = new_acc;
    }
}

#[derive(Clone, Deserialize, Debug)]
pub struct Collider {
    pub hitbox: HitBox,
    pub corrupted: bool,
    pub elast: Elast,
}

impl Collider {
    #[inline]
    pub fn new(hitbox: HitBox, elast: Elast) -> Self {
        Self {
            hitbox,
            corrupted: true,
            elast,
        }
    }
}

pub type HitBox = Rect;

pub trait ToHitBox {
    fn hitbox(&self) -> HitBox;
}

pub type Elast = f32;

#[derive(Clone, Deserialize, Debug)]
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
        let delta_x = (self.b.x - self.a.x).abs().max(EPS);
        let delta_y = (self.b.y - self.a.y).abs().max(EPS);

        HitBox::new(delta_x, delta_y)
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
        let max_x = self.a.x.max(self.b.x.max(self.c.x));

        let min_y = self.a.y.min(self.b.y.min(self.c.y));
        let max_y = self.a.y.max(self.b.y.max(self.c.y));

        let delta_x = max_x - min_x;
        let delta_y = max_y - min_y;

        HitBox::new(delta_x, delta_y)
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
        self.clone()
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
        HitBox::new(self.radius * 2.0, self.radius * 2.0)
    }
}

#[derive(Clone, Deserialize, Debug)]
pub struct Polygon {
    pub verts: Vec<Vec2>,
}

/// notice that vertices are local positions, you may need to manually integrate them with a position
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
        let mut positive_sign = true; // it should be always reinitialize, I'm assigning it here so the compiler doesn't complain
        for i in 0..verts_len {
            let i1 = (i + 1) % verts_len; // use modulo indexing to restart when the end is reached
            let i2 = (i + 2) % verts_len;

            let a = self.verts[i1].sub(self.verts[i]);
            let b = self.verts[i2].sub(self.verts[i1]);
            let cross = a.cross(b);

            if i == 0 {
                // first iteration, define the sign
                positive_sign = if cross > EPS {
                    true
                } else if cross < -EPS {
                    false
                } else {
                    panic!("({};{}) and ({};{}) are collinear", i, i1, i1, i2)
                };
            } else {
                if (positive_sign && cross < -EPS) || (!positive_sign && cross > EPS) {
                    // wrong sign
                    panic!(
                        "cross between ({};{}) and ({};{}) has the wrong sign (expected {}, got {})",
                        i,
                        i1,
                        i1,
                        i2,
                        if positive_sign { "positive" } else { "negative" },
                        cross
                    );
                }
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
        let mut max_x = f32::MIN;
        let mut min_y = f32::MAX;
        let mut max_y = f32::MIN;

        // update extremes
        for v in &self.verts {
            if v.x < min_x {
                min_x = v.x;
            }
            if v.x > max_x {
                max_x = v.x;
            }
            if v.y < min_y {
                min_y = v.y;
            }
            if v.y > max_y {
                max_y = v.y;
            }
        }

        let delta_x = max_x - min_x;
        let delta_y = max_y - min_y;

        HitBox::new(delta_x, delta_y)
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

#[derive(Clone, Debug)]
pub struct Force {
    pub mag: f32,
    pub dir: Dir,
}

impl Force {
    #[inline]
    pub fn new(mag: f32, dir: Dir) -> Self {
        Self { mag, dir }
    }
}

// OBJECTS

#[derive(Deserialize)]
pub struct Static {
    pub transform: Transform,
    pub collider: Collider,
    pub shape: Shape,
    pub material: Material,
}

#[derive(Deserialize)]
pub struct Dynamic {
    pub transform: Transform,
    pub rigid_body: RigidBody,
    pub collider: Collider,
    pub shape: Shape,
    pub material: Material,
}
