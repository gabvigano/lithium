use crate::{base, math};

use std::{fmt, mem};

use bincode::{Decode, Encode};
use serde::Deserialize;

#[derive(Debug, Clone)]
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
        Self::new(self.min_x + pos.x, self.min_y + pos.y, self.max_x + pos.x, self.max_y + pos.y)
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
    fn validate(&self) -> Result<(), base::GeometryError>;
    fn normalize(&mut self) -> Result<(), base::GeometryError>;
}

pub trait Centroid {
    fn centroid(&self) -> math::Vec2;
}

pub trait ToHitBox {
    fn to_hitbox(&self) -> HitBox;
}

pub trait SatCompatible {
    fn sides_number(&self) -> usize;
    fn append_sides(&self, sides: &mut Vec<math::Vec2>);
    fn project(&self, axis: math::Vec2) -> (f32, f32);
}

pub trait ApplyTransformationVerts {
    type Output;
    type OutputStep;
    fn apply_vec2(&self, vec: math::Vec2) -> Self::Output;
    fn apply_mat2x3(&self, mat: &math::Mat2x3) -> Self::Output;
    fn apply_mat2x3_then_vec2(&self, vec: math::Vec2, mat: &math::Mat2x3) -> Self::Output;
    fn apply_vec2_step(&self, vec_1: math::Vec2, vec_2: math::Vec2) -> Self::OutputStep;
    fn apply_mat2x3_step(&self, mat_1: &math::Mat2x3, mat_2: &math::Mat2x3) -> Self::OutputStep;
    fn apply_mat2x3_then_vec2_step(
        &self,
        vec_1: math::Vec2,
        vec_2: math::Vec2,
        mat_1: &math::Mat2x3,
        mat_2: &math::Mat2x3,
    ) -> Self::OutputStep;
}

pub trait ApplyTransformationShape {
    fn apply_vec2_checked(&self, vec: math::Vec2) -> Result<Self, base::GeometryError>
    where
        Self: Sized;
    fn apply_vec2_unchecked(&self, vec: math::Vec2) -> Self;
    fn apply_mat2x3_checked(&self, mat: &math::Mat2x3) -> Result<Self, base::GeometryError>
    where
        Self: Sized;
    fn apply_mat2x3_unchecked(&self, mat: &math::Mat2x3) -> Self;
}

/// generates a convex hull from a vector of points using monotone chain algorithm
pub fn convex_hull(mut verts: &mut [math::Vec2]) -> Result<Polygon, base::GeometryError> {
    if verts.len() < 3 {
        return Err(base::GeometryError::TooFewVertices(verts.len()));
    }

    // sort by x and, if x is the same by y
    verts.sort_unstable_by(|a, b| a.x.total_cmp(&b.x).then_with(|| a.y.total_cmp(&b.y)));

    // remove near-duplicates
    verts = math::dedup_by_approx_equal(verts);

    if verts.len() < 3 {
        return Err(base::GeometryError::TooFewVertices(verts.len()));
    }

    fn push_vert(boundary: &mut Vec<math::Vec2>, vert: math::Vec2, init_len: usize) {
        while boundary.len() >= init_len {
            let len = boundary.len();
            if (boundary[len - 2]).signed_area(boundary[len - 1], vert) >= -math::EPS {
                boundary.pop();
            } else {
                break;
            }
        }
        boundary.push(vert);
    }

    let mut hull: Vec<math::Vec2> = Vec::with_capacity(verts.len() * 2);

    // compute top boundary (clockwise from leftmost to rightmost)
    for &v in verts.iter() {
        push_vert(&mut hull, v, 2)
    }

    if !hull.is_empty() {
        hull.pop();
    }

    let init_len = hull.len() + 2;

    // compute bottom boundary (clockwise from rightmost to leftmost)
    for &v in verts.iter().rev() {
        push_vert(&mut hull, v, init_len);
    }

    if !hull.is_empty() {
        hull.pop();
    }

    if hull.len() < 3 {
        return Err(base::GeometryError::TooFewVertices(hull.len()));
    }

    Ok(Polygon::new_unchecked(hull))
}

#[derive(Debug, Clone)]
pub enum SweptShape {
    Unchanged(Shape),
    Changed(Polygon),
}

