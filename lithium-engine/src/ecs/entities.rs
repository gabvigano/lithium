pub type Entity = u32;

pub struct EntityManager {
    next_id: Entity,
}

impl EntityManager {
    pub fn new() -> Self {
        EntityManager { next_id: 0 }
    }

    pub fn create(&mut self) -> Entity {
        let id = self.next_id;
        self.next_id += 1;
        id
    }

    pub fn skip_to(&mut self, idx: Entity) {
        self.next_id = idx;
    }

    pub fn reset(&mut self) {
        *self = Self::new();
    }
}
