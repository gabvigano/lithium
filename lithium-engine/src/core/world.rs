use crate::{
    ecs::{self, components},
    math,
};

pub struct World {
    pub transform: ecs::SparseSet<components::Transform>,
    pub rotation_matrix: ecs::SparseSet<components::RotationMatrix>,
    pub translation: ecs::SparseSet<components::Translation>,
    pub rotation: ecs::SparseSet<components::Rotation>,
    pub surface: ecs::SparseSet<components::Surface>,
    pub shape: ecs::SparseSet<math::Shape>,
    pub material: ecs::SparseSet<components::Material>,
}

impl World {
    pub fn new() -> Self {
        Self {
            transform: ecs::SparseSet::new(),
            rotation_matrix: ecs::SparseSet::new(),
            translation: ecs::SparseSet::new(),
            rotation: ecs::SparseSet::new(),
            surface: ecs::SparseSet::new(),
            shape: ecs::SparseSet::new(),
            material: ecs::SparseSet::new(),
        }
    }

    pub fn reset(&mut self) {
        *self = Self::new();
    }
}
