use serde::Deserialize;

// PHYSICS

#[derive(Debug, Copy, Clone, Deserialize)]
pub struct Pos {
    pub x: f32,
    pub y: f32,
}

#[derive(Debug, Copy, Clone, Deserialize)]
pub struct Vel {
    pub x: f32,
    pub y: f32,
}

#[derive(Debug, Copy, Clone, Deserialize)]
pub struct Acc {
    pub x: f32,
    pub y: f32,
}

#[derive(Debug)]
pub struct Force {
    pub mag: f32,
    pub dir: Dir,
}

pub type Rest = bool;

#[derive(Debug, Deserialize)]
pub struct Mass(pub f32);

#[derive(Debug, Deserialize)]
pub struct Elast(pub f32);

#[derive(Debug)]
pub enum Dir {
    Angle(Angle),
    Axis(Axis),
}

#[derive(Debug)]
pub struct Angle {
    pub radians: f32,
}

impl Angle {
    pub fn norm(mut self) -> Self {
        self.radians = self.radians.rem_euclid(std::f32::consts::PI * 2.0);
        self
    }
}

#[derive(Debug)]
pub enum Axis {
    Horizontal,
    Vertical,
}

// DISPLAY

#[derive(Debug, Copy, Clone, Deserialize)]
pub enum Shape {
    Rect(Rect),
    Circle(Circle),
}

#[derive(Debug, Copy, Clone, Deserialize)]
pub struct Rect {
    pub width: f32,
    pub height: f32,
}

#[derive(Debug, Copy, Clone, Deserialize)]
pub struct Circle {
    pub radius: f32,
}

#[derive(Debug, Deserialize)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

#[derive(Debug, Deserialize)]
pub struct Layer(pub usize);

pub type Show = bool;

// OBJECTS

#[derive(Deserialize)]
pub struct Static {
    pub pos: Pos,
    pub elast: Elast,
    pub shape: Shape,
    pub color: Color,
    pub layer: Layer,
    pub show: Show,
}

#[derive(Deserialize)]
pub struct Dynamic {
    pub pos: Pos,
    pub vel: Vel,
    pub acc: Acc,
    pub rest: Rest,
    pub mass: Mass,
    pub elast: Elast,
    pub shape: Shape,
    pub color: Color,
    pub layer: Layer,
    pub show: Show,
}