impl Validate for SweptShape {
    #[inline]
    fn validate(&self) -> Result<(), base::GeometryError> {
        match self {
            SweptShape::Unchanged(shape) => shape.validate(),
            SweptShape::Changed(swept) => swept.validate(),
        }
    }

    #[inline]
    fn normalize(&mut self) -> Result<(), base::GeometryError> {
        match self {
            SweptShape::Unchanged(shape) => shape.normalize(),
            SweptShape::Changed(swept) => swept.normalize(),
        }
    }
}

impl Centroid for SweptShape {
    #[inline]
    fn centroid(&self) -> math::Vec2 {
        match self {
            SweptShape::Unchanged(shape) => shape.centroid(),
            SweptShape::Changed(swept) => swept.centroid(),
        }
    }
}

impl ToHitBox for SweptShape {
    #[inline]
    fn to_hitbox(&self) -> HitBox {
        match self {
            SweptShape::Unchanged(shape) => shape.to_hitbox(),
            SweptShape::Changed(swept) => swept.to_hitbox(),
        }
    }
}

impl SatCompatible for SweptShape {
    #[inline]
    fn sides_number(&self) -> usize {
        match self {
            SweptShape::Unchanged(shape) => match shape {
                Shape::Segment(_) => 2,
                Shape::Triangle(_) => 3,
                Shape::Quad(_) => 4,
                Shape::Polygon(polygon) => polygon.sides_number(),
                Shape::Circle(_) => unimplemented!(),
            },
            SweptShape::Changed(polygon) => polygon.sides_number(),
        }
    }

    #[inline]
    fn append_sides(&self, sides: &mut Vec<math::Vec2>) {
        match self {
            SweptShape::Unchanged(shape) => shape.append_sides(sides),
            SweptShape::Changed(swept) => swept.append_sides(sides),
        }
    }

    #[inline]
    fn project(&self, axis: math::Vec2) -> (f32, f32) {
        match self {
            SweptShape::Unchanged(shape) => shape.project(axis),
            SweptShape::Changed(swept) => swept.project(axis),
        }
    }
}

impl ApplyTransformationShape for SweptShape {
    #[inline]
    fn apply_vec2_checked(&self, vec: math::Vec2) -> Result<Self, base::GeometryError> {
        Ok(match self {
            SweptShape::Unchanged(shape) => SweptShape::Unchanged(shape.apply_vec2_checked(vec)?),
            SweptShape::Changed(swept) => SweptShape::Changed(swept.apply_vec2_checked(vec)?),
        })
    }

    #[inline]
    fn apply_vec2_unchecked(&self, vec: math::Vec2) -> Self {
        match self {
            SweptShape::Unchanged(shape) => SweptShape::Unchanged(shape.apply_vec2_unchecked(vec)),
            SweptShape::Changed(swept) => SweptShape::Changed(swept.apply_vec2_unchecked(vec)),
        }
    }

    #[inline]
    fn apply_mat2x3_checked(&self, mat: &math::Mat2x3) -> Result<Self, base::GeometryError> {
        Ok(match self {
            SweptShape::Unchanged(shape) => SweptShape::Unchanged(shape.apply_mat2x3_checked(mat)?),
            SweptShape::Changed(swept) => SweptShape::Changed(swept.apply_mat2x3_checked(mat)?),
        })
    }

    #[inline]
    fn apply_mat2x3_unchecked(&self, mat: &math::Mat2x3) -> Self {
        match self {
            SweptShape::Unchanged(shape) => SweptShape::Unchanged(shape.apply_mat2x3_unchecked(mat)),
            SweptShape::Changed(swept) => SweptShape::Changed(swept.apply_mat2x3_unchecked(mat)),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Encode, Decode, Deserialize)]
pub enum Shape {
    Segment(Segment),
    Triangle(Triangle),
    Quad(Quad),
    Polygon(Polygon),
    Circle(Circle),
}

impl Validate for Shape {
    #[inline]
    fn validate(&self) -> Result<(), base::GeometryError> {
        match self {
            Shape::Segment(segment) => segment.validate()?,
            Shape::Triangle(triangle) => triangle.validate()?,
            Shape::Quad(quad) => quad.validate()?,
            Shape::Polygon(polygon) => polygon.validate()?,
            Shape::Circle(_) => unimplemented!(),
        };

        Ok(())
    }

