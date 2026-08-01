use crate::math::Centroid;
use crate::{base, math};

use std::{any::Any, fmt};

use bincode::{Decode, Encode};
use serde::Deserialize;

pub trait UserComponent: Any + 'static {
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

#[derive(Deserialize)]
pub struct TransformSpec {
    pub pos: math::Vec2,
}

#[derive(Debug, Clone, PartialEq, Encode, Decode)]
pub struct Transform {
    pub(crate) pos: math::Vec2,
}

impl Transform {
    #[inline]
    pub const fn new(pos: math::Vec2) -> Self {
        Self { pos }
    }

    #[inline]
    pub fn pos(&self) -> math::Vec2 {
        self.pos
    }

    #[inline]
    pub fn pos_mut(&mut self) -> &mut math::Vec2 {
        &mut self.pos
    }
}

impl fmt::Display for Transform {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "transform (pos: {})", self.pos)
    }
}

impl From<TransformSpec> for Transform {
    fn from(spec: TransformSpec) -> Self {
        Self::new(spec.pos)
    }
}

#[derive(Deserialize)]
pub struct RotationMatrixSpec {
    pub rot_degrees: f32,
    pub pivot: math::Vec2,
}

impl RotationMatrixSpec {
    #[inline]
    pub fn to_rot_mat(&self) -> RotationMatrix {
        let mut rot = math::Radians::from_degrees(self.rot_degrees);
        rot.norm_mut();

        RotationMatrix::new(math::Mat2x3::from_rot_and_pivot(rot, self.pivot))
    }
}

#[derive(Debug, Clone, PartialEq, Encode, Decode)]
pub struct RotationMatrix {
    pub(crate) rot_mat: math::Mat2x3,
}

impl RotationMatrix {
    #[inline]
    pub const fn new(rot_mat: math::Mat2x3) -> Self {
        Self { rot_mat }
    }

    #[inline]
    pub const fn zero() -> Self {
        Self {
            rot_mat: math::Mat2x3::ZERO,
        }
    }

    #[inline]
    pub const fn one() -> Self {
        Self {
            rot_mat: math::Mat2x3::ONE,
        }
    }

    #[inline]
    pub const fn identity() -> Self {
        Self {
            rot_mat: math::Mat2x3::IDENTITY,
        }
    }

    #[inline]
    pub fn rot_mat(&self) -> &math::Mat2x3 {
        &self.rot_mat
    }

    #[inline]
    pub fn rot_mat_mut(&mut self) -> &mut math::Mat2x3 {
        &mut self.rot_mat
    }

    #[inline]
    pub fn update(&self, delta_rot: math::Radians, pivot: math::Vec2) -> Self {
        if delta_rot.0.abs() < math::EPS {
            // early return deltas close to 0
            return self.clone();
        }

        // compute the transformation for this rotation
        let transformation = math::Mat2x3::from_rot_and_pivot(delta_rot, pivot);

        // apply the rotation to the rotation matrix
        Self::new(self.rot_mat.pre_mul(&transformation))
    }

    #[inline]
    pub fn update_mut(&mut self, delta_rot: math::Radians, pivot: math::Vec2) -> bool {
        if delta_rot.0.abs() < math::EPS {
            // early return deltas close to 0
            return false;
        }

        // compute the transformation for this rotation
        let transformation = math::Mat2x3::from_rot_and_pivot(delta_rot, pivot);

        // apply the rotation to the rotation matrix
        self.rot_mat.pre_mul_mut(&transformation);

        true
    }
}

impl fmt::Display for RotationMatrix {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "rotation_matrix (rot_mat: {})", self.rot_mat)
    }
}

#[derive(Deserialize)]
pub struct TranslationSpec {
    pub lin_vel: math::Vec2,
    pub force: math::Vec2,
    pub mass: f32,
}

#[derive(Debug, Clone, PartialEq, Encode, Decode)]
pub struct Translation {
    pub(crate) lin_vel: math::Vec2,
    pub(crate) force: math::Vec2,
    mass: f32,
    inv_mass: f32,
    pub(crate) rest: bool,
}

impl Translation {
    #[inline]
    pub const fn new(lin_vel: math::Vec2, force: math::Vec2, mass: f32) -> Result<Self, base::MathError> {
        if mass <= 0.0 {
            return Err(base::MathError::NonPositive("mass"));
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
    pub fn lin_vel_mut(&mut self) -> &mut math::Vec2 {
        &mut self.lin_vel
    }

    #[inline]
    pub fn force_mut(&mut self) -> &mut math::Vec2 {
        &mut self.force
    }

    #[inline]
    pub fn set_mass(&mut self, new_mass: f32) {
        self.mass = new_mass;
        self.inv_mass = 1.0 / new_mass;
    }

    #[inline]
    pub fn rest_mut(&mut self) -> &mut bool {
        &mut self.rest
    }

    #[inline]
    pub fn apply_lin_vel_axis(&mut self, lin_vel: f32, axis: math::Axis) {
        match axis {
            math::Axis::X => {
                self.lin_vel.x += lin_vel;
            }
            math::Axis::Y => {
                self.lin_vel.y += lin_vel;
            }
        }
    }

    #[inline]
    pub fn apply_lin_vel(&mut self, lin_vel: math::Vec2) {
        self.lin_vel.add_mut(lin_vel);
    }

    #[inline]
    pub fn apply_force_axis(&mut self, force: f32, axis: math::Axis) {
        match axis {
            math::Axis::X => {
                self.force.x += force;
            }
            math::Axis::Y => {
                self.force.y += force;
            }
        }
    }

    #[inline]
    pub fn apply_force(&mut self, force: math::Vec2) {
        self.force.add_mut(force);
    }
}

impl fmt::Display for Translation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "translation (lin_vel: {}, force: {}, mass: {:.4}, rest: {})",
            self.lin_vel, self.force, self.mass, self.rest
        )
    }
}

