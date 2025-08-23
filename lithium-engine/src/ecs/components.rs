use serde::Deserialize;

use crate::ecs::systems::physics::{EPS, EPS_SQR, pow2};

#[derive(Copy, Clone, Deserialize, Debug)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}

impl Vec2 {
    #[inline(always)]
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

pub type Rest = bool;

#[derive(Copy, Clone, Deserialize, Debug)]
pub struct Mass(pub f32);

impl Mass {
    #[inline(always)]
    pub fn new(mass: f32) -> Self {
        Self(mass)
    }
}

#[derive(Copy, Clone, Deserialize, Debug)]
pub struct Elast(pub f32);

impl Elast {
    #[inline(always)]
    pub fn new(elast: f32) -> Self {
        Self(elast)
    }
}

#[derive(Copy, Clone, Debug)]
pub struct Force {
    pub mag: f32,
    pub dir: Dir,
}

impl Force {
    #[inline(always)]
    pub fn new(mag: f32, dir: Dir) -> Self {
        Self { mag, dir }
    }
}

#[derive(Copy, Clone, Debug)]
pub enum Dir {
    Axis(Axis),
    Angle(Angle),
}

#[derive(Copy, Clone, Debug)]
pub enum Axis {
    Horizontal,
    Vertical,
}

#[derive(Copy, Clone, Debug)]
pub struct Angle {
    pub radians: f32,
}

impl Angle {
    #[inline(always)]
    pub fn new(radians: f32) -> Self {
        Self { radians }
    }

    #[inline(always)]
    pub fn norm(mut self) -> Self {
        self.radians = self.radians.rem_euclid(std::f32::consts::PI * 2.0);
        self
    }
}

#[derive(Deserialize, Debug)]
pub enum Shape {
    Rect(Rect),
    Circle(Circle),
}

#[derive(Copy, Clone, Deserialize, Debug)]
pub struct Rect {
    pub width: f32,
    pub height: f32,
}

impl Rect {
    #[inline(always)]
    pub fn new(width: f32, height: f32) -> Self {
        Self { width, height }
    }
}

#[derive(Copy, Clone, Deserialize, Debug)]
pub struct Circle {
    pub radius: f32,
}

impl Circle {
    #[inline(always)]
    pub fn new(radius: f32) -> Self {
        Self { radius }
    }
}

#[derive(Copy, Clone, Deserialize, Debug)]
pub struct Line {
    pub a: Vec2,
    pub b: Vec2,
}

impl Line {
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

/// chaches data about a line to speed up calculations
#[derive(Debug)]
pub struct LineCache {
    pub a: Vec2,
    pub b: Vec2,
    pub delta_x: f32,
    pub delta_y: f32,
    pub m: Option<f32>,
    pub q: Option<f32>,
    pub c: f32,
    pub pow_sum: f32,
}

impl LineCache {
    #[inline]
    pub fn new(a: Vec2, b: Vec2) -> Self {
        let delta_x = b.x - a.x;
        let delta_y = b.y - a.y;

        let (m, q) = if delta_x.abs() <= EPS {
            (None, None)
        } else {
            let m = delta_y / delta_x;
            (Some(m), Some(a.y - m * a.x))
        };

        Self {
            a: a,
            b: b,
            delta_x: delta_x,
            delta_y: delta_y,
            m: m,
            q: q,
            c: (b.x * a.y - b.y * a.x),
            pow_sum: (pow2(delta_x) + pow2(delta_y)),
        }
    }

    #[inline]
    pub fn eval_x(&self, x: f32) -> Option<f32> {
        if x < self.a.x.min(self.b.x) - EPS || x > self.a.x.max(self.b.x) + EPS {
            // out of range
            return None;
        };

        match (self.m, self.q) {
            (Some(m), Some(q)) => Some(x.mul_add(m, q)),
            _ => None,
        }
    }

    #[inline]
    pub fn eval_y(&self, y: f32) -> Option<f32> {
        if y < self.a.y.min(self.b.y) - EPS || y > self.a.y.max(self.b.y) + EPS {
            // out of range
            return None;
        };

        match (self.m, self.q) {
            (Some(m), Some(q)) if m.abs() > EPS => Some((y - q) / m),
            _ => None,
        }
    }

    #[inline]
    pub fn point_square_dist(&self, point: Vec2) -> f32 {
        // vector from A to the point
        let ap = Vec2::new(point.x - self.a.x, point.y - self.a.y);

        // squared length of the segment
        if self.pow_sum <= EPS_SQR {
            // segment is a single point at A
            return ap.square_len();
        }

        // projection factor of AP onto AB, normalized by |AB|^2
        let t = ap.dot(Vec2::new(self.delta_x, self.delta_y)) / self.pow_sum;

        if t < 0.0 {
            // projection is before A → closest point is A
            ap.square_len()
        } else if t > 1.0 {
            // projection is past B → closest point is B
            Vec2::new(point.x - self.b.x, point.y - self.b.y).square_len()
        } else {
            // projection falls on the segment → use perpendicular distance
            pow2(self.delta_y * point.x - self.delta_x * point.y + self.c) / self.pow_sum
        }
    }

    #[inline]
    pub fn rect_square_dist(&self, pos: Vec2, rect: Rect) -> (f32, f32, f32, f32) {
        let tl = self.point_square_dist(pos);
        let tr = self.point_square_dist(Vec2::new(pos.x + rect.width, pos.y));
        let bl = self.point_square_dist(Vec2::new(pos.x, pos.y + rect.height));
        let br = self.point_square_dist(Vec2::new(pos.x + rect.width, pos.y + rect.height));

        (tl, tr, bl, br)
    }

    #[inline]
    pub fn circle_square_dist(&self, pos: Vec2, circle: Circle) -> f32 {
        self.point_square_dist(Vec2::new(pos.x + circle.radius, pos.y + circle.radius))
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
    #[inline(always)]
    pub fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }
}

#[derive(Copy, Clone, Deserialize, Debug)]
pub struct Layer(pub usize);

impl Layer {
    #[inline(always)]
    pub fn new(layer: usize) -> Self {
        Self(layer)
    }
}

pub type Show = bool;

// OBJECTS

#[derive(Deserialize)]
pub struct Static {
    pub pos: Vec2,
    pub elast: Elast,
    pub shape: Shape,
    pub color: Color,
    pub layer: Layer,
    pub show: Show,
}

#[derive(Deserialize)]
pub struct Dynamic {
    pub pos: Vec2,
    pub vel: Vec2,
    pub acc: Vec2,
    pub rest: Rest,
    pub mass: Mass,
    pub elast: Elast,
    pub shape: Shape,
    pub color: Color,
    pub layer: Layer,
    pub show: Show,
}
