use crate::{core::error, math};

use serde::Deserialize;
use std::fmt;

#[derive(Clone, Debug)]
pub struct HitBox {
    pub(crate) min_x: f32,
    pub(crate) min_y: f32,
    pub(crate) max_x: f32,
    pub(crate) max_y: f32,
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
    pub fn min_x(&self) -> f32 {
        self.min_x
    }

    #[inline]
    pub fn min_y(&self) -> f32 {
        self.min_y
    }

    #[inline]
    pub fn max_x(&self) -> f32 {
        self.max_x
    }

    #[inline]
    pub fn max_y(&self) -> f32 {
        self.max_y
    }

    #[inline]
    pub fn set_min_x(&mut self, new_min_x: f32) {
        self.min_x = new_min_x;
    }

    #[inline]
    pub fn set_min_y(&mut self, new_min_y: f32) {
        self.min_y = new_min_y;
    }

    #[inline]
    pub fn set_max_x(&mut self, new_max_x: f32) {
        self.max_x = new_max_x;
    }

    #[inline]
    pub fn set_max_y(&mut self, new_max_y: f32) {
        self.max_y = new_max_y;
    }

    #[inline]
    pub fn add_pos(self, pos: math::Vec2) -> Self {
        Self::new(
            self.min_x + pos.x,
            self.min_y + pos.y,
            self.max_x + pos.x,
            self.max_y + pos.y,
        )
    }

