use crate::{
    core::error,
    ecs::{components, entities},
};

use std::any::Any;

/// sparseset storage for components of type T:
/// - components[dense] stores the component value
/// - entities[dense] stores the entity that owns components[dense]
/// - sparse[entity] stores Some(dense) if that entity has a component, otherwise None
pub struct SparseSet<T> {
    components: Vec<T>,
    entities: Vec<entities::Entity>,
    sparse: Vec<Option<usize>>,
}

impl<T> SparseSet<T> {
    #[inline]
    pub fn new() -> Self {
        Self {
            components: Vec::new(),
            entities: Vec::new(),
            sparse: Vec::new(),
        }
    }

    #[inline]
    fn ensure_sparse_idx(&mut self, entity: entities::Entity) -> usize {
        let sparse_idx = entity as usize;

        if sparse_idx >= self.sparse.len() {
            self.sparse.resize(sparse_idx + 1, None);
        }

        sparse_idx
    }

    #[inline]
    fn dense_idx(&self, entity: entities::Entity) -> Option<usize> {
        self.sparse.get(entity as usize).and_then(|slot| *slot)
    }

    #[inline]
    fn push_new(&mut self, entity: entities::Entity, component: T) {
        let sparse_idx = self.ensure_sparse_idx(entity);
        let dense_idx = self.components.len();

        self.components.push(component);
        self.entities.push(entity);
        self.sparse[sparse_idx] = Some(dense_idx);
    }

    #[inline]
    pub fn insert(&mut self, entity: entities::Entity, component: T) -> Result<(), error::ComponentError> {
        let sparse_idx = self.ensure_sparse_idx(entity);

        if self.sparse[sparse_idx].is_some() {
            return Err(error::ComponentError::DuplicateComponent(entity));
        }

        self.push_new(entity, component);
        Ok(())
    }

    #[inline]
    pub fn upsert(&mut self, entity: entities::Entity, component: T) {
        match self.sparse.get(entity as usize).and_then(|slot| *slot) {
            Some(dense_idx) => {
                self.components[dense_idx] = component;
            }
            None => {
                self.push_new(entity, component);
            }
        }
    }

    #[inline]
    pub fn update(&mut self, entity: entities::Entity, component: T) -> Result<(), error::ComponentError> {
        let dense_idx = self
            .dense_idx(entity)
            .ok_or(error::ComponentError::ComponentNotFound(entity))?;

        self.components[dense_idx] = component;
        Ok(())
    }

    #[inline]
    pub fn remove(&mut self, entity: entities::Entity) -> Option<T> {
        let sparse_idx = entity as usize;
        let dense_idx = self.sparse.get_mut(sparse_idx)?.take()?;

        let last_dense_idx = self.components.len() - 1;

        if dense_idx != last_dense_idx {
            self.components.swap(dense_idx, last_dense_idx);
            self.entities.swap(dense_idx, last_dense_idx);

            let moved_entity = self.entities[dense_idx];
            self.sparse[moved_entity as usize] = Some(dense_idx);
        }

        self.entities.pop();
        self.components.pop()
    }

    #[inline]
    pub fn get(&self, entity: entities::Entity) -> Option<&T> {
        let dense_idx = self.dense_idx(entity)?;
        self.components.get(dense_idx)
    }

    #[inline]
    pub fn get_mut(&mut self, entity: entities::Entity) -> Option<&mut T> {
        let dense_idx = self.dense_idx(entity)?;
        self.components.get_mut(dense_idx)
    }

    #[inline]
    pub fn get2(&self, entity_1: entities::Entity, entity_2: entities::Entity) -> (Option<&T>, Option<&T>) {
        if entity_1 == entity_2 {
            return (None, None);
        }

        (self.get(entity_1), self.get(entity_2))
    }

    #[inline]
    pub fn get2_mut(
        &mut self,
        entity_1: entities::Entity,
        entity_2: entities::Entity,
    ) -> (Option<&mut T>, Option<&mut T>) {
        if entity_1 == entity_2 {
            return (None, None);
        }

        let idx_1 = self.dense_idx(entity_1);
        let idx_2 = self.dense_idx(entity_2);

        match (idx_1, idx_2) {
            (None, None) => (None, None),
            (Some(idx), None) => (self.components.get_mut(idx), None),
            (None, Some(idx)) => (None, self.components.get_mut(idx)),
            (Some(idx_1), Some(idx_2)) => {
                if idx_1 < idx_2 {
                    let (left, right) = self.components.split_at_mut(idx_2);
                    (Some(&mut left[idx_1]), Some(&mut right[0]))
                } else {
                    let (left, right) = self.components.split_at_mut(idx_1);
                    (Some(&mut right[0]), Some(&mut left[idx_2]))
                }
            }
        }
    }

    #[inline]
    pub fn iter(&self) -> impl Iterator<Item = (entities::Entity, &T)> {
        self.entities.iter().copied().zip(self.components.iter())
    }

    #[inline]
    pub fn iter_mut(&mut self) -> impl Iterator<Item = (entities::Entity, &mut T)> {
        self.entities.iter().copied().zip(self.components.iter_mut())
    }

    #[inline]
    pub fn get_ents(&self) -> &[entities::Entity] {
        &self.entities
    }

    #[inline]
    pub fn get_comps(&self) -> &[T] {
        &self.components
    }
}

pub trait ErasedStorage: 'static {
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;

    fn get_any(&self, entity: entities::Entity) -> Option<&dyn Any>;
    fn get_any_mut(&mut self, entity: entities::Entity) -> Option<&mut dyn Any>;
}

impl<T: components::UserComponent> ErasedStorage for SparseSet<T> {
    #[inline]
    fn as_any(&self) -> &dyn Any {
        self
    }

    #[inline]
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    #[inline]
    fn get_any(&self, entity: entities::Entity) -> Option<&dyn Any> {
        self.get(entity).map(|c| c as &dyn Any)
    }

    #[inline]
    fn get_any_mut(&mut self, entity: entities::Entity) -> Option<&mut dyn Any> {
        self.get_mut(entity).map(|c| c as &mut dyn Any)
    }
}
