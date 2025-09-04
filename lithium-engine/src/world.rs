use crate::ecs::{self, components};

pub struct World {
    pub transform: ecs::SparseSet<components::Transform>,
    pub rigid_body: ecs::SparseSet<components::RigidBody>,
    pub shape: ecs::SparseSet<components::Shape>,
    pub collider: ecs::SparseSet<components::Collider>,
    pub material: ecs::SparseSet<components::Material>,
}

impl World {
    pub fn new() -> Self {
        Self {
            transform: ecs::SparseSet::new(),
            rigid_body: ecs::SparseSet::new(),
            shape: ecs::SparseSet::new(),
            collider: ecs::SparseSet::new(),
            material: ecs::SparseSet::new(),
        }
    }
}
