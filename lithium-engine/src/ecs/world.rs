use crate::{base, ecs};

pub struct World<const N: usize> {
    pub engine: EngineComponents, // todo: set back to pub(crate) when a better way to keep track of world changes is implemented
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
    pub fn new(items: [Box<dyn ecs::ErasedStorage>; N]) -> Self {
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

#[derive(Debug, Clone)] // todo: maybe derive for World and UserComponents too?
pub struct EngineComponents {
    pub transform: ecs::SparseSet<ecs::Transform>,
    pub rotation_matrix: ecs::SparseSet<ecs::RotationMatrix>,
    pub translation: ecs::SparseSet<ecs::Translation>,
    pub rotation: ecs::SparseSet<ecs::Rotation>,
    pub surface: ecs::SparseSet<ecs::Surface>,
    pub body: ecs::SparseSet<ecs::Body>,
    pub material: ecs::SparseSet<ecs::Material>,
}

impl EngineComponents {
    #[inline]
    pub fn new() -> Self {
        Self {
            transform: ecs::SparseSet::new(),
            rotation_matrix: ecs::SparseSet::new(),
            translation: ecs::SparseSet::new(),
            rotation: ecs::SparseSet::new(),
            surface: ecs::SparseSet::new(),
            body: ecs::SparseSet::new(),
            material: ecs::SparseSet::new(),
        }
    }
}

pub struct UserComponents<const N: usize> {
    items: [Box<dyn ecs::ErasedStorage>; N],
}

impl UserComponents<0> {
    #[inline]
    pub fn empty() -> Self {
        Self { items: [] }
    }
}

impl<const N: usize> UserComponents<N> {
    #[inline]
    pub fn new(items: [Box<dyn ecs::ErasedStorage>; N]) -> Self {
        Self { items }
    }

    #[inline]
    pub fn get<T: ecs::UserComponent>(&self, item: usize) -> Result<&ecs::SparseSet<T>, base::ComponentError> {
        let item = self
            .items
            .get(item)
            .map(|i| &**i)
            .ok_or(base::ComponentError::ComponentOutOfRange(item))?;
        let any_ref = item.as_any();
        any_ref
            .downcast_ref::<ecs::SparseSet<T>>()
            .ok_or(base::ComponentError::MismatchingComponent())
    }

    #[inline]
    pub fn get_mut<T: ecs::UserComponent>(&mut self, item: usize) -> Result<&mut ecs::SparseSet<T>, base::ComponentError> {
        let item = self
            .items
            .get_mut(item)
            .map(|i| &mut **i)
            .ok_or(base::ComponentError::ComponentOutOfRange(item))?;
        let any_ref = item.as_any_mut();
        any_ref
            .downcast_mut::<ecs::SparseSet<T>>()
            .ok_or(base::ComponentError::MismatchingComponent())
    }
}
