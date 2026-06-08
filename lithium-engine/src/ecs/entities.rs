pub type Entity = u32;

pub struct EntityManager {
    next_id: Entity,
}

impl EntityManager {
    #[inline]
    pub fn new() -> Self {
        Self { next_id: 0 }
    }

    #[inline]
    pub fn create(&mut self) -> Entity {
        let id = self.next_id;
        self.next_id += 1;
        id
    }

    #[inline]
    pub fn current(&self) -> Entity {
        self.next_id
    }

    #[inline]
    pub fn skip_to(&mut self, idx: Entity) {
        self.next_id = idx;
    }

    #[inline]
    pub fn reset(&mut self) {
        *self = Self::new();
    }
}
