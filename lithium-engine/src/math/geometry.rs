use crate::{
    core::error,
    math::{self, Vec2},
};

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
    pub fn from_verts_array<const N: usize>(verts: &[math::Vec2; N]) -> Self {
        // initialize extremes
        let first = verts[0];

        let mut min_x = first.x;
        let mut min_y = first.y;
        let mut max_x = first.x;
        let mut max_y = first.y;

        // update extremes (skip first element since the extremes were initialized to that)
        for i in 1..N {
            let vert = verts[i];
            min_x = min_x.min(vert.x);
            min_y = min_y.min(vert.y);
            max_x = max_x.max(vert.x);
            max_y = max_y.max(vert.y);
        }

        Self::new(min_x, min_y, max_x, max_y)
    }

    #[inline]
    pub fn from_verts_slice(verts: &[math::Vec2]) -> Self {
        // initialize extremes
        let first = verts[0];

        let mut min_x = first.x;
        let mut min_y = first.y;
        let mut max_x = first.x;
        let mut max_y = first.y;

        // update extremes (skip first element since the extremes were initialized to that)
        for vert in &verts[1..] {
            min_x = min_x.min(vert.x);
            min_y = min_y.min(vert.y);
            max_x = max_x.max(vert.x);
            max_y = max_y.max(vert.y);
        }

        Self::new(min_x, min_y, max_x, max_y)
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

pub trait Validate {
    fn validate(&self) -> Result<(), error::GeometryError>;
}

pub trait ApplyTransformationVerts {
    type Output;
    fn apply_vec2(&self, vec: math::Vec2) -> Self::Output;
    fn apply_mat2x3(&self, mat: &math::Mat2x3) -> Self::Output;
    fn apply_mat2x3_then_vec2(&self, vec: math::Vec2, mat: &math::Mat2x3) -> Self::Output;
}

pub trait ApplyTransformationVertsStep {
    type Output;
    fn apply_vec2_step(&self, vec_1: math::Vec2, vec_2: math::Vec2) -> Self::Output;
    fn apply_mat2x3_step(&self, mat_1: &math::Mat2x3, mat_2: &math::Mat2x3) -> Self::Output;
    fn apply_mat2x3_then_vec2_step(
        &self,
        vec_1: math::Vec2,
        vec_2: math::Vec2,
        mat_1: &math::Mat2x3,
        mat_2: &math::Mat2x3,
    ) -> Self::Output;
}

pub trait ApplyTransformationShape {
    fn apply_vec2_checked(&self, vec: math::Vec2) -> Result<Self, error::GeometryError>
    where
        Self: Sized;
    fn apply_vec2_unchecked(&self, vec: math::Vec2) -> Self;
    fn apply_mat2x3_checked(&self, mat: &math::Mat2x3) -> Result<Self, error::GeometryError>
    where
        Self: Sized;
    fn apply_mat2x3_unchecked(&self, mat: &math::Mat2x3) -> Self;
}

pub trait ToHitBox {
    fn to_hitbox(&self) -> HitBox;
}

#[derive(Clone, Debug)]
pub enum SweptShape {
    Unchanged(Shape),
    Changed(Polygon),
}

impl SweptShape {
    #[inline]
    pub fn sides(&self) -> usize {
        match self {
            SweptShape::Unchanged(shape) => match shape {
                Shape::Segment(_) => 1,
                Shape::Triangle(_) => 3,
                Shape::Quad(_) => 4,
                Shape::Polygon(polygon) => polygon.verts.len(),
                Shape::Circle(_) => unimplemented!(),
            },
            SweptShape::Changed(polygon) => polygon.verts.len(),
        }
    }
}

#[derive(Clone, Deserialize, Debug)]
pub enum Shape {
    Segment(Segment),
    Triangle(Triangle),
    Quad(Quad),
    Polygon(Polygon),
    Circle(Circle),
}

impl Validate for Shape {
    #[inline]
    fn validate(&self) -> Result<(), error::GeometryError> {
        match self {
            Shape::Segment(segment) => segment.validate()?,
            Shape::Triangle(triangle) => triangle.validate()?,
            Shape::Quad(quad) => quad.validate()?,
            Shape::Polygon(polygon) => polygon.validate()?,
            Shape::Circle(_) => unimplemented!(),
        };

        Ok(())
    }
}

impl ApplyTransformationShape for Shape {
    #[inline]
    fn apply_vec2_checked(&self, vec: math::Vec2) -> Result<Self, error::GeometryError> {
        Ok(match self {
            Shape::Segment(segment) => Shape::Segment(segment.apply_vec2_checked(vec)?),
            Shape::Triangle(triangle) => Shape::Triangle(triangle.apply_vec2_checked(vec)?),
            Shape::Quad(quad) => Shape::Quad(quad.apply_vec2_checked(vec)?),
            Shape::Polygon(polygon) => Shape::Polygon(polygon.apply_vec2_checked(vec)?),
            Shape::Circle(_) => unimplemented!(),
        })
    }

    #[inline]
    fn apply_vec2_unchecked(&self, vec: math::Vec2) -> Self {
        match self {
            Shape::Segment(segment) => Shape::Segment(segment.apply_vec2_unchecked(vec)),
            Shape::Triangle(triangle) => Shape::Triangle(triangle.apply_vec2_unchecked(vec)),
            Shape::Quad(quad) => Shape::Quad(quad.apply_vec2_unchecked(vec)),
            Shape::Polygon(polygon) => Shape::Polygon(polygon.apply_vec2_unchecked(vec)),
            Shape::Circle(_) => unimplemented!(),
        }
    }

    #[inline]
    fn apply_mat2x3_checked(&self, mat: &math::Mat2x3) -> Result<Self, error::GeometryError> {
        Ok(match self {
            Shape::Segment(segment) => Shape::Segment(segment.apply_mat2x3_checked(mat)?),
            Shape::Triangle(triangle) => Shape::Triangle(triangle.apply_mat2x3_checked(mat)?),
            Shape::Quad(quad) => Shape::Quad(quad.apply_mat2x3_checked(mat)?),
            Shape::Polygon(polygon) => Shape::Polygon(polygon.apply_mat2x3_checked(mat)?),
            Shape::Circle(_) => unimplemented!(),
        })
    }

    #[inline]
    fn apply_mat2x3_unchecked(&self, mat: &math::Mat2x3) -> Self {
        match self {
            Shape::Segment(segment) => Shape::Segment(segment.apply_mat2x3_unchecked(mat)),
            Shape::Triangle(triangle) => Shape::Triangle(triangle.apply_mat2x3_unchecked(mat)),
            Shape::Quad(quad) => Shape::Quad(quad.apply_mat2x3_unchecked(mat)),
            Shape::Polygon(polygon) => Shape::Polygon(polygon.apply_mat2x3_unchecked(mat)),
            Shape::Circle(_) => unimplemented!(),
        }
    }
}

impl ToHitBox for Shape {
    #[inline]
    fn to_hitbox(&self) -> HitBox {
        match self {
            Shape::Segment(segment) => segment.to_hitbox(),
            Shape::Triangle(triangle) => triangle.to_hitbox(),
            Shape::Quad(quad) => quad.to_hitbox(),
            Shape::Polygon(polygon) => polygon.to_hitbox(),
            Shape::Circle(circle) => circle.to_hitbox(),
        }
    }
}

impl fmt::Display for Shape {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Shape::Segment(segment) => write!(f, "{}", segment),
            Shape::Triangle(triangle) => write!(f, "{}", triangle),
            Shape::Quad(quad) => write!(f, "{}", quad),
            Shape::Polygon(polygon) => write!(f, "{}", polygon),
            Shape::Circle(circle) => write!(f, "{}", circle),
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
    pub fn new_unchecked(a: math::Vec2, b: math::Vec2) -> Self {
        Self { a, b }
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

impl Validate for Segment {
    #[inline]
    fn validate(&self) -> Result<(), error::GeometryError> {
        // check duplicates vertices
        if self.a.square_dist(self.b) < math::EPS_SQR {
            return Err(error::GeometryError::DuplicateVertices);
        };

        Ok(())
    }
}

impl ApplyTransformationVerts for Segment {
    type Output = [Vec2; 2];

    #[inline]
    fn apply_vec2(&self, vec: math::Vec2) -> Self::Output {
        [self.a.add(vec), self.b.add(vec)]
    }

    #[inline]
    fn apply_mat2x3(&self, mat: &math::Mat2x3) -> Self::Output {
        [mat.pre_mul_vec2(self.a), mat.pre_mul_vec2(self.b)]
    }

    #[inline]
    fn apply_mat2x3_then_vec2(&self, vec: math::Vec2, mat: &math::Mat2x3) -> Self::Output {
        [mat.pre_mul_vec2(self.a).add(vec), mat.pre_mul_vec2(self.b).add(vec)]
    }
}

impl ApplyTransformationVertsStep for Segment {
    type Output = [Vec2; 4];

    #[inline]
    fn apply_vec2_step(&self, vec_1: math::Vec2, vec_2: math::Vec2) -> Self::Output {
        [
            self.a.add(vec_1),
            self.b.add(vec_1),
            self.a.add(vec_2),
            self.b.add(vec_2),
        ]
    }

    #[inline]
    fn apply_mat2x3_step(&self, mat_1: &math::Mat2x3, mat_2: &math::Mat2x3) -> Self::Output {
        [
            mat_1.pre_mul_vec2(self.a),
            mat_1.pre_mul_vec2(self.b),
            mat_2.pre_mul_vec2(self.a),
            mat_2.pre_mul_vec2(self.b),
        ]
    }

    #[inline]
    fn apply_mat2x3_then_vec2_step(
        &self,
        vec_1: math::Vec2,
        vec_2: math::Vec2,
        mat_1: &math::Mat2x3,
        mat_2: &math::Mat2x3,
    ) -> Self::Output {
        [
            mat_1.pre_mul_vec2(self.a).add(vec_1),
            mat_1.pre_mul_vec2(self.b).add(vec_1),
            mat_2.pre_mul_vec2(self.a).add(vec_2),
            mat_2.pre_mul_vec2(self.b).add(vec_2),
        ]
    }
}

impl ApplyTransformationShape for Segment {
    #[inline]
    fn apply_vec2_checked(&self, vec: math::Vec2) -> Result<Self, error::GeometryError>
    where
        Self: Sized,
    {
        Self::new(self.a.add(vec), self.b.add(vec))
    }

    #[inline]
    fn apply_vec2_unchecked(&self, vec: math::Vec2) -> Self {
        Self::new_unchecked(self.a.add(vec), self.b.add(vec))
    }

    #[inline]
    fn apply_mat2x3_checked(&self, mat: &math::Mat2x3) -> Result<Self, error::GeometryError>
    where
        Self: Sized,
    {
        Self::new(mat.pre_mul_vec2(self.a), mat.pre_mul_vec2(self.b))
    }

    #[inline]
    fn apply_mat2x3_unchecked(&self, mat: &math::Mat2x3) -> Self {
        Self::new_unchecked(mat.pre_mul_vec2(self.a), mat.pre_mul_vec2(self.b))
    }
}

impl ToHitBox for Segment {
    #[inline]
    fn to_hitbox(&self) -> HitBox {
        HitBox::from_verts_array(&[self.a, self.b])
    }
}

impl fmt::Display for Segment {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "segment ({}, {})", self.a, self.b)
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
    pub fn new_unchecked(a: math::Vec2, b: math::Vec2, c: math::Vec2) -> Self {
        Self { a, b, c }
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

impl Validate for Triangle {
    #[inline]
    fn validate(&self) -> Result<(), error::GeometryError> {
        // check duplicates vertices
        if self.a.square_dist(self.b) < math::EPS_SQR
            || self.a.square_dist(self.c) < math::EPS_SQR
            || self.b.square_dist(self.c) < math::EPS_SQR
        {
            return Err(error::GeometryError::DuplicateVertices);
        };

        Ok(())
    }
}

impl ApplyTransformationVerts for Triangle {
    type Output = [Vec2; 3];

    #[inline]
    fn apply_vec2(&self, vec: math::Vec2) -> Self::Output {
        [self.a.add(vec), self.b.add(vec), self.c.add(vec)]
    }

    #[inline]
    fn apply_mat2x3(&self, mat: &math::Mat2x3) -> Self::Output {
        [
            mat.pre_mul_vec2(self.a),
            mat.pre_mul_vec2(self.b),
            mat.pre_mul_vec2(self.c),
        ]
    }

    #[inline]
    fn apply_mat2x3_then_vec2(&self, vec: math::Vec2, mat: &math::Mat2x3) -> Self::Output {
        [
            mat.pre_mul_vec2(self.a).add(vec),
            mat.pre_mul_vec2(self.b).add(vec),
            mat.pre_mul_vec2(self.c).add(vec),
        ]
    }
}

impl ApplyTransformationVertsStep for Triangle {
    type Output = [Vec2; 6];

    #[inline]
    fn apply_vec2_step(&self, vec_1: math::Vec2, vec_2: math::Vec2) -> Self::Output {
        [
            self.a.add(vec_1),
            self.b.add(vec_1),
            self.c.add(vec_1),
            self.a.add(vec_2),
            self.b.add(vec_2),
            self.c.add(vec_2),
        ]
    }

    #[inline]
    fn apply_mat2x3_step(&self, mat_1: &math::Mat2x3, mat_2: &math::Mat2x3) -> Self::Output {
        [
            mat_1.pre_mul_vec2(self.a),
            mat_1.pre_mul_vec2(self.b),
            mat_1.pre_mul_vec2(self.c),
            mat_2.pre_mul_vec2(self.a),
            mat_2.pre_mul_vec2(self.b),
            mat_2.pre_mul_vec2(self.c),
        ]
    }

    #[inline]
    fn apply_mat2x3_then_vec2_step(
        &self,
        vec_1: math::Vec2,
        vec_2: math::Vec2,
        mat_1: &math::Mat2x3,
        mat_2: &math::Mat2x3,
    ) -> Self::Output {
        [
            mat_1.pre_mul_vec2(self.a).add(vec_1),
            mat_1.pre_mul_vec2(self.b).add(vec_1),
            mat_1.pre_mul_vec2(self.c).add(vec_1),
            mat_2.pre_mul_vec2(self.a).add(vec_2),
            mat_2.pre_mul_vec2(self.b).add(vec_2),
            mat_2.pre_mul_vec2(self.c).add(vec_2),
        ]
    }
}

impl ApplyTransformationShape for Triangle {
    #[inline]
    fn apply_vec2_checked(&self, vec: math::Vec2) -> Result<Self, error::GeometryError>
    where
        Self: Sized,
    {
        Self::new(self.a.add(vec), self.b.add(vec), self.c.add(vec))
    }

    #[inline]
    fn apply_vec2_unchecked(&self, vec: math::Vec2) -> Self {
        Self::new_unchecked(self.a.add(vec), self.b.add(vec), self.c.add(vec))
    }

    #[inline]
    fn apply_mat2x3_checked(&self, mat: &math::Mat2x3) -> Result<Self, error::GeometryError>
    where
        Self: Sized,
    {
        Self::new(
            mat.pre_mul_vec2(self.a),
            mat.pre_mul_vec2(self.b),
            mat.pre_mul_vec2(self.c),
        )
    }

    #[inline]
    fn apply_mat2x3_unchecked(&self, mat: &math::Mat2x3) -> Self {
        Self::new_unchecked(
            mat.pre_mul_vec2(self.a),
            mat.pre_mul_vec2(self.b),
            mat.pre_mul_vec2(self.c),
        )
    }
}

impl ToHitBox for Triangle {
    #[inline]
    fn to_hitbox(&self) -> HitBox {
        HitBox::from_verts_array(&[self.a, self.b, self.c])
    }
}

impl fmt::Display for Triangle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "triangle ({}, {}, {})", self.a, self.b, self.c)
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
    fn to_hitbox(&self) -> HitBox {
        HitBox::new(0.0, 0.0, self.width, self.height)
    }
}

impl fmt::Display for Rect {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "rectangle ({:.4}, {:.4})", self.width, self.height)
    }
}

