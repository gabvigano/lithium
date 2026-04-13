use crate::{
    core::error,
    ecs::{components, entities},
};

use std::any::Any;

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
    pub fn insert(&mut self, entity: entities::Entity, component: T) -> Result<(), error::ComponentError> {
        let index = self.components.len();
        let sparse_id = entity as usize;

        // ensure self.sparse is long enough
        if sparse_id >= self.sparse.len() {
            self.sparse.resize(sparse_id + 1, None);
        }

        if self.sparse[sparse_id].is_some() {
            return Err(error::ComponentError::DuplicateComponent(entity));
        }

        self.components.push(component);
        self.entities.push(entity);
        self.sparse[sparse_id] = Some(index);

        Ok(())
    }

    #[inline]
    pub fn remove(&mut self, entity: entities::Entity) -> Option<T> {
        let sparse_id = entity as usize;
        let index = self.sparse.get_mut(sparse_id)?.take()?;

        let last_index = self.components.len() - 1;

        if index != last_index {
            // swap the last item with the one to remove
            self.components.swap(index, last_index);
            self.entities.swap(index, last_index);

            // update the sparse index of the moved entity
            let moved_entity = self.entities[index];
            self.sparse[moved_entity as usize] = Some(index);
        }

        // remove the entity to remove and return the associated component
        self.entities.pop();
        let removed = self.components.pop();

        removed
    }

    #[inline]
    pub fn set(&mut self, entity: entities::Entity, component: T) -> Result<(), error::ComponentError> {
        let sparse_id = entity as usize;
        let index = *self
            .sparse
            .get_mut(sparse_id)
            .ok_or(error::ComponentError::ComponentNotFound(entity))?
            .as_ref()
            .ok_or(error::ComponentError::ComponentNotFound(entity))?;

        self.components[index] = component;
        Ok(())
    }

    #[inline]
    pub fn get(&self, entity: entities::Entity) -> Option<&T> {
        let sparse_id = entity as usize;
        self.sparse.get(sparse_id)?.map(|index| &self.components[index])
    }

    #[inline]
    pub fn get_mut(&mut self, entity: entities::Entity) -> Option<&mut T> {
        let sparse_id = entity as usize;
        self.sparse.get(sparse_id)?.map(|index| &mut self.components[index])
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

        let index_1 = self.sparse.get(entity_1 as usize).and_then(|x| *x);
        let index_2 = self.sparse.get(entity_2 as usize).and_then(|x| *x);

        match (index_1, index_2) {
            (None, None) => (None, None),
            (Some(index), None) => (self.components.get_mut(index), None),
            (None, Some(index)) => (None, self.components.get_mut(index)),
            (Some(index_1), Some(index_2)) => {
                if index_1 < index_2 {
                    let (left, right) = self.components.split_at_mut(index_2);
                    (Some(&mut left[index_1]), Some(&mut right[0]))
                } else {
                    let (left, right) = self.components.split_at_mut(index_1);
                    (Some(&mut right[0]), Some(&mut left[index_2]))
                }
            }
        }
    }

    #[inline]
    pub fn iter(&self) -> impl Iterator<Item = (entities::Entity, &T)> {
        self.entities.iter().cloned().zip(self.components.iter())
    }

    #[inline]
    pub fn iter_mut(&mut self) -> impl Iterator<Item = (entities::Entity, &mut T)> {
        self.entities.iter().cloned().zip(self.components.iter_mut())
    }

    #[inline]
    pub fn get_ents(&self) -> Vec<entities::Entity> {
        self.entities.clone()
    }

    #[inline]
    pub fn get_ref(&self) -> &Vec<T> {
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
