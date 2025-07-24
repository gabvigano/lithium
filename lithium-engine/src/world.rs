use crate::ecs::{self, components}; //, entities};

pub struct World {
    pub start_pos: ecs::SparseSet<components::Pos>,
    pub pos: ecs::SparseSet<components::Pos>,
    pub vel: ecs::SparseSet<components::Vel>,
    pub acc: ecs::SparseSet<components::Acc>,
    pub force: ecs::SparseSet<components::Force>,
    pub rest: ecs::SparseSet<components::Rest>,
    pub mass: ecs::SparseSet<components::Mass>,
    pub elast: ecs::SparseSet<components::Elast>,
    pub dir: ecs::SparseSet<components::Dir>,

    pub shape: ecs::SparseSet<components::Shape>,
    pub color: ecs::SparseSet<components::Color>,
    pub layer: ecs::SparseSet<components::Layer>,
    pub show: ecs::SparseSet<components::Show>,
}

impl World {
    pub fn new() -> Self {
        Self {
            start_pos: ecs::SparseSet::new(),
            pos: ecs::SparseSet::new(),
            vel: ecs::SparseSet::new(),
            acc: ecs::SparseSet::new(),
            force: ecs::SparseSet::new(),
            rest: ecs::SparseSet::new(),
            mass: ecs::SparseSet::new(),
            elast: ecs::SparseSet::new(),
            dir: ecs::SparseSet::new(),

            shape: ecs::SparseSet::new(),
            color: ecs::SparseSet::new(),
            layer: ecs::SparseSet::new(),
            show: ecs::SparseSet::new(),
        }
    }
}