/// notice that a, b, c and d are local positions, you may need to manually integrate them with a position
#[derive(Clone, Deserialize, Debug)]
pub struct Quad {
    pub(crate) a: math::Vec2,
    pub(crate) b: math::Vec2,
    pub(crate) c: math::Vec2,
    pub(crate) d: math::Vec2,
}

impl Quad {
    #[inline]
    pub fn new(a: math::Vec2, b: math::Vec2, c: math::Vec2, d: math::Vec2) -> Result<Self, error::GeometryError> {
        let quad = Self { a, b, c, d };

        quad.validate()?;

        Ok(quad)
    }

    #[inline]
    pub fn new_unchecked(a: math::Vec2, b: math::Vec2, c: math::Vec2, d: math::Vec2) -> Self {
        Self { a, b, c, d }
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
    pub fn d(&self) -> math::Vec2 {
        self.d
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

    #[inline]
    pub fn set_d(&mut self, new_d: math::Vec2) {
        self.d = new_d;
    }
}

impl Validate for Quad {
    #[inline]
    fn validate(&self) -> Result<(), error::GeometryError> {
        // check duplicates vertices
        if self.a.square_dist(self.b) < math::EPS_SQR
            || self.a.square_dist(self.c) < math::EPS_SQR
            || self.a.square_dist(self.d) < math::EPS_SQR
            || self.b.square_dist(self.c) < math::EPS_SQR
            || self.b.square_dist(self.d) < math::EPS_SQR
            || self.c.square_dist(self.d) < math::EPS_SQR
        {
            return Err(error::GeometryError::DuplicateVertices);
        };

        // check if the quadrilateral is convex
        if self.a.signed_area(self.b, self.c) >= -math::EPS
            || self.b.signed_area(self.c, self.d) >= -math::EPS
            || self.c.signed_area(self.d, self.a) >= -math::EPS
            || self.d.signed_area(self.a, self.b) >= -math::EPS
        {
            return Err(error::GeometryError::NotConvex);
        }

        Ok(())
    }
}

impl ApplyTransformationVerts for Quad {
    type Output = [Vec2; 4];

    #[inline]
    fn apply_vec2(&self, vec: math::Vec2) -> Self::Output {
        [self.a.add(vec), self.b.add(vec), self.c.add(vec), self.d.add(vec)]
    }

    #[inline]
    fn apply_mat2x3(&self, mat: &math::Mat2x3) -> Self::Output {
        [
            mat.pre_mul_vec2(self.a),
            mat.pre_mul_vec2(self.b),
            mat.pre_mul_vec2(self.c),
            mat.pre_mul_vec2(self.d),
        ]
    }

    #[inline]
    fn apply_mat2x3_then_vec2(&self, vec: math::Vec2, mat: &math::Mat2x3) -> Self::Output {
        [
            mat.pre_mul_vec2(self.a).add(vec),
            mat.pre_mul_vec2(self.b).add(vec),
            mat.pre_mul_vec2(self.c).add(vec),
            mat.pre_mul_vec2(self.d).add(vec),
        ]
    }
}

impl ApplyTransformationVertsStep for Quad {
    type Output = [Vec2; 8];

    #[inline]
    fn apply_vec2_step(&self, vec_1: math::Vec2, vec_2: math::Vec2) -> Self::Output {
        [
            self.a.add(vec_1),
            self.b.add(vec_1),
            self.c.add(vec_1),
            self.d.add(vec_1),
            self.a.add(vec_2),
            self.b.add(vec_2),
            self.c.add(vec_2),
            self.d.add(vec_2),
        ]
    }

    #[inline]
    fn apply_mat2x3_step(&self, mat_1: &math::Mat2x3, mat_2: &math::Mat2x3) -> Self::Output {
        [
            mat_1.pre_mul_vec2(self.a),
            mat_1.pre_mul_vec2(self.b),
            mat_1.pre_mul_vec2(self.c),
            mat_1.pre_mul_vec2(self.d),
            mat_2.pre_mul_vec2(self.a),
            mat_2.pre_mul_vec2(self.b),
            mat_2.pre_mul_vec2(self.c),
            mat_2.pre_mul_vec2(self.d),
        ]
    }

    #[inline]
    fn apply_mat2x3_then_vec2_step(
        &self,
        vec_1: math::Vec2,
        vec_2: math::Vec2,
        mat_1: &math::Mat2x3,
        mat_2: &math::Mat2x3,
    ) -> Self::Output {
        [
            mat_1.pre_mul_vec2(self.a).add(vec_1),
            mat_1.pre_mul_vec2(self.b).add(vec_1),
            mat_1.pre_mul_vec2(self.c).add(vec_1),
            mat_1.pre_mul_vec2(self.d).add(vec_1),
            mat_2.pre_mul_vec2(self.a).add(vec_2),
            mat_2.pre_mul_vec2(self.b).add(vec_2),
            mat_2.pre_mul_vec2(self.c).add(vec_2),
            mat_2.pre_mul_vec2(self.d).add(vec_2),
        ]
    }
}

impl ApplyTransformationShape for Quad {
    #[inline]
    fn apply_vec2_checked(&self, vec: math::Vec2) -> Result<Self, error::GeometryError>
    where
        Self: Sized,
    {
        Self::new(self.a.add(vec), self.b.add(vec), self.c.add(vec), self.d.add(vec))
    }

    #[inline]
    fn apply_vec2_unchecked(&self, vec: math::Vec2) -> Self {
        Self::new_unchecked(self.a.add(vec), self.b.add(vec), self.c.add(vec), self.d.add(vec))
    }

    #[inline]
    fn apply_mat2x3_checked(&self, mat: &math::Mat2x3) -> Result<Self, error::GeometryError>
    where
        Self: Sized,
    {
        Self::new(
            mat.pre_mul_vec2(self.a),
            mat.pre_mul_vec2(self.b),
            mat.pre_mul_vec2(self.c),
            mat.pre_mul_vec2(self.d),
        )
    }

    #[inline]
    fn apply_mat2x3_unchecked(&self, mat: &math::Mat2x3) -> Self {
        Self::new_unchecked(
            mat.pre_mul_vec2(self.a),
            mat.pre_mul_vec2(self.b),
            mat.pre_mul_vec2(self.c),
            mat.pre_mul_vec2(self.d),
        )
    }
}

impl ToHitBox for Quad {
    #[inline]
    fn to_hitbox(&self) -> HitBox {
        HitBox::from_verts_array(&[self.a, self.b, self.c, self.d])
    }
}

impl fmt::Display for Quad {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "quadrilateral ({}, {}, {}, {})", self.a, self.b, self.c, self.d)
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

    #[inline]
    pub fn new_unchecked(verts: Vec<math::Vec2>) -> Self {
        Self { verts }
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

impl Validate for Polygon {
    fn validate(&self) -> Result<(), error::GeometryError> {
        let verts_len = self.verts.len();

        if verts_len < 3 {
            return Err(error::GeometryError::TooFewVertices(verts_len));
        } else if verts_len == 3 {
            eprintln!("warning: polygon with 3 vertices, consider Shape::Triangle for efficiency");
        } else if verts_len == 4 {
            eprintln!("warning: polygon with 4 vertices, consider Shape::Quad for efficiency");
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
}

impl ApplyTransformationVerts for Polygon {
    type Output = Vec<Vec2>;

    #[inline]
    fn apply_vec2(&self, vec: math::Vec2) -> Self::Output {
        let mut verts = Vec::with_capacity(self.verts.len() * 2);

        for vert in self.verts.iter() {
            verts.push(vert.add(vec));
        }

        verts
    }

    #[inline]
    fn apply_mat2x3(&self, mat: &math::Mat2x3) -> Self::Output {
        let mut verts = Vec::with_capacity(self.verts.len() * 2);

        for vert in self.verts.iter() {
            verts.push(mat.pre_mul_vec2(*vert));
        }

        verts
    }

    #[inline]
    fn apply_mat2x3_then_vec2(&self, vec: math::Vec2, mat: &math::Mat2x3) -> Self::Output {
        let mut verts = Vec::with_capacity(self.verts.len() * 2);

        for vert in self.verts.iter() {
            verts.push(mat.pre_mul_vec2(*vert).add(vec));
        }

        verts
    }
}

impl ApplyTransformationVertsStep for Polygon {
    type Output = Vec<Vec2>;

    #[inline]
    fn apply_vec2_step(&self, vec_1: math::Vec2, vec_2: math::Vec2) -> Self::Output {
        let mut verts = Vec::with_capacity(self.verts.len() * 2);

        for vert in self.verts.iter() {
            verts.push(vert.add(vec_1));
            verts.push(vert.add(vec_2));
        }

        verts
    }

    #[inline]
    fn apply_mat2x3_step(&self, mat_1: &math::Mat2x3, mat_2: &math::Mat2x3) -> Self::Output {
        let mut verts = Vec::with_capacity(self.verts.len() * 2);

        for vert in self.verts.iter() {
            verts.push(mat_1.pre_mul_vec2(*vert));
            verts.push(mat_2.pre_mul_vec2(*vert));
        }

        verts
    }

    #[inline]
    fn apply_mat2x3_then_vec2_step(
        &self,
        vec_1: math::Vec2,
        vec_2: math::Vec2,
        mat_1: &math::Mat2x3,
        mat_2: &math::Mat2x3,
    ) -> Self::Output {
        let mut verts = Vec::with_capacity(self.verts.len() * 2);

        for vert in self.verts.iter() {
            verts.push(mat_1.pre_mul_vec2(*vert).add(vec_1));
            verts.push(mat_2.pre_mul_vec2(*vert).add(vec_2));
        }

        verts
    }
}

impl ApplyTransformationShape for Polygon {
    #[inline]
    fn apply_vec2_checked(&self, vec: math::Vec2) -> Result<Self, error::GeometryError>
    where
        Self: Sized,
    {
        Self::new(self.verts().into_iter().map(|v| vec.add(*v)).collect())
    }

    #[inline]
    fn apply_vec2_unchecked(&self, vec: math::Vec2) -> Self {
        Self::new_unchecked(self.verts().into_iter().map(|v| vec.add(*v)).collect())
    }

    #[inline]
    fn apply_mat2x3_checked(&self, mat: &math::Mat2x3) -> Result<Self, error::GeometryError>
    where
        Self: Sized,
    {
        Self::new(self.verts.iter().map(|v| mat.pre_mul_vec2(*v)).collect())
    }

    #[inline]
    fn apply_mat2x3_unchecked(&self, mat: &math::Mat2x3) -> Self {
        Self::new_unchecked(self.verts.iter().map(|v| mat.pre_mul_vec2(*v)).collect())
    }
}

impl ToHitBox for Polygon {
    #[inline]
    fn to_hitbox(&self) -> HitBox {
        HitBox::from_verts_slice(&self.verts)
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
    fn to_hitbox(&self) -> HitBox {
        let diameter = self.radius * 2.0;
        HitBox::new(0.0, 0.0, diameter, diameter)
    }
}

impl fmt::Display for Circle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "circle ({:.4})", self.radius)
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
