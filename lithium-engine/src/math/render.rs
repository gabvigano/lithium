use std::fmt;

use bincode::{Decode, Encode};
use serde::Deserialize;

#[derive(Debug, Clone, Copy, PartialEq, Encode, Decode, Deserialize)]
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

impl fmt::Display for Color {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "rgba ({}, {}, {}, {})", self.r, self.g, self.b, self.a)
    }
}
