use crate::{
    core::error,
    ecs::{components, storage},
    math,
    prelude::SparseSet,
};

pub struct World<const N: usize> {
    pub(crate) engine: EngineComponents,
    user: UserComponents<N>,
}

impl World<0> {
    #[inline]
    pub fn default() -> Self {
        let engine = EngineComponents::new();
        let user = UserComponents::empty();

        Self { engine, user }
    }
}

impl<const N: usize> World<N> {
    #[inline]
    pub fn new(items: [Box<dyn storage::ErasedStorage>; N]) -> Self {
        let engine = EngineComponents::new();
        let user = UserComponents::new(items);

        Self { engine, user }
    }

    #[inline]
    pub fn engine(&self) -> &EngineComponents {
        &self.engine
    }

    #[inline]
    pub fn engine_mut(&mut self) -> &mut EngineComponents {
        &mut self.engine
    }

    #[inline]
    pub fn user(&self) -> &UserComponents<N> {
        &self.user
    }

    #[inline]
    pub fn user_mut(&mut self) -> &mut UserComponents<N> {
        &mut self.user
    }
}

pub struct EngineComponents {
    pub transform: storage::SparseSet<components::Transform>,
    pub rotation_matrix: storage::SparseSet<components::RotationMatrix>,
    pub translation: storage::SparseSet<components::Translation>,
    pub rotation: storage::SparseSet<components::Rotation>,
    pub surface: storage::SparseSet<components::Surface>,
    pub shape: storage::SparseSet<math::Shape>,
    pub material: storage::SparseSet<components::Material>,
}

impl EngineComponents {
    #[inline]
    pub fn new() -> Self {
        Self {
            transform: storage::SparseSet::new(),
            rotation_matrix: storage::SparseSet::new(),
            translation: storage::SparseSet::new(),
            rotation: storage::SparseSet::new(),
            surface: storage::SparseSet::new(),
            shape: storage::SparseSet::new(),
            material: storage::SparseSet::new(),
        }
    }
}

pub struct UserComponents<const N: usize> {
    items: [Box<dyn storage::ErasedStorage>; N],
}

impl UserComponents<0> {
    #[inline]
    pub fn empty() -> Self {
        Self { items: [] }
    }
}

impl<const N: usize> UserComponents<N> {
    #[inline]
    pub fn new(items: [Box<dyn storage::ErasedStorage>; N]) -> Self {
        Self { items }
    }

    #[inline]
    pub fn get<T: components::UserComponent>(&self, item: usize) -> Result<&SparseSet<T>, error::ComponentError> {
        let item = self
            .items
            .get(item)
            .map(|i| &**i)
            .ok_or(error::ComponentError::ComponentOutOfRange(item))?;
        let any_ref = item.as_any();
        any_ref
            .downcast_ref::<SparseSet<T>>()
            .ok_or(error::ComponentError::MismatchingComponent())
    }

    #[inline]
    pub fn get_mut<T: components::UserComponent>(
        &mut self,
        item: usize,
    ) -> Result<&mut SparseSet<T>, error::ComponentError> {
        let item = self
            .items
            .get_mut(item)
            .map(|i| &mut **i)
            .ok_or(error::ComponentError::ComponentOutOfRange(item))?;
        let any_ref = item.as_any_mut();
        any_ref
            .downcast_mut::<SparseSet<T>>()
            .ok_or(error::ComponentError::MismatchingComponent())
    }
}
