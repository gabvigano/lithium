use crate::core::error;

pub mod components;
pub mod entities;
pub mod systems;

pub struct SparseSet<T> {
    components: Vec<T>,
    entities: Vec<entities::Entity>,
    sparse: Vec<Option<usize>>,
}

impl<T> SparseSet<T> {
    pub fn new() -> Self {
        Self {
            components: Vec::new(),
            entities: Vec::new(),
            sparse: Vec::new(),
        }
    }

    pub fn insert(&mut self, entity: entities::Entity, component: T) -> Result<(), error::ComponentError> {
        let index = self.components.len();
        let sparse_id = entity as usize;

        // ensure self.sparse is long enough
        if sparse_id >= self.sparse.len() {
            self.sparse.resize(sparse_id + 1, None);
        }

        if self.sparse[sparse_id].is_some() {
            return Err(error::ComponentError::AlreadyExistingComponent(entity));
        }

        self.components.push(component);
        self.entities.push(entity);
        self.sparse[sparse_id] = Some(index);

        Ok(())
    }

    pub fn remove(&mut self, entity: entities::Entity) -> Option<T> {
        let sparse_id = entity as usize;
        let index = self.sparse.get_mut(sparse_id)?.take()?;

        // swap the last item with the one to remove
        let last_index = self.components.len() - 1;
        self.components.swap(index, last_index);
        self.entities.swap(index, last_index);

        // update the sparse index of the moved entity
        let moved_entity = self.entities[index];
        self.sparse[moved_entity as usize] = Some(index);

        // remove the entity to remove and return the associated component
        self.entities.pop();
        let removed = self.components.pop();

        removed
    }

    pub fn set(&mut self, entity: entities::Entity, component: T) -> Result<(), error::ComponentError> {
        let sparse_id = entity as usize;
        let index = *self
            .sparse
            .get_mut(sparse_id)
            .ok_or(error::ComponentError::MissingComponent(entity))?
            .as_ref()
            .ok_or(error::ComponentError::MissingComponent(entity))?;

        self.components[index] = component;
        Ok(())
    }

    pub fn get(&self, entity: entities::Entity) -> Option<&T> {
        let sparse_id = entity as usize;
        self.sparse.get(sparse_id)?.map(|index| &self.components[index])
    }

    pub fn get_mut(&mut self, entity: entities::Entity) -> Option<&mut T> {
        let sparse_id = entity as usize;
        self.sparse.get(sparse_id)?.map(|index| &mut self.components[index])
    }

    pub fn iter(&self) -> impl Iterator<Item = (entities::Entity, &T)> {
        self.entities.iter().cloned().zip(self.components.iter())
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = (entities::Entity, &mut T)> {
        self.entities.iter().cloned().zip(self.components.iter_mut())
    }

    pub fn get_ents(&self) -> Vec<entities::Entity> {
        self.entities.clone()
    }

    pub fn get_ref(&self) -> &Vec<T> {
        &self.components
    }
}
