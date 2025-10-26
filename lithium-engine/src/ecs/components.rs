use crate::{core::error, math};

use serde::Deserialize;
use std::fmt;

#[derive(Deserialize)]
pub struct TransformSpec {
    pub pos: math::Vec2,
    pub rot: math::Vec2,
}

#[derive(Clone, Debug)]
pub struct Transform {
    pub(crate) pos: math::Vec2,
    pub(crate) rot: math::Vec2,
}

impl Transform {
    #[inline]
    pub fn new(pos: math::Vec2, rot: math::Vec2) -> Self {
        Self { pos, rot }
    }

    #[inline]
    pub fn pos(&self) -> math::Vec2 {
        self.pos
    }

    #[inline]
    pub fn rot(&self) -> math::Vec2 {
        self.rot
    }

    #[inline]
    pub fn set_pos(&mut self, new_pos: math::Vec2) {
        self.pos = new_pos
    }

    #[inline]
    pub fn set_rot(&mut self, new_rot: math::Vec2) {
        self.rot = new_rot
    }
}

impl fmt::Display for Transform {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "transform (pos: {}, rot: {})", self.pos, self.rot)
    }
}

impl From<TransformSpec> for Transform {
    fn from(spec: TransformSpec) -> Self {
        Self::new(spec.pos, spec.rot)
    }
}

#[derive(Deserialize)]
pub struct TranslationSpec {
    pub lin_vel: math::Vec2,
    pub force: math::Vec2,
    pub mass: f32,
}

#[derive(Clone, Debug)]
pub struct Translation {
    pub(crate) lin_vel: math::Vec2,
    pub(crate) force: math::Vec2,
    mass: f32,
    inv_mass: f32,
    pub(crate) rest: bool,
}

impl Translation {
    #[inline]
    pub fn new(lin_vel: math::Vec2, force: math::Vec2, mass: f32) -> Result<Self, error::MathError> {
        if mass <= 0.0 {
            return Err(error::MathError::NonPositive("mass"));
        }

        Ok(Self {
            lin_vel,
            force,
            mass,
            inv_mass: 1.0 / mass,
            rest: false,
        })
    }

    #[inline]
    pub fn lin_vel(&self) -> math::Vec2 {
        self.lin_vel
    }

    #[inline]
    pub fn force(&self) -> math::Vec2 {
        self.force
    }

    #[inline]
    pub fn mass(&self) -> f32 {
        self.mass
    }

    #[inline]
    pub fn inv_mass(&self) -> f32 {
        self.inv_mass
    }

    #[inline]
    pub fn rest(&self) -> bool {
        self.rest
    }

    #[inline]
    pub fn set_lin_vel(&mut self, new_lin_vel: math::Vec2) {
        self.lin_vel = new_lin_vel;
    }

    #[inline]
    pub fn set_force(&mut self, new_force: math::Vec2) {
        self.force = new_force;
    }

    #[inline]
    pub fn set_mass(&mut self, new_mass: f32) {
        self.mass = new_mass;
        self.inv_mass = 1.0 / new_mass;
    }

    #[inline]
    pub fn set_rest(&mut self, new_rest: bool) {
        self.rest = new_rest;
    }
}

impl fmt::Display for Translation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "translation (lin_vel: {:.4}, force: {:.4}, mass: {:.4}, rest: {})",
            self.lin_vel, self.force, self.mass, self.rest
        )
    }
}

impl TryFrom<TranslationSpec> for Translation {
    type Error = error::MathError;

    fn try_from(spec: TranslationSpec) -> Result<Self, Self::Error> {
        Self::new(spec.lin_vel, spec.force, spec.mass)
    }
}

#[derive(Deserialize)]
pub struct RotationSpec {
    pub ang_vel: f32,
    pub torque: f32,
    pub inertia: f32,
}

#[derive(Clone, Debug)]
pub struct Rotation {
    pub(crate) ang_vel: f32,
    pub(crate) torque: f32,
    inertia: f32,
    inv_inertia: f32,
}

impl Rotation {
    #[inline]
    pub fn new(ang_vel: f32, torque: f32, inertia: f32) -> Result<Self, error::MathError> {
        if inertia <= 0.0 {
            return Err(error::MathError::NonPositive("inertia"));
        }

        Ok(Self {
            ang_vel,
            torque,
            inertia,
            inv_inertia: 1.0 / inertia,
        })
    }

    #[inline]
    pub fn ang_vel(&self) -> f32 {
        self.ang_vel
    }

    #[inline]
    pub fn torque(&self) -> f32 {
        self.torque
    }

