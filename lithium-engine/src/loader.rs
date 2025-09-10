use crate::{
    ecs::{components, entities},
    world::World,
};

pub fn load_static_map(
    path: &str,
    world: &mut World,
    entity_manager: &mut entities::EntityManager,
) -> Vec<entities::Entity> {
    let data = std::fs::read_to_string(path).expect("failed to read static map file");
    let map: Vec<components::Static> = ron::from_str(&data).expect("invalid static map format");
    let mut entities = Vec::new();

    for obj in map {
        let entity = entity_manager.create();
        entities.push(entity);

        world.transform.insert(entity, obj.transform);
        world.surface.insert(entity, obj.surface);
        world.shape.insert(entity, obj.shape);
        world.material.insert(entity, obj.material);
    }

    entities
}

pub fn load_dynamic_map(
    path: &str,
    world: &mut World,
    entity_manager: &mut entities::EntityManager,
) -> Vec<entities::Entity> {
    let data = std::fs::read_to_string(path).expect("failed to read dynamic map file");
    let map: Vec<components::Dynamic> = ron::from_str(&data).expect("invalid dynamic map format");
    let mut entities = Vec::new();

    for obj in map {
        let entity = entity_manager.create();
        entities.push(entity);

        world.transform.insert(entity, obj.transform);
        world.rigid_body.insert(entity, obj.rigid_body);
        world.surface.insert(entity, obj.surface);
        world.shape.insert(entity, obj.shape);
        world.material.insert(entity, obj.material);
    }

    entities
}