impl TryFrom<TranslationSpec> for Translation {
    type Error = base::MathError;

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

#[derive(Debug, Clone, PartialEq, Encode, Decode)]
pub struct Rotation {
    pub(crate) ang_vel: f32,
    pub(crate) torque: f32,
    inertia: f32,
    inv_inertia: f32,
}

impl Rotation {
    #[inline]
    pub const fn new(ang_vel: f32, torque: f32, inertia: f32) -> Result<Self, base::MathError> {
        if inertia <= 0.0 {
            return Err(base::MathError::NonPositive("inertia"));
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
    pub fn ang_vel_mut(&mut self) -> &mut f32 {
        &mut self.ang_vel
    }

    #[inline]
    pub fn torque_mut(&mut self) -> &mut f32 {
        &mut self.torque
    }

    #[inline]
    pub fn set_inertia(&mut self, new_inertia: f32) {
        self.inertia = new_inertia;
        self.inv_inertia = 1.0 / new_inertia;
    }

    #[inline]
    pub fn apply_ang_vel(&mut self, ang_vel: f32) {
        self.ang_vel += ang_vel;
    }

    #[inline]
    pub fn apply_torque(&mut self, torque: f32) {
        self.torque += torque;
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
    type Error = base::MathError;

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

#[derive(Debug, Clone, PartialEq, Encode, Decode)]
pub struct Surface {
    pub(crate) elast: f32,
    pub(crate) static_friction: f32,
    pub(crate) kinetic_friction: f32,
}

impl Surface {
    #[inline]
    pub const fn new(elast: f32, static_friction: f32, kinetic_friction: f32) -> Self {
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
    pub fn elast_mut(&mut self) -> &mut f32 {
        &mut self.elast
    }

    #[inline]
    pub fn static_friction_mut(&mut self) -> &mut f32 {
        &mut self.static_friction
    }

    #[inline]
    pub fn kinetic_friction_mut(&mut self) -> &mut f32 {
        &mut self.kinetic_friction
    }
}

impl fmt::Display for Surface {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "surface (elast: {:.4}, static_friction: {:.4}, kinetic_friction: {:.4})",
            self.elast, self.static_friction, self.kinetic_friction
        )
    }
}

impl From<SurfaceSpec> for Surface {
    fn from(spec: SurfaceSpec) -> Self {
        Self::new(spec.elast, spec.static_friction, spec.kinetic_friction)
    }
}

#[derive(Deserialize)]
pub struct BodySpec {
    pub shape: math::Shape,
}

#[derive(Debug, Clone, PartialEq, Encode, Decode)]
pub struct Body {
    pub(crate) shape: math::Shape,
    pub(crate) centroid: math::Vec2,
}

impl Body {
    #[inline]
    pub fn new(shape: math::Shape) -> Self {
        let centroid = shape.centroid();
        Self { shape, centroid }
    }

    #[inline]
    pub fn shape(&self) -> &math::Shape {
        &self.shape
    }

    #[inline]
    pub fn centroid(&self) -> math::Vec2 {
        self.centroid
    }

    #[inline]
    pub fn set_shape(&mut self, new_shape: math::Shape) {
        self.shape = new_shape;
        self.centroid = self.shape.centroid()
    }
}

impl fmt::Display for Body {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "body (shape: {}, centroid: {})", self.shape, self.centroid)
    }
}

impl From<BodySpec> for Body {
    fn from(spec: BodySpec) -> Self {
        Self::new(spec.shape)
    }
}

#[derive(Deserialize)]
pub struct MaterialSpec {
    pub color: math::Color,
    pub layer: usize,
    pub show: bool,
}

#[derive(Debug, Clone, PartialEq, Encode, Decode)]
pub struct Material {
    pub(crate) color: math::Color,
    pub(crate) layer: usize,
    pub(crate) show: bool,
}

impl Material {
    #[inline]
    pub const fn new(color: math::Color, layer: usize, show: bool) -> Self {
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
    pub fn color_mut(&mut self) -> &mut math::Color {
        &mut self.color
    }

    #[inline]
    pub fn layer_mut(&mut self) -> &mut usize {
        &mut self.layer
    }

    #[inline]
    pub fn show_mut(&mut self) -> &mut bool {
        &mut self.show
    }
}

impl fmt::Display for Material {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "material (color: {}, layer: {}, show: {})", self.color, self.layer, self.show)
    }
}

impl From<MaterialSpec> for Material {
    fn from(spec: MaterialSpec) -> Self {
        Self::new(spec.color, spec.layer, spec.show)
    }
}
