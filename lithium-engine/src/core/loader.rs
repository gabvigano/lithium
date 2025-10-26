use crate::{
    core::{error, world::World},
    ecs::{components, entities},
};

pub fn load_static_map(
    path: &str,
    world: &mut World,
    entity_manager: &mut entities::EntityManager,
) -> Result<Vec<entities::Entity>, error::EngineError> {
    let data = std::fs::read_to_string(path).map_err(error::FileError::from)?;
    let specs: Vec<components::StaticSpec> = ron::from_str(&data).map_err(error::FileError::from)?;
    let mut entities = Vec::with_capacity(specs.len());

    for obj in specs {
        // validate shape before allocating an entity
        obj.shape.validate()?;

        let entity = entity_manager.create();
        entities.push(entity);

        let initial_tranform: components::Transform = obj.transform.into();

        world.initial_transform.insert(entity, initial_tranform.clone())?;
        world.transform.insert(entity, initial_tranform)?;
        world.surface.insert(entity, obj.surface.into())?;
        world.shape.insert(entity, obj.shape)?;
        world.material.insert(entity, obj.material.into())?;
    }

    Ok(entities)
}

pub fn load_dynamic_map(
    path: &str,
    world: &mut World,
    entity_manager: &mut entities::EntityManager,
) -> Result<Vec<entities::Entity>, error::EngineError> {
    let data = std::fs::read_to_string(path).map_err(error::FileError::from)?;
    let specs: Vec<components::DynamicSpec> = ron::from_str(&data).map_err(error::FileError::from)?;
    let mut entities = Vec::with_capacity(specs.len());

    for obj in specs {
        // validate shape before allocating an entity
        obj.shape.validate()?;

        let entity = entity_manager.create();
        entities.push(entity);

        let initial_tranform: components::Transform = obj.transform.into();

        world.initial_transform.insert(entity, initial_tranform.clone())?;
        world.transform.insert(entity, initial_tranform)?;
        world.translation.insert(entity, obj.translation.try_into()?)?;
        world.surface.insert(entity, obj.surface.into())?;
        world.shape.insert(entity, obj.shape)?;
        world.material.insert(entity, obj.material.into())?;
    }

    Ok(entities)
}
