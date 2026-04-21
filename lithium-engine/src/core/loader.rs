use crate::{
    core::error,
    ecs::{components, entities, world::World},
    math::geometry::Validate,
};

use serde::Deserialize;
use serde_yaml::Value;
use std::{collections::HashMap, fs};

pub type FileEntity = u32;
type ComponentKey = (FileEntity, String);

#[derive(Deserialize, Clone, Debug)]
pub struct LoadableComponent {
    pub entity: u32,
    pub kind: String,
    pub data: Value,
}

pub struct MapCache {
    pub path: String,
    pub metadata: std::time::SystemTime,
    pub raw_file: String,
    pub entity_map: HashMap<u32, entities::Entity>,
    pub storage: HashMap<ComponentKey, Value>,
}

#[inline]
pub fn read_metadata(path: &str) -> Result<std::time::SystemTime, std::io::Error> {
    let metadata = fs::metadata(path)?;
    metadata.modified()
}

#[inline]
pub fn read_raw_file(path: &str) -> Result<String, error::FileError> {
    Ok(std::fs::read_to_string(path)?)
}

#[inline]
pub fn parse_file(file: &str) -> Result<Vec<LoadableComponent>, error::FileError> {
    Ok(serde_yaml::from_str(&file).map_err(error::FileError::from)?)
}

#[inline]
pub fn load<const N: usize>(
    path: &str,
    world: &mut World<N>,
    entity_manager: &mut entities::EntityManager,
    match_user_upsert_option: Option<
        fn(&mut World<N>, entities::Entity, &str, Value) -> Result<(), error::EngineError>,
    >,
) -> Result<MapCache, error::EngineError> {
    let metadata = read_metadata(path).map_err(error::FileError::from)?;
    let raw_file = read_raw_file(path)?;
    let parsed_file = parse_file(&raw_file)?;
    let parsed_file_len = parsed_file.len();
    let mut entity_map: HashMap<FileEntity, entities::Entity> = HashMap::with_capacity(parsed_file_len / 6);
    let mut storage: HashMap<ComponentKey, Value> = HashMap::with_capacity(parsed_file_len);

    match match_user_upsert_option {
        Some(match_user_upsert) => {
            for component in parsed_file {
                let entity = *entity_map
                    .entry(component.entity)
                    .or_insert_with(|| entity_manager.create());

                let kind = component.kind.as_str();

                match_engine_upsert(world, entity, kind, component.data.clone())?;
                match_user_upsert(world, entity, kind, component.data.clone())?;
                storage.insert((component.entity, component.kind), component.data);
            }
        }
        None => {
            for component in parsed_file {
                let entity = *entity_map
                    .entry(component.entity)
                    .or_insert_with(|| entity_manager.create());

                match_engine_upsert(world, entity, component.kind.as_str(), component.data.clone())?;
                storage.insert((component.entity, component.kind), component.data);
            }
        }
    };

    Ok(MapCache {
        path: String::from(path),
        metadata: metadata,
        raw_file: raw_file,
        entity_map: entity_map,
        storage: storage,
    })
}