    #[inline]
    fn normalize(&mut self) -> Result<(), base::GeometryError> {
        match self {
            Shape::Segment(segment) => segment.normalize()?,
            Shape::Triangle(triangle) => triangle.normalize()?,
            Shape::Quad(quad) => quad.normalize()?,
            Shape::Polygon(polygon) => polygon.normalize()?,
            Shape::Circle(_) => unimplemented!(),
        };

        Ok(())
    }
}

impl Centroid for Shape {
    #[inline]
    fn centroid(&self) -> math::Vec2 {
        match self {
            Shape::Segment(segment) => segment.centroid(),
            Shape::Triangle(triangle) => triangle.centroid(),
            Shape::Quad(quad) => quad.centroid(),
            Shape::Polygon(polygon) => polygon.centroid(),
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

impl SatCompatible for Shape {
    #[inline]
    fn sides_number(&self) -> usize {
        match self {
            Shape::Segment(_) => 2,
            Shape::Triangle(_) => 3,
            Shape::Quad(_) => 4,
            Shape::Polygon(polygon) => polygon.sides_number(),
            Shape::Circle(_) => unimplemented!(),
        }
    }

    #[inline]
    fn append_sides(&self, sides: &mut Vec<math::Vec2>) {
        match self {
            Shape::Segment(segment) => segment.append_sides(sides),
            Shape::Triangle(triangle) => triangle.append_sides(sides),
            Shape::Quad(quad) => quad.append_sides(sides),
            Shape::Polygon(polygon) => polygon.append_sides(sides),
            Shape::Circle(_) => unimplemented!(),
        }
    }

    #[inline]
    fn project(&self, axis: math::Vec2) -> (f32, f32) {
        match self {
            Shape::Segment(segment) => segment.project(axis),
            Shape::Triangle(triangle) => triangle.project(axis),
            Shape::Quad(quad) => quad.project(axis),
            Shape::Polygon(polygon) => polygon.project(axis),
            Shape::Circle(_) => unimplemented!(),
        }
    }
}

impl ApplyTransformationShape for Shape {
    #[inline]
    fn apply_vec2_checked(&self, vec: math::Vec2) -> Result<Self, base::GeometryError> {
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
    fn apply_mat2x3_checked(&self, mat: &math::Mat2x3) -> Result<Self, base::GeometryError> {
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
#[derive(Debug, Clone, PartialEq, Encode, Decode, Deserialize)]
pub struct Segment {
    pub(crate) a: math::Vec2,
    pub(crate) b: math::Vec2,
}

impl Segment {
    #[inline]
    pub fn new(a: math::Vec2, b: math::Vec2) -> Result<Self, base::GeometryError> {
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
    pub fn get_vec2(&self) -> math::Vec2 {
        self.b.sub(self.a)
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
    fn validate(&self) -> Result<(), base::GeometryError> {
        // check duplicates vertices
        if self.a.square_dist(self.b) < math::EPS_SQR {
            return Err(base::GeometryError::DuplicateVertices);
        };

        Ok(())
    }

    #[inline]
    fn normalize(&mut self) -> Result<(), base::GeometryError> {
        Ok(())
    }
}

impl Centroid for Segment {
    #[inline]
    fn centroid(&self) -> math::Vec2 {
        self.a.add(self.b).scale(0.5)
    }
}

impl ToHitBox for Segment {
    #[inline]
    fn to_hitbox(&self) -> HitBox {
        HitBox::from_verts_array(&[self.a, self.b])
    }
}

impl SatCompatible for Segment {
    #[inline]
    fn sides_number(&self) -> usize {
        2
    }

    #[inline]
    fn append_sides(&self, sides: &mut Vec<math::Vec2>) {
        let segment_side = self.b.sub(self.a);
        if segment_side.square_mag() > math::EPS_SQR {
            sides.push(segment_side)
        }
    }

    #[inline]
    fn project(&self, axis: math::Vec2) -> (f32, f32) {
        let (a_proj, b_proj) = (self.a.dot(axis), self.b.dot(axis));
        (a_proj.min(b_proj), a_proj.max(b_proj))
    }
}

impl ApplyTransformationVerts for Segment {
    type Output = [math::Vec2; 2];
    type OutputStep = [math::Vec2; 4];

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

    #[inline]
    fn apply_vec2_step(&self, vec_1: math::Vec2, vec_2: math::Vec2) -> Self::OutputStep {
        [self.a.add(vec_1), self.b.add(vec_1), self.a.add(vec_2), self.b.add(vec_2)]
    }

    #[inline]
    fn apply_mat2x3_step(&self, mat_1: &math::Mat2x3, mat_2: &math::Mat2x3) -> Self::OutputStep {
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
    ) -> Self::OutputStep {
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
    fn apply_vec2_checked(&self, vec: math::Vec2) -> Result<Self, base::GeometryError>
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
    fn apply_mat2x3_checked(&self, mat: &math::Mat2x3) -> Result<Self, base::GeometryError>
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

impl fmt::Display for Segment {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "segment ({}, {})", self.a, self.b)
    }
}

/// notice that a, b and c are local positions, you may need to manually integrate them with a position
#[derive(Debug, Clone, PartialEq, Encode, Decode, Deserialize)]
pub struct Triangle {
    pub(crate) a: math::Vec2,
    pub(crate) b: math::Vec2,
    pub(crate) c: math::Vec2,
}

impl Triangle {
    #[inline]
    pub fn new(a: math::Vec2, b: math::Vec2, c: math::Vec2) -> Result<Self, base::GeometryError> {
        let triangle = Self { a, b, c };

        triangle.validate()?;

        Ok(triangle)
    }

    #[inline]
    pub fn new_unchecked(a: math::Vec2, b: math::Vec2, c: math::Vec2) -> Self {
        Self { a, b, c }
    }

    #[inline]
    pub fn from_hull(mut verts: &mut [math::Vec2]) -> Result<Triangle, base::GeometryError> {
        verts = math::dedup_by_approx_equal(verts);

        if verts.len() != 3 {
            return Err(base::GeometryError::TooFewVertices(verts.len()));
        }

        let mut triangle = Triangle::new_unchecked(verts[0], verts[1], verts[2]);
        triangle.normalize()?;

        Ok(triangle)
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
    fn validate(&self) -> Result<(), base::GeometryError> {
        // check duplicates vertices
        if self.a.square_dist(self.b) < math::EPS_SQR
            || self.a.square_dist(self.c) < math::EPS_SQR
            || self.b.square_dist(self.c) < math::EPS_SQR
        {
            return Err(base::GeometryError::DuplicateVertices);
        };

        // check non-collinear + clockwise winding
        if self.a.signed_area(self.b, self.c) >= -math::EPS {
            return Err(base::GeometryError::NotConvex);
        }

        Ok(())
    }

    #[inline]
    fn normalize(&mut self) -> Result<(), base::GeometryError> {
        if self.a.signed_area(self.b, self.c) > 0.0 {
            mem::swap(&mut self.b, &mut self.c);
        }

        Ok(())
    }
}

impl Centroid for Triangle {
    #[inline]
    fn centroid(&self) -> math::Vec2 {
        self.a.add(self.b.add(self.c)).scale(1.0 / 3.0)
    }
}

impl ToHitBox for Triangle {
    #[inline]
    fn to_hitbox(&self) -> HitBox {
        HitBox::from_verts_array(&[self.a, self.b, self.c])
    }
}

impl SatCompatible for Triangle {
    #[inline]
    fn sides_number(&self) -> usize {
        3
    }

    #[inline]
    fn append_sides(&self, sides: &mut Vec<math::Vec2>) {
        let triangle_sides = [self.b.sub(self.a), self.c.sub(self.b), self.a.sub(self.c)];
        for triangle_side in triangle_sides {
            if triangle_side.square_mag() > math::EPS_SQR {
                sides.push(triangle_side);
            }
        }
    }

    #[inline]
    fn project(&self, axis: math::Vec2) -> (f32, f32) {
        let (a_proj, b_proj, c_proj) = (self.a.dot(axis), self.b.dot(axis), self.c.dot(axis));
        (a_proj.min(b_proj).min(c_proj), a_proj.max(b_proj).max(c_proj))
    }
}

impl ApplyTransformationVerts for Triangle {
    type Output = [math::Vec2; 3];
    type OutputStep = [math::Vec2; 6];

    #[inline]
    fn apply_vec2(&self, vec: math::Vec2) -> Self::Output {
        [self.a.add(vec), self.b.add(vec), self.c.add(vec)]
    }

    #[inline]
    fn apply_mat2x3(&self, mat: &math::Mat2x3) -> Self::Output {
        [mat.pre_mul_vec2(self.a), mat.pre_mul_vec2(self.b), mat.pre_mul_vec2(self.c)]
    }

    #[inline]
    fn apply_mat2x3_then_vec2(&self, vec: math::Vec2, mat: &math::Mat2x3) -> Self::Output {
        [
            mat.pre_mul_vec2(self.a).add(vec),
            mat.pre_mul_vec2(self.b).add(vec),
            mat.pre_mul_vec2(self.c).add(vec),
        ]
    }

    #[inline]
    fn apply_vec2_step(&self, vec_1: math::Vec2, vec_2: math::Vec2) -> Self::OutputStep {
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
    fn apply_mat2x3_step(&self, mat_1: &math::Mat2x3, mat_2: &math::Mat2x3) -> Self::OutputStep {
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
    ) -> Self::OutputStep {
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
    fn apply_vec2_checked(&self, vec: math::Vec2) -> Result<Self, base::GeometryError>
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
    fn apply_mat2x3_checked(&self, mat: &math::Mat2x3) -> Result<Self, base::GeometryError>
    where
        Self: Sized,
    {
        Self::new(mat.pre_mul_vec2(self.a), mat.pre_mul_vec2(self.b), mat.pre_mul_vec2(self.c))
    }

    #[inline]
    fn apply_mat2x3_unchecked(&self, mat: &math::Mat2x3) -> Self {
        Self::new_unchecked(mat.pre_mul_vec2(self.a), mat.pre_mul_vec2(self.b), mat.pre_mul_vec2(self.c))
    }
}

impl fmt::Display for Triangle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "triangle ({}, {}, {})", self.a, self.b, self.c)
    }
}

#[derive(Debug, Clone, PartialEq, Encode, Decode, Deserialize)]
pub struct Rect {
    pub(crate) width: f32,
    pub(crate) height: f32,
}

impl Rect {
    #[inline]
    pub fn new(width: f32, height: f32) -> Result<Self, base::MathError> {
        let rect = Self { width, height };

        rect.validate()?;

        Ok(rect)
    }

    #[inline]
    pub fn validate(&self) -> Result<(), base::MathError> {
        if self.width <= 0.0 {
            return Err(base::MathError::NonPositive("width"));
        }

        if self.height <= 0.0 {
            return Err(base::MathError::NonPositive("height"));
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
#[derive(Debug, Clone, PartialEq, Encode, Decode, Deserialize)]
pub struct Quad {
    pub(crate) a: math::Vec2,
    pub(crate) b: math::Vec2,
    pub(crate) c: math::Vec2,
    pub(crate) d: math::Vec2,
}

impl Quad {
    #[inline]
    pub fn new(a: math::Vec2, b: math::Vec2, c: math::Vec2, d: math::Vec2) -> Result<Self, base::GeometryError> {
        let quad = Self { a, b, c, d };

        quad.validate()?;

        Ok(quad)
    }

    #[inline]
    pub fn new_unchecked(a: math::Vec2, b: math::Vec2, c: math::Vec2, d: math::Vec2) -> Self {
        Self { a, b, c, d }
    }

    #[inline]
    pub fn from_hull(verts: &mut [math::Vec2]) -> Result<Quad, base::GeometryError> {
        let hull = convex_hull(verts)?;

        let verts_len = hull.verts.len();
        if verts_len != 4 {
            return Err(base::GeometryError::NormalizationError);
        }

        Ok(Quad::new_unchecked(hull.verts[0], hull.verts[1], hull.verts[2], hull.verts[3]))
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
    fn validate(&self) -> Result<(), base::GeometryError> {
        // check duplicates vertices
        if self.a.square_dist(self.b) < math::EPS_SQR
            || self.a.square_dist(self.c) < math::EPS_SQR
            || self.a.square_dist(self.d) < math::EPS_SQR
            || self.b.square_dist(self.c) < math::EPS_SQR
            || self.b.square_dist(self.d) < math::EPS_SQR
            || self.c.square_dist(self.d) < math::EPS_SQR
        {
            return Err(base::GeometryError::DuplicateVertices);
        };

        // check if the quadrilateral is convex
        if self.a.signed_area(self.b, self.c) >= -math::EPS
            || self.b.signed_area(self.c, self.d) >= -math::EPS
            || self.c.signed_area(self.d, self.a) >= -math::EPS
            || self.d.signed_area(self.a, self.b) >= -math::EPS
        {
            return Err(base::GeometryError::NotConvex);
        }

        Ok(())
    }

    #[inline]
    fn normalize(&mut self) -> Result<(), base::GeometryError> {
        let mut verts = [self.a, self.b, self.c, self.d];

        *self = Self::from_hull(&mut verts)?;

        Ok(())
    }
}

impl Centroid for Quad {
    #[inline]
    fn centroid(&self) -> math::Vec2 {
        self.a.add(self.b.add(self.c.add(self.d))).scale(0.25)
    }
}

impl ToHitBox for Quad {
    #[inline]
    fn to_hitbox(&self) -> HitBox {
        HitBox::from_verts_array(&[self.a, self.b, self.c, self.d])
    }
}

impl SatCompatible for Quad {
    #[inline]
    fn sides_number(&self) -> usize {
        4
    }

    #[inline]
    fn append_sides(&self, sides: &mut Vec<math::Vec2>) {
        let quad_sides = [self.b.sub(self.a), self.c.sub(self.b), self.d.sub(self.c), self.a.sub(self.d)];
        for quad_side in quad_sides {
            if quad_side.square_mag() > math::EPS_SQR {
                sides.push(quad_side);
            }
        }
    }

    #[inline]
    fn project(&self, axis: math::Vec2) -> (f32, f32) {
        let (a_proj, b_proj, c_proj, d_proj) = (self.a.dot(axis), self.b.dot(axis), self.c.dot(axis), self.d.dot(axis));
        (
            a_proj.min(b_proj).min(c_proj).min(d_proj),
            a_proj.max(b_proj).max(c_proj).max(d_proj),
        )
    }
}

impl ApplyTransformationVerts for Quad {
    type Output = [math::Vec2; 4];
    type OutputStep = [math::Vec2; 8];

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

    #[inline]
    fn apply_vec2_step(&self, vec_1: math::Vec2, vec_2: math::Vec2) -> Self::OutputStep {
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
    fn apply_mat2x3_step(&self, mat_1: &math::Mat2x3, mat_2: &math::Mat2x3) -> Self::OutputStep {
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
    ) -> Self::OutputStep {
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
    fn apply_vec2_checked(&self, vec: math::Vec2) -> Result<Self, base::GeometryError>
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
    fn apply_mat2x3_checked(&self, mat: &math::Mat2x3) -> Result<Self, base::GeometryError>
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

impl fmt::Display for Quad {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "quadrilateral ({}, {}, {}, {})", self.a, self.b, self.c, self.d)
    }
}

/// polygons must be convex, vertices must be stored counterclockwise, and there must be no collinear edges
/// notice that vertices are local positions, you may need to manually integrate them with a position
#[derive(Debug, Clone, PartialEq, Encode, Decode, Deserialize)]
pub struct Polygon {
    pub(crate) verts: Vec<math::Vec2>,
}

impl Polygon {
    #[inline]
    pub fn new(verts: Vec<math::Vec2>) -> Result<Self, base::GeometryError> {
        let polygon = Self { verts };

        polygon.validate()?;

        Ok(polygon)
    }

    #[inline]
    pub fn new_unchecked(verts: Vec<math::Vec2>) -> Self {
        Self { verts }
    }

    #[inline]
    pub fn from_hull(verts: &mut [math::Vec2]) -> Result<Polygon, base::GeometryError> {
        convex_hull(verts)
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
    fn validate(&self) -> Result<(), base::GeometryError> {
        let verts_len = self.verts.len();

        if verts_len < 3 {
            return Err(base::GeometryError::TooFewVertices(verts_len));
        } else if verts_len == 3 {
            eprintln!("warning: polygon with 3 vertices, consider Shape::Triangle for efficiency");
        } else if verts_len == 4 {
            eprintln!("warning: polygon with 4 vertices, consider Shape::Quad for efficiency");
        }

        // check duplicates vertices
        for i in 0..verts_len {
            for j in (i + 1)..verts_len {
                if self.verts[i].square_dist(self.verts[j]) < math::EPS_SQR {
                    return Err(base::GeometryError::DuplicateVertices);
                }
            }
        }

        // check if the polygon is convex
        for i in 0..verts_len {
            let i1 = (i + 1) % verts_len; // use modulo indexing to restart when the end is reached
            let i2 = (i + 2) % verts_len;

            let area = self.verts[i].signed_area(self.verts[i1], self.verts[i2]);

            if area >= -math::EPS {
                return Err(base::GeometryError::NotConvex);
            }
        }

        Ok(())
    }

    #[inline]
    fn normalize(&mut self) -> Result<(), base::GeometryError> {
        let mut verts = self.verts.clone();

        let normalized = Self::from_hull(&mut verts)?;

        if self.verts.len() != normalized.verts.len() {
            return Err(base::GeometryError::NormalizationError);
        }

        *self = normalized;

        Ok(())
    }
}

impl Centroid for Polygon {
    #[inline]
    fn centroid(&self) -> math::Vec2 {
        let mut sum = math::Vec2::new(0.0, 0.0);
        for vert in &self.verts {
            sum.add_mut(*vert);
        }
        sum.scale(1.0 / self.verts.len() as f32)
    }
}

impl ToHitBox for Polygon {
    #[inline]
    fn to_hitbox(&self) -> HitBox {
        HitBox::from_verts_slice(&self.verts)
    }
}

impl SatCompatible for Polygon {
    #[inline]
    fn sides_number(&self) -> usize {
        self.verts().len()
    }

    #[inline]
    fn append_sides(&self, sides: &mut Vec<math::Vec2>) {
        let verts = &self.verts;
        let mut prev = *verts.last().unwrap();
        for &curr in verts {
            let side = curr.sub(prev);
            if side.square_mag() > math::EPS_SQR {
                sides.push(side);
            }
            prev = curr;
        }
    }

    #[inline]
    fn project(&self, axis: math::Vec2) -> (f32, f32) {
        let mut min = self.verts[0].dot(axis);
        let mut max = min;

        for vert in self.verts.iter().skip(1) {
            let proj = vert.dot(axis);
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

impl ApplyTransformationVerts for Polygon {
    type Output = Vec<math::Vec2>;
    type OutputStep = Vec<math::Vec2>;

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

    #[inline]
    fn apply_vec2_step(&self, vec_1: math::Vec2, vec_2: math::Vec2) -> Self::OutputStep {
        let mut verts = Vec::with_capacity(self.verts.len() * 2);

        for vert in self.verts.iter() {
            verts.push(vert.add(vec_1));
            verts.push(vert.add(vec_2));
        }

        verts
    }

    #[inline]
    fn apply_mat2x3_step(&self, mat_1: &math::Mat2x3, mat_2: &math::Mat2x3) -> Self::OutputStep {
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
    ) -> Self::OutputStep {
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
    fn apply_vec2_checked(&self, vec: math::Vec2) -> Result<Self, base::GeometryError>
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
    fn apply_mat2x3_checked(&self, mat: &math::Mat2x3) -> Result<Self, base::GeometryError>
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

#[derive(Debug, Clone, PartialEq, Encode, Decode, Deserialize)]
pub struct Circle {
    pub(crate) radius: f32,
}

impl Circle {
    #[inline]
    pub fn new(radius: f32) -> Result<Self, base::MathError> {
        let circle = Self { radius };

        circle.validate()?;

        Ok(circle)
    }

    #[inline]
    pub fn validate(&self) -> Result<(), base::MathError> {
        if self.radius <= 0.0 {
            return Err(base::MathError::NonPositive("radius"));
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
