use serde::Deserialize;
use std::fmt;

pub const EPS: f32 = 1e-6;
pub const EPS_SQR: f32 = EPS * EPS;

#[inline(always)]
pub fn pow2(x: f32) -> f32 {
    x * x
}

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
        write!(f, "[{:.4} , {:.4}]", self.x, self.y)
    }
}

#[derive(Copy, Clone, Debug)]
pub enum Axis {
    X,
    Y,
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

    #[inline]
    pub fn to_vec2(self) -> Vec2 {
        Vec2::new(self.radians.cos(), self.radians.sin())
    }
}

impl fmt::Display for Angle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:.4} rad", self.radians)
    }
}
