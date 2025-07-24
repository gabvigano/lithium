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

        world.pos.insert(entity, obj.pos);
        world.elast.insert(entity, obj.elast);
        world.shape.insert(entity, obj.shape);
        world.color.insert(entity, obj.color);
        world.layer.insert(entity, obj.layer);
        world.show.insert(entity, obj.show);
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

        world.start_pos.insert(entity, obj.pos);
        world.pos.insert(entity, obj.pos);
        world.vel.insert(entity, obj.vel);
        world.acc.insert(entity, obj.acc);
        world.rest.insert(entity, obj.rest);
        world.mass.insert(entity, obj.mass);
        world.elast.insert(entity, obj.elast);
        world.shape.insert(entity, obj.shape);
        world.color.insert(entity, obj.color);
        world.layer.insert(entity, obj.layer);
        world.show.insert(entity, obj.show);
    }

    entities
}
