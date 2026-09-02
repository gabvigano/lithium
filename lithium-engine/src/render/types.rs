use crate::{math, render};

use std::fmt;

#[cfg(feature = "network")]
use bincode::{Decode, Encode};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PixelPos {
    pub x: u32,
    pub y: u32,
}

#[derive(Debug, Clone)]
pub struct BoundingBox {
    pub(crate) min_x: usize,
    pub(crate) min_y: usize,
    pub(crate) max_x: usize,
    pub(crate) max_y: usize,
}

impl BoundingBox {
    #[inline]
    pub const fn new(min_x: usize, min_y: usize, max_x: usize, max_y: usize) -> Self {
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

        Self::new(
            min_x.floor() as usize,
            min_y.floor() as usize,
            max_x.ceil() as usize,
            max_y.ceil() as usize,
        )
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

        Self::new(
            min_x.floor() as usize,
            min_y.floor() as usize,
            max_x.ceil() as usize,
            max_y.ceil() as usize,
        )
    }

    #[inline]
    pub fn min_x(&self) -> usize {
        self.min_x
    }

    #[inline]
    pub fn min_y(&self) -> usize {
        self.min_y
    }

    #[inline]
    pub fn max_x(&self) -> usize {
        self.max_x
    }

    #[inline]
    pub fn max_y(&self) -> usize {
        self.max_y
    }

    #[inline]
    pub fn set_min_x(&mut self, new_min_x: usize) {
        self.min_x = new_min_x;
    }

    #[inline]
    pub fn set_min_y(&mut self, new_min_y: usize) {
        self.min_y = new_min_y;
    }

    #[inline]
    pub fn set_max_x(&mut self, new_max_x: usize) {
        self.max_x = new_max_x;
    }

    #[inline]
    pub fn set_max_y(&mut self, new_max_y: usize) {
        self.max_y = new_max_y;
    }

    #[inline]
    pub fn crop_in_framebuffer(&self, surface: &render::FrameBuffer) -> Option<Self> {
        let min_x = self.min_x.min(surface.width());
        let min_y = self.min_y.min(surface.height());
        let max_x = self.max_x.min(surface.width());
        let max_y = self.max_y.min(surface.height());

        if min_x >= max_x || min_y >= max_y {
            return None;
        }

        Some(Self {
            min_x,
            min_y,
            max_x,
            max_y,
        })
    }
}

impl fmt::Display for BoundingBox {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "boundingbox ({:.4}, {:.4}, {:.4}, {:.4})",
            self.min_x, self.min_y, self.max_x, self.max_y
        )
    }
}

pub trait ToBoundingBox {
    fn to_bounding_box(&self) -> BoundingBox;
}

impl ToBoundingBox for math::Shape {
    #[inline]
    fn to_bounding_box(&self) -> BoundingBox {
        match self {
            math::Shape::Segment(segment) => segment.to_bounding_box(),
            math::Shape::Triangle(triangle) => triangle.to_bounding_box(),
            math::Shape::Quad(quad) => quad.to_bounding_box(),
            math::Shape::CvxPoly(cvx_poly) => cvx_poly.to_bounding_box(),
            math::Shape::CavePoly(cave_poly) => cave_poly.to_bounding_box(),
            math::Shape::Circle(_) => unimplemented!(),
        }
    }
}

impl ToBoundingBox for math::Segment {
    #[inline]
    fn to_bounding_box(&self) -> BoundingBox {
        BoundingBox::from_verts_array(&[self.a, self.b])
    }
}

impl ToBoundingBox for math::Triangle {
    #[inline]
    fn to_bounding_box(&self) -> BoundingBox {
        BoundingBox::from_verts_array(&[self.a, self.b, self.c])
    }
}

impl ToBoundingBox for math::Quad {
    #[inline]
    fn to_bounding_box(&self) -> BoundingBox {
        BoundingBox::from_verts_array(&[self.a, self.b, self.c, self.d])
    }
}

impl ToBoundingBox for math::CvxPoly {
    #[inline]
    fn to_bounding_box(&self) -> BoundingBox {
        BoundingBox::from_verts_slice(&self.verts)
    }
}

impl ToBoundingBox for math::CavePoly {
    #[inline]
    fn to_bounding_box(&self) -> BoundingBox {
        BoundingBox::from_verts_slice(&self.verts)
    }
}