#[inline]
pub fn hot_reload<const N: usize>(
    cache: &mut MapCache,
    world: &mut World<N>,
    entity_manager: &mut entities::EntityManager,
    match_user_upsert_option: Option<
        fn(&mut World<N>, entities::Entity, &str, Value) -> Result<(), error::EngineError>,
    >,
    match_user_remove_option: Option<fn(&mut World<N>, entities::Entity, &str)>,
) -> Result<(), error::EngineError> {
    let path = cache.path.as_str();
    let new_metadata = read_metadata(&cache.path).map_err(error::FileError::from)?;

    if new_metadata == cache.metadata {
        // file hasn't changed
        return Ok(());
    }

    cache.metadata = new_metadata;

    let new_raw_file = read_raw_file(path)?;

    if new_raw_file == cache.raw_file {
        // file hasn't changed
        return Ok(());
    }

    let new_parsed_file = parse_file(&new_raw_file)?;
    let mut new_storage: HashMap<ComponentKey, Value> = HashMap::with_capacity(new_parsed_file.len());

    for component in new_parsed_file {
        new_storage.insert((component.entity, component.kind), component.data);
    }

    match match_user_upsert_option {
        Some(match_user_upsert) => {
            for ((new_entity, new_kind), new_value) in &new_storage {
                match cache.storage.get_mut(&(*new_entity, new_kind.clone())) {
                    Some(value) if value != new_value => {
                        // value was modified
                        let entity = *cache
                            .entity_map
                            .entry(*new_entity)
                            .or_insert_with(|| entity_manager.create());

                        let kind = new_kind.as_str();

                        match_engine_upsert(world, entity, kind, new_value.clone())?; // update world
                        match_user_upsert(world, entity, kind, new_value.clone())?;
                        *value = new_value.clone(); // update cache
                    }
                    Some(_) => (), // value didn't change
                    None => {
                        // value was created
                        let entity = *cache
                            .entity_map
                            .entry(*new_entity)
                            .or_insert_with(|| entity_manager.create());

                        let kind = new_kind.as_str();

                        match_engine_upsert(world, entity, kind, new_value.clone())?; // update world
                        match_user_upsert(world, entity, kind, new_value.clone())?;
                        cache.storage.insert((*new_entity, new_kind.clone()), new_value.clone()); // update cache
                    }
                }
            }
        }
        None => {
            for ((new_entity, new_kind), new_value) in &new_storage {
                match cache.storage.get_mut(&(*new_entity, new_kind.clone())) {
                    Some(value) if value != new_value => {
                        // value was modified
                        let entity = *cache
                            .entity_map
                            .entry(*new_entity)
                            .or_insert_with(|| entity_manager.create());

                        let kind = new_kind.as_str();

                        match_engine_upsert(world, entity, kind, new_value.clone())?; // update world
                        *value = new_value.clone(); // update cache
                    }
                    Some(_) => (), // value didn't change
                    None => {
                        // value was created
                        let entity = *cache
                            .entity_map
                            .entry(*new_entity)
                            .or_insert_with(|| entity_manager.create());

                        let kind = new_kind.as_str();

                        match_engine_upsert(world, entity, kind, new_value.clone())?; // update world
                        cache.storage.insert((*new_entity, new_kind.clone()), new_value.clone()); // update cache
                    }
                }
            }
        }
    }

    match match_user_remove_option {
        Some(match_user_remove) => {
            cache.storage.retain(|key, _| {
                if new_storage.contains_key(key) {
                    true
                } else {
                    // value was removed
                    let entity = *cache.entity_map.get(&key.0).expect("missing global entity");
                    let kind = key.1.as_str();

                    match_engine_remove(world, entity, kind);
                    match_user_remove(world, entity, kind);
                    false
                }
            })
        }
        None => {
            cache.storage.retain(|key, _| {
                if new_storage.contains_key(key) {
                    true
                } else {
                    // value was removed
                    let entity = *cache.entity_map.get(&key.0).expect("missing global entity");

                    match_engine_remove(world, entity, key.1.as_str());
                    false
                }
            })
        }
    };

    cache.raw_file = new_raw_file;
    Ok(())
}

fn match_engine_upsert<const N: usize>(
    world: &mut World<N>,
    entity: entities::Entity,
    kind: &str,
    data: Value,
) -> Result<(), error::EngineError> {
    match kind {
        "transform" => {
            let transform_spec = components::TransformSpec::deserialize(data).map_err(error::FileError::from)?;
            world.engine.transform.upsert(entity, transform_spec.into());
            Ok(())
        }
        "rotation_matrix" => {
            let rot_mat_spec = components::RotationMatrixSpec::deserialize(data).map_err(error::FileError::from)?;
            world.engine.rotation_matrix.upsert(entity, rot_mat_spec.to_rot_mat());
            Ok(())
        }
        "translation" => {
            let translation_spec = components::TranslationSpec::deserialize(data).map_err(error::FileError::from)?;
            world.engine.translation.upsert(entity, translation_spec.try_into()?);
            Ok(())
        }
        "rotation" => {
            let rotation_spec = components::RotationSpec::deserialize(data).map_err(error::FileError::from)?;
            world.engine.rotation.upsert(entity, rotation_spec.try_into()?);
            Ok(())
        }
        "surface" => {
            let surface_spec = components::SurfaceSpec::deserialize(data).map_err(error::FileError::from)?;
            world.engine.surface.upsert(entity, surface_spec.into());
            Ok(())
        }
        "body" => {
            let body_spec = components::BodySpec::deserialize(data).map_err(error::FileError::from)?;
            let body: components::Body = body_spec.into();
            body.shape().validate()?;
            world.engine.body.upsert(entity, body);
            Ok(())
        }
        "material" => {
            let material_spec = components::MaterialSpec::deserialize(data).map_err(error::FileError::from)?;
            world.engine.material.upsert(entity, material_spec.into());
            Ok(())
        }
        _ => Ok(()),
    }
}

fn match_engine_remove<const N: usize>(world: &mut World<N>, entity: entities::Entity, kind: &str) {
    match kind {
        "transform" => {
            world.engine.transform.remove(entity);
        }
        "rotation_matrix" => {
            world.engine.rotation_matrix.remove(entity);
        }
        "translation" => {
            world.engine.translation.remove(entity);
        }
        "rotation" => {
            world.engine.rotation.remove(entity);
        }
        "surface" => {
            world.engine.surface.remove(entity);
        }
        "body" => {
            world.engine.body.remove(entity);
        }
        "material" => {
            world.engine.material.remove(entity);
        }
        _ => (),
    }
}
