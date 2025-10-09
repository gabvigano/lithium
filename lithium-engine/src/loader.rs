use crate::{
    ecs::{components, entities},
    error,
    world::World,
};

pub fn load_static_map(
    path: &str,
    world: &mut World,
    entity_manager: &mut entities::EntityManager,
) -> Result<Vec<entities::Entity>, error::EngineError> {
    let data = std::fs::read_to_string(path).map_err(error::FileError::from)?;
    let map: Vec<components::StaticSpec> = ron::from_str(&data).map_err(error::FileError::from)?;
    let mut entities = Vec::new();

    for obj in map {
        let entity = entity_manager.create();
        entities.push(entity);

        world.transform.insert(entity, obj.transform.into())?;
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
    let map: Vec<components::DynamicSpec> = ron::from_str(&data).map_err(error::FileError::from)?;
    let mut entities = Vec::new();

    for obj in map {
        let entity = entity_manager.create();
        entities.push(entity);

        world.transform.insert(entity, obj.transform.into())?;
        world.rigid_body.insert(entity, obj.rigid_body.try_into()?)?;
        world.surface.insert(entity, obj.surface.into())?;
        world.shape.insert(entity, obj.shape)?;
        world.material.insert(entity, obj.material.into())?;
    }

    Ok(entities)
}
