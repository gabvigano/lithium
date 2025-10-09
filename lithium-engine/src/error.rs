use crate::ecs::entities;

use std::fmt;

#[derive(Debug)]
pub enum EngineError {
    FileError(FileError),
    ComponentError(ComponentError),
    MathError(MathError),
    GeometryError(GeometryError),
}

impl std::error::Error for EngineError {}

impl std::fmt::Display for EngineError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::FileError(e) => write!(f, "{e}"),
            Self::ComponentError(e) => write!(f, "{e}"),
            Self::MathError(e) => write!(f, "{e}"),
            Self::GeometryError(e) => write!(f, "{e}"),
        }
    }
}

impl From<FileError> for EngineError {
    fn from(e: FileError) -> Self {
        Self::FileError(e)
    }
}

impl From<ComponentError> for EngineError {
    fn from(e: ComponentError) -> Self {
        Self::ComponentError(e)
    }
}

impl From<MathError> for EngineError {
    fn from(e: MathError) -> Self {
        Self::MathError(e)
    }
}

impl From<GeometryError> for EngineError {
    fn from(e: GeometryError) -> Self {
        Self::GeometryError(e)
    }
}

#[derive(Debug)]
pub enum FileError {
    Load(std::io::Error),
    Parse(ron::error::SpannedError),
}

impl std::error::Error for FileError {}

impl fmt::Display for FileError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FileError::Load(e) => write!(f, "error during file loading: {e}"),
            FileError::Parse(e) => write!(f, "error during parsing: {e}"),
        }
    }
}

impl From<std::io::Error> for FileError {
    fn from(e: std::io::Error) -> Self {
        Self::Load(e)
    }
}

impl From<ron::error::SpannedError> for FileError {
    fn from(e: ron::error::SpannedError) -> Self {
        Self::Parse(e)
    }
}

#[derive(Debug)]
pub enum ComponentError {
    MissingComponent(entities::Entity),
    AlreadyExistingComponent(entities::Entity),
}

impl std::error::Error for ComponentError {}

impl fmt::Display for ComponentError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ComponentError::MissingComponent(entity) => {
                write!(f, "component not found for entity {entity}")
            }
            ComponentError::AlreadyExistingComponent(entity) => {
                write!(f, "component already defined for entity {entity}")
            }
        }
    }
}

#[derive(Debug)]
pub enum MathError {
    NonPositive(&'static str),
}

impl std::error::Error for MathError {}

impl fmt::Display for MathError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MathError::NonPositive(param) => write!(f, "{param} must be positive"),
        }
    }
}

#[derive(Debug)]
pub enum GeometryError {
    TooFewVertices(usize),
    DuplicateVertices,
    NotConvex,
}

impl std::error::Error for GeometryError {}

impl fmt::Display for GeometryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GeometryError::TooFewVertices(verts) => write!(f, "cannot build this shape with only {verts} vertices"),
            GeometryError::DuplicateVertices => write!(f, "shape has overlapping or duplicate vertices"),
            GeometryError::NotConvex => write!(f, "shape must be convex"),
        }
    }
}