    #[inline]
    pub fn inertia(&self) -> f32 {
        self.inertia
    }

    #[inline]
    pub fn inv_inertia(&self) -> f32 {
        self.inv_inertia
    }

    #[inline]
    pub fn set_ang_vel(&mut self, new_ang_vel: f32) {
        self.ang_vel = new_ang_vel;
    }

    #[inline]
    pub fn set_torque(&mut self, new_torque: f32) {
        self.torque = new_torque;
    }

    #[inline]
    pub fn set_inertia(&mut self, new_inertia: f32) {
        self.inertia = new_inertia;
        self.inv_inertia = 1.0 / new_inertia;
    }
}

impl fmt::Display for Rotation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "rotation (ang_vel: {:.4}, torque: {:.4}, inertia: {:.4})",
            self.ang_vel, self.torque, self.inertia
        )
    }
}

impl TryFrom<RotationSpec> for Rotation {
    type Error = error::MathError;

    fn try_from(spec: RotationSpec) -> Result<Self, Self::Error> {
        Self::new(spec.ang_vel, spec.torque, spec.inertia)
    }
}

#[derive(Deserialize)]
pub struct SurfaceSpec {
    pub elast: f32,
    pub static_friction: f32,
    pub kinetic_friction: f32,
}

#[derive(Clone, Debug)]
pub struct Surface {
    pub(crate) elast: f32,
    pub(crate) static_friction: f32,
    pub(crate) kinetic_friction: f32,
}

impl Surface {
    #[inline]
    pub fn new(elast: f32, static_friction: f32, kinetic_friction: f32) -> Self {
        Self {
            elast,
            static_friction,
            kinetic_friction,
        }
    }

    #[inline]
    pub fn elast(&self) -> f32 {
        self.elast
    }

    #[inline]
    pub fn static_friction(&self) -> f32 {
        self.static_friction
    }

    #[inline]
    pub fn kinetic_friction(&self) -> f32 {
        self.kinetic_friction
    }

    #[inline]
    pub fn set_elast(&mut self, new_elast: f32) {
        self.elast = new_elast;
    }

    #[inline]
    pub fn set_static_friction(&mut self, new_static_friction: f32) {
        self.static_friction = new_static_friction;
    }

    #[inline]
    pub fn set_kinetic_friction(&mut self, new_kinetic_friction: f32) {
        self.kinetic_friction = new_kinetic_friction;
    }
}

impl fmt::Display for Surface {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "surface (elast: {:.4})", self.elast)
    }
}

impl From<SurfaceSpec> for Surface {
    fn from(spec: SurfaceSpec) -> Self {
        Self::new(spec.elast, spec.static_friction, spec.kinetic_friction)
    }
}

#[derive(Deserialize)]
pub struct MaterialSpec {
    pub color: math::Color,
    pub layer: usize,
    pub show: bool,
}

#[derive(Clone, Debug)]
pub struct Material {
    pub(crate) color: math::Color,
    pub(crate) layer: usize,
    pub(crate) show: bool,
}

impl Material {
    #[inline]
    pub fn new(color: math::Color, layer: usize, show: bool) -> Self {
        Self { color, layer, show }
    }

    #[inline]
    pub fn color(&self) -> math::Color {
        self.color
    }

    #[inline]
    pub fn layer(&self) -> usize {
        self.layer
    }

    #[inline]
    pub fn show(&self) -> bool {
        self.show
    }

    #[inline]
    pub fn set_color(&mut self, new_color: math::Color) {
        self.color = new_color;
    }

    #[inline]
    pub fn set_layer(&mut self, new_layer: usize) {
        self.layer = new_layer;
    }

    #[inline]
    pub fn set_show(&mut self, new_show: bool) {
        self.show = new_show;
    }
}

impl fmt::Display for Material {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "material (color: {}, layer: {}, show: {})",
            self.color, self.layer, self.show
        )
    }
}

impl From<MaterialSpec> for Material {
    fn from(spec: MaterialSpec) -> Self {
        Self::new(spec.color, spec.layer, spec.show)
    }
}

#[derive(Deserialize)]
pub struct StaticSpec {
    pub transform: TransformSpec,
    pub surface: SurfaceSpec,
    pub shape: math::Shape,
    pub material: MaterialSpec,
}

#[derive(Deserialize)]
pub struct DynamicSpec {
    pub transform: TransformSpec,
    pub translation: TranslationSpec,
    pub surface: SurfaceSpec,
    pub shape: math::Shape,
    pub material: MaterialSpec,
}