    #[inline]
    pub fn add_pos_mut(&mut self, pos: math::Vec2) {
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
    Unmoved { shape: &'a Shape, pos: math::Vec2 },
    AxisRect { swept: Rect, pos: math::Vec2 },
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

impl Shape {
    #[inline]
    pub fn validate(&self) -> Result<(), error::EngineError> {
        match self {
            Shape::Segment(segment) => segment.validate()?,
            Shape::Triangle(triangle) => triangle.validate()?,
            Shape::Rect(rect) => rect.validate()?,
            Shape::Circle(circle) => circle.validate()?,
            Shape::Polygon(polygon) => polygon.validate()?,
        };

        Ok(())
    }
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
    pub(crate) a: math::Vec2,
    pub(crate) b: math::Vec2,
}

impl Segment {
    #[inline]
    pub fn new(a: math::Vec2, b: math::Vec2) -> Result<Self, error::GeometryError> {
        let segment = Self { a, b };

        segment.validate()?;

        Ok(segment)
    }

    #[inline]
    pub fn validate(&self) -> Result<(), error::GeometryError> {
        // check duplicates vertices
        if self.a.square_dist(self.b) < math::EPS_SQR {
            return Err(error::GeometryError::DuplicateVertices);
        };

        Ok(())
    }

    #[inline]
    pub fn a(&self) -> math::Vec2 {
        self.a
    }

    #[inline]
    pub fn b(&self) -> math::Vec2 {
        self.b
    }

    #[inline]
    pub fn set_a(&mut self, new_a: math::Vec2) {
        self.a = new_a;
    }

    #[inline]
    pub fn set_b(&mut self, new_b: math::Vec2) {
        self.b = new_b;
    }

    #[inline]
    pub fn eval_x(&self, x: f32) -> Option<f32> {
        if x < self.a.x.min(self.b.x) - math::EPS || x > self.a.x.max(self.b.x) + math::EPS {
            // out of range
            return None;
        };

        let delta_x = self.b.x - self.a.x;
        let delta_y = self.b.y - self.a.y;

        if delta_x.abs() <= math::EPS {
            // vertical line
            return None;
        };

        let m = delta_y / delta_x;
        let q = self.a.y - m * self.a.x;

        Some(x.mul_add(m, q))
    }

    #[inline]
    pub fn eval_y(&self, y: f32) -> Option<f32> {
        if y < self.a.y.min(self.b.y) - math::EPS || y > self.a.y.max(self.b.y) + math::EPS {
            // out of range
            return None;
        };

        let delta_x = self.b.x - self.a.x;
        let delta_y = self.b.y - self.a.y;

        if delta_x.abs() <= math::EPS {
            // vertical line
            return Some(self.a.x);
        };

        if delta_y.abs() <= math::EPS {
            // horizontal line
            return None;
        };

        let m = delta_y / delta_x;
        let q = self.a.y - m * self.a.x;

        Some((y - q) / m) // m should never be 0 since delta_y is never 0
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
//     pub a: math::Vec2,
//     pub b: math::Vec2,
//     pub delta_x: f32,
//     pub delta_y: f32,
//     pub m: Option<f32>,
//     pub q: Option<f32>,
//     pub c: f32,
//     pub pow_sum: f32,
// }

// impl SegmentCache {
//     #[inline]
//     pub fn new(a: math::Vec2, b: math::Vec2) -> Self {
//         let delta_x = b.x - a.x;
//         let delta_y = b.y - a.y;

//         let (m, q) = if delta_x.abs() <= math::EPS {
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
//         if x < self.a.x.min(self.b.x) - math::EPS || x > self.a.x.max(self.b.x) + math::EPS {
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
//         if y < self.a.y.min(self.b.y) - math::EPS || y > self.a.y.max(self.b.y) + math::EPS {
//             // out of range
//             return None;
//         };

//         match (self.m, self.q) {
//             (Some(m), Some(q)) if m.abs() > math::EPS => Some((y - q) / m),
//             _ => None,
//         }
//     }

//     #[inline]
//     pub fn point_square_dist(&self, point: math::Vec2) -> f32 {
//         // vector from A to the point
//         let ap = math::Vec2::new(point.x - self.a.x, point.y - self.a.y);

//         // squared length of the segment
//         if self.pow_sum <= math::math::EPS_SQR {
//             // segment is a single point at A
//             return ap.square_len();
//         }

//         // projection factor of AP onto AB, normalized by |AB|^2
//         let t = ap.dot(math::Vec2::new(self.delta_x, self.delta_y)) / self.pow_sum;

//         if t < 0.0 {
//             // projection is before A → closest point is A
//             ap.square_len()
//         } else if t > 1.0 {
//             // projection is past B → closest point is B
//             math::Vec2::new(point.x - self.b.x, point.y - self.b.y).square_len()
//         } else {
//             // projection falls on the segment → use perpendicular distance
//             pow2(self.delta_y * point.x - self.delta_x * point.y + self.c) / self.pow_sum
//         }
//     }

//     #[inline]
//     pub fn rect_square_dist(&self, pos: math::Vec2, rect: Rect) -> (f32, f32, f32, f32) {
//         let tl = self.point_square_dist(pos);
//         let tr = self.point_square_dist(math::Vec2::new(pos.x + rect.width, pos.y));
//         let bl = self.point_square_dist(math::Vec2::new(pos.x, pos.y + rect.height));
//         let br = self.point_square_dist(math::Vec2::new(pos.x + rect.width, pos.y + rect.height));

//         (tl, tr, bl, br)
//     }

//     #[inline]
//     pub fn circle_square_dist(&self, pos: math::Vec2, circle: Circle) -> f32 {
//         self.point_square_dist(math::Vec2::new(pos.x + circle.radius, pos.y + circle.radius))
//     }
// }

/// notice that a, b and c are local positions, you may need to manually integrate them with a position
#[derive(Clone, Deserialize, Debug)]
pub struct Triangle {
    pub(crate) a: math::Vec2,
    pub(crate) b: math::Vec2,
    pub(crate) c: math::Vec2,
}

impl Triangle {
    #[inline]
    pub fn new(a: math::Vec2, b: math::Vec2, c: math::Vec2) -> Result<Self, error::GeometryError> {
        let triangle = Self { a, b, c };

        triangle.validate()?;

        Ok(triangle)
    }

    #[inline]
    pub fn validate(&self) -> Result<(), error::GeometryError> {
        // check duplicates vertices
        if self.a.square_dist(self.b) < math::EPS_SQR
            || self.a.square_dist(self.c) < math::EPS_SQR
            || self.b.square_dist(self.c) < math::EPS_SQR
        {
            return Err(error::GeometryError::DuplicateVertices);
        };

        Ok(())
    }

    #[inline]
    pub fn a(&self) -> math::Vec2 {
        self.a
    }

    #[inline]
    pub fn b(&self) -> math::Vec2 {
        self.b
    }

    #[inline]
    pub fn c(&self) -> math::Vec2 {
        self.c
    }

    #[inline]
    pub fn set_a(&mut self, new_a: math::Vec2) {
        self.a = new_a;
    }

    #[inline]
    pub fn set_b(&mut self, new_b: math::Vec2) {
        self.b = new_b;
    }

    #[inline]
    pub fn set_c(&mut self, new_c: math::Vec2) {
        self.c = new_c;
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
    pub(crate) width: f32,
    pub(crate) height: f32,
}

impl Rect {
    #[inline]
    pub fn new(width: f32, height: f32) -> Result<Self, error::MathError> {
        let rect = Self { width, height };

        rect.validate()?;

        Ok(rect)
    }

    #[inline]
    pub fn validate(&self) -> Result<(), error::MathError> {
        if self.width <= 0.0 {
            return Err(error::MathError::NonPositive("width"));
        }

        if self.height <= 0.0 {
            return Err(error::MathError::NonPositive("height"));
        }

        Ok(())
    }

    #[inline]
    pub fn width(&self) -> f32 {
        self.width
    }

    #[inline]
    pub fn height(&self) -> f32 {
        self.height
    }

    #[inline]
    pub fn set_width(&mut self, new_width: f32) {
        self.width = new_width;
    }

    #[inline]
    pub fn set_height(&mut self, new_height: f32) {
        self.height = new_height;
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
    pub(crate) radius: f32,
}

impl Circle {
    #[inline]
    pub fn new(radius: f32) -> Result<Self, error::MathError> {
        let circle = Self { radius };

        circle.validate()?;

        Ok(circle)
    }

    #[inline]
    pub fn validate(&self) -> Result<(), error::MathError> {
        if self.radius <= 0.0 {
            return Err(error::MathError::NonPositive("radius"));
        }

        Ok(())
    }

    #[inline]
    pub fn radius(&self) -> f32 {
        self.radius
    }

    #[inline]
    pub fn set_radius(&mut self, new_radius: f32) {
        self.radius = new_radius;
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
    pub(crate) verts: Vec<math::Vec2>,
}

impl Polygon {
    #[inline]
    pub fn new(verts: Vec<math::Vec2>) -> Result<Self, error::GeometryError> {
        let polygon = Self { verts };

        polygon.validate()?;

        Ok(polygon)
    }

    pub fn validate(&self) -> Result<(), error::GeometryError> {
        let verts_len = self.verts.len();

        if verts_len < 3 {
            return Err(error::GeometryError::TooFewVertices(verts_len));
        } else if verts_len == 3 {
            eprintln!("warning: polygon with 3 vertices, consider Shape::Triangle for efficiency");
        }

        // check duplicates vertices
        for i in 0..verts_len {
            for j in (i + 1)..verts_len {
                if self.verts[i].square_dist(self.verts[j]) < math::EPS_SQR {
                    return Err(error::GeometryError::DuplicateVertices);
                }
            }
        }

        // check if the polygon is convex
        for i in 0..verts_len {
            let i1 = (i + 1) % verts_len; // use modulo indexing to restart when the end is reached
            let i2 = (i + 2) % verts_len;

            let area = self.verts[i].signed_area(self.verts[i1], self.verts[i2]);

            if area >= -math::EPS {
                return Err(error::GeometryError::NotConvex);
            }
        }

        Ok(())
    }

    #[inline]
    pub fn verts(&self) -> &Vec<math::Vec2> {
        &self.verts
    }

    #[inline]
    pub fn verts_mut(&mut self) -> &mut Vec<math::Vec2> {
        &mut self.verts
    }

    #[inline]
    pub fn set_verts(&mut self, new_verts: Vec<math::Vec2>) {
        self.verts = new_verts;
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

// #[derive(Copy, Clone, Deserialize, Debug)]
// pub struct Line {
//     pub m: f32,
//     pub q: f32,
// }

// impl Line {
//     #[inline]
//     pub fn new(m: f32, q: f32) -> Self {
//         Self { m, q }
//     }

//     #[inline]
//     pub fn from(segment: Segment) -> Self {
//         let delta_x = segment.b.x - segment.a.x;
//         let delta_y = segment.b.y - segment.a.y;

//         let (m, q) = if delta_x.abs() <= math::EPS {
//             (None, None)
//         } else {
//             let m = delta_y / delta_x;
//             (Some(m), Some(segment.a.y - m * segment.a.x))
//         };

//         Self {
//             m: m.expect("m is None"),
//             q: q.expect("q is None"),
//         }
//     }
// }
