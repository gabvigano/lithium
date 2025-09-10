use crate::ecs::{self, components};

pub struct World {
    pub transform: ecs::SparseSet<components::Transform>,
    pub rigid_body: ecs::SparseSet<components::RigidBody>,
    pub surface: ecs::SparseSet<components::Surface>,
    pub shape: ecs::SparseSet<components::Shape>,
    pub material: ecs::SparseSet<components::Material>,
}

impl World {
    pub fn new() -> Self {
        Self {
            transform: ecs::SparseSet::new(),
            rigid_body: ecs::SparseSet::new(),
            surface: ecs::SparseSet::new(),
            shape: ecs::SparseSet::new(),
            material: ecs::SparseSet::new(),
        }
    }
}
