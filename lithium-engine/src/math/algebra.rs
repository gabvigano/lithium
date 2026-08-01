use std::fmt;

use bincode::{Decode, Encode};
use serde::Deserialize;

pub const EPS: f32 = 1e-6;
pub const EPS_SQR: f32 = EPS * EPS;

#[inline(always)]
pub fn pow2(x: f32) -> f32 {
    x * x
}

#[inline]
pub fn clamp(value: &mut f32, min: f32, max: f32) {
    *value = value.clamp(min, max)
}

#[inline]
pub fn clamp_max(value: &mut f32, max: f32) {
    *value = value.min(max);
}

#[inline]
pub fn clamp_min(value: &mut f32, min: f32) {
    *value = value.max(min);
}

/// checks if 2 segments are intersecting
#[inline]
pub fn check_segments_intersection(a1: Vec2, a2: Vec2, b1: Vec2, b2: Vec2) -> bool {
    let d1 = a1.signed_area(a2, b1);
    let d2 = a1.signed_area(a2, b2);
    let d3 = b1.signed_area(b2, a1);
    let d4 = b1.signed_area(b2, a2);

    d1 * d2 < -EPS && d3 * d4 < -EPS
}

#[derive(Debug, Clone, Copy, PartialEq, Encode, Decode, Deserialize)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}

impl Vec2 {
    pub const ZERO: Self = Self { x: 0.0, y: 0.0 };
    pub const ONE: Self = Self { x: 0.0, y: 0.0 };

    #[inline]
    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    #[inline]
    pub fn equal(self, vec2: Self) -> bool {
        self.x == vec2.x && self.y == vec2.y
    }

    #[inline]
    pub fn approx_equal(self, vec2: Self) -> bool {
        (self.x - vec2.x).abs() < EPS && (self.y - vec2.y).abs() < EPS
    }

    #[inline]
    pub fn approx_equal_zero(self) -> bool {
        self.x.abs() < EPS && self.y.abs() < EPS
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
    pub fn rev(self) -> Self {
        Self::new(-self.x, -self.y)
    }

    #[inline]
    pub fn rev_mut(&mut self) {
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
    pub fn cross_scalar(self, scalar: f32) -> Self {
        Self::new(-scalar * self.y, scalar * self.x)
    }

    #[inline]
    pub fn signed_area(self, q: Self, r: Self) -> f32 {
        (q.sub(self)).cross(r.sub(self))
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

    #[inline]
    pub fn midpoint(self, point: Self) -> Self {
        self.add(point).scale(0.5)
    }

    /// barycentric coordinate method
    #[inline]
    pub fn is_inside_3_vec2(self, a: Self, b: Self, c: Self) -> bool {
        let v0 = c.sub(a);
        let v1 = b.sub(a);
        let v2 = self.sub(a);

        let denominator = v0.cross(v1);

        if denominator.abs() < EPS {
            return false;
        }

        if denominator > 0.0 {
            let u = v2.cross(v1);
            let v = v0.cross(v2);
            (u > -EPS) && (v > -EPS) && (u + v <= denominator + EPS)
        } else {
            let u = v1.cross(v2);
            let v = v2.cross(v0);
            (u > -EPS) && (v > -EPS) && (u + v <= -denominator - EPS)
        }
    }

    #[inline]
    pub fn is_inside_3_ccw_vec2(self, a: Self, b: Self, c: Self) -> bool {
        a.signed_area(b, self) < EPS && b.signed_area(c, self) < EPS && c.signed_area(a, self) < EPS
    }
}

impl fmt::Display for Vec2 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{:.4} , {:.4}]", self.x, self.y)
    }
}

/// dedup for slices that uses Vec2.approx_equal()
pub fn dedup_by_approx_equal(slice: &mut [Vec2]) -> &mut [Vec2] {
    let len = slice.len();
    if len <= 1 {
        return slice;
    }

    let mut read: usize = 1;
    let mut write: usize = 1;

    while read < len {
        if !slice[read].approx_equal(slice[write - 1]) {
            if read != write {
                slice[write] = slice[read];
            }
            write += 1;
        }
        read += 1;
    }

    &mut slice[..write]
}

/// column-major 2x3 matrix; the matrix is like this
/// [x.0 y.0 z.0]
/// [x.1 y.1 z.1]
///
/// conceptually represents a 3x3 matrix with an implicit third row [0 0 1],
/// so some operations (like mat2x3 * mat2x3) that would not even be possible
/// are done by hardcoding that third row

#[derive(Debug, Clone, PartialEq, Encode, Decode, Deserialize)]
pub struct Mat2x3 {
    pub x: (f32, f32),
    pub y: (f32, f32),
    pub z: (f32, f32),
}

