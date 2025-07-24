// PHYSICS

#[derive(Debug, Clone)]
pub struct Pos {
    pub x: f32,
    pub y: f32,
}

#[derive(Debug)]
pub struct Vel {
    pub x: f32,
    pub y: f32,
}

#[derive(Debug)]
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

#[derive(Debug)]
pub struct Mass(pub f32);

#[derive(Debug)]
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

#[derive(Debug)]
pub enum Axis {
    Horizontal,
    Vertical,
}

// DISPLAY

#[derive(Debug, Clone)]
pub enum Shape {
    Rect(Rect),
    Circle(Circle),
}

#[derive(Debug, Clone)]
pub struct Rect {
    pub width: f32,
    pub height: f32,
}

#[derive(Debug, Clone)]
pub struct Circle {
    pub radius: f32,
}

#[derive(Debug)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

#[derive(Debug)]
pub struct Layer(pub usize);

pub type Show = bool;
