use crate::{
    core::error,
    ecs::{components, entities, world::World},
    math::{self, geometry::Validate},
};

use serde::Deserialize;
use serde_yaml::Value;
use std::collections::HashSet;

#[derive(Deserialize, Clone, Debug)]
pub struct LoadableComponent {
    pub entity: u32,
    pub kind: String,
    pub data: Value,
}

fn match_engine<const N: usize>(world: &mut World<N>, comp: LoadableComponent) -> Result<(), error::EngineError> {
    match comp.kind.as_str() {
        "transform" => {
            let transform_spec = components::TransformSpec::deserialize(comp.data).map_err(error::FileError::from)?;
            world.engine.transform.insert(comp.entity, transform_spec.into())?;
            Ok(())
        }
        "rotation_matrix" => {
            let rot = world
                .engine
                .transform
                .get(comp.entity)
                .ok_or(error::ComponentError::ComponentNotFound(comp.entity))?
                .rot;
            let rot_mat_spec =
                components::RotationMatrixSpec::deserialize(comp.data).map_err(error::FileError::from)?;
            world
                .engine
                .rotation_matrix
                .insert(comp.entity, rot_mat_spec.to_rot_mat(rot))?;
            Ok(())
        }
        "translation" => {
            let translation_spec =
                components::TranslationSpec::deserialize(comp.data).map_err(error::FileError::from)?;
            world
                .engine
                .translation
                .insert(comp.entity, translation_spec.try_into()?)?;
            Ok(())
        }
        "rotation" => {
            let rotation_spec = components::RotationSpec::deserialize(comp.data).map_err(error::FileError::from)?;
            world.engine.rotation.insert(comp.entity, rotation_spec.try_into()?)?;
            Ok(())
        }
        "surface" => {
            let surface_spec = components::SurfaceSpec::deserialize(comp.data).map_err(error::FileError::from)?;
            world.engine.surface.insert(comp.entity, surface_spec.into())?;
            Ok(())
        }
        "shape" => {
            let shape = math::Shape::deserialize(comp.data).map_err(error::FileError::from)?;
            shape.validate()?;
            world.engine.shape.insert(comp.entity, shape)?;
            Ok(())
        }
        "material" => {
            let material_spec = components::MaterialSpec::deserialize(comp.data).map_err(error::FileError::from)?;
            world.engine.material.insert(comp.entity, material_spec.into())?;
            Ok(())
        }
        _ => Ok(()),
    }
}

pub fn load<const N: usize>(
    path: &str,
    world: &mut World<N>,
    entity_manager: &mut entities::EntityManager,
    match_user_option: Option<fn(&mut World<N>, LoadableComponent) -> Result<(), error::EngineError>>,
) -> Result<HashSet<entities::Entity>, error::EngineError> {
    let file = std::fs::read_to_string(path).map_err(error::FileError::from)?;
    let comps: Vec<LoadableComponent> = serde_yaml::from_str(&file).map_err(error::FileError::from)?;
    let mut entities = HashSet::with_capacity(comps.len());

    match match_user_option {
        Some(match_user) => {
            for comp in comps {
                entities.insert(comp.entity);
                match_engine(world, comp.clone())?;
                match_user(world, comp)?;
            }
        }
        None => {
            for comp in comps {
                entities.insert(comp.entity);
                match_engine(world, comp)?;
            }
        }
    };

    if let Some(max_entity) = entities.iter().max() {
        entity_manager.skip_to(max_entity + 1);
    }

    Ok(entities)
}