impl Mat2x3 {
    pub const ZERO: Self = Self {
        x: (0.0, 0.0),
        y: (0.0, 0.0),
        z: (0.0, 0.0),
    };

    pub const ONE: Self = Self {
        x: (1.0, 1.0),
        y: (1.0, 1.0),
        z: (1.0, 1.0),
    };

    pub const IDENTITY: Self = Self {
        x: (1.0, 0.0),
        y: (0.0, 1.0),
        z: (0.0, 0.0),
    };

    #[inline]
    pub const fn new(x: (f32, f32), y: (f32, f32), z: (f32, f32)) -> Self {
        Self { x: x, y: y, z: z }
    }

    #[inline]
    pub fn from_rot_and_pivot(rot: Radians, pivot: Vec2) -> Self {
        let (cos, sin) = (rot.0.cos(), rot.0.sin());

        Self::new(
            (cos, sin),
            (-sin, cos),
            ((1.0 - cos) * pivot.x + sin * pivot.y, (1.0 - cos) * pivot.y - sin * pivot.x),
        )
    }

    #[inline]
    pub fn equal(&self, mat2: &Self) -> bool {
        self.x.0 == mat2.x.0
            && self.x.1 == mat2.x.1
            && self.y.0 == mat2.y.0
            && self.y.1 == mat2.y.1
            && self.z.0 == mat2.z.0
            && self.z.1 == mat2.z.1
    }

    #[inline]
    pub fn approx_equal(&self, mat2: &Self) -> bool {
        (self.x.0 - mat2.x.0).abs() < EPS
            && (self.x.1 - mat2.x.1).abs() < EPS
            && (self.y.0 - mat2.y.0).abs() < EPS
            && (self.y.1 - mat2.y.1).abs() < EPS
            && (self.z.0 - mat2.z.0).abs() < EPS
            && (self.z.1 - mat2.z.1).abs() < EPS
    }

    #[inline]
    pub fn approx_equal_identity(&self) -> bool {
        (self.x.0 - 1.0).abs() < EPS
            && self.x.1.abs() < EPS
            && self.y.0.abs() < EPS
            && (self.y.1 - 1.0).abs() < EPS
            && self.z.0.abs() < EPS
            && self.z.1.abs() < EPS
    }

    #[inline]
    pub fn pre_mul(&self, mat2: &Self) -> Self {
        Self::new(
            (
                self.x.0.mul_add(mat2.x.0, self.x.1 * mat2.y.0),
                self.x.0.mul_add(mat2.x.1, self.x.1 * mat2.y.1),
            ),
            (
                self.y.0.mul_add(mat2.x.0, self.y.1 * mat2.y.0),
                self.y.0.mul_add(mat2.x.1, self.y.1 * mat2.y.1),
            ),
            (
                self.z.0.mul_add(mat2.x.0, self.z.1.mul_add(mat2.y.0, mat2.z.0)),
                self.z.0.mul_add(mat2.x.1, self.z.1.mul_add(mat2.y.1, mat2.z.1)),
            ),
        )
    }

    #[inline]
    pub fn pre_mul_mut(&mut self, mat2: &Self) {
        *self = self.pre_mul(mat2);
    }

    #[inline]
    pub fn pre_mul_vec2(&self, vec: Vec2) -> Vec2 {
        Vec2::new(
            vec.x * self.x.0 + vec.y * self.y.0 + self.z.0,
            vec.x * self.x.1 + vec.y * self.y.1 + self.z.1,
        )
    }
}

impl fmt::Display for Mat2x3 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "[[{:.4} , {:.4} , {:.4}] , [{:.4} , {:.4} , {:.4}]]",
            self.x.0, self.y.0, self.z.0, self.x.1, self.y.1, self.z.1
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Axis {
    X,
    Y,
}

#[derive(Debug, Clone, Copy, PartialEq, Deserialize)]
pub struct Radians(pub f32);

impl Radians {
    #[inline]
    pub const fn new(radians: f32) -> Self {
        Self(radians)
    }

    #[inline]
    pub fn from_degrees(degrees: f32) -> Self {
        Self::new(degrees.to_radians()).norm()
    }

    #[inline]
    pub fn norm(mut self) -> Self {
        self.0 = self.0.rem_euclid(std::f32::consts::TAU);
        self
    }

    #[inline]
    pub fn norm_mut(&mut self) {
        self.0 = self.0.rem_euclid(std::f32::consts::TAU);
    }
}

impl fmt::Display for Radians {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:.4} rad", self.0)
    }
}
