use crate::ecs;

use std::{error, fmt, io};

#[derive(Debug)]
pub enum EngineError {
    FileError(FileError),
    ComponentError(ComponentError),
    MathError(MathError),
    GeometryError(GeometryError),
    NetworkError(NetworkError),
}

impl error::Error for EngineError {}

impl fmt::Display for EngineError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::FileError(e) => write!(f, "{e}"),
            Self::ComponentError(e) => write!(f, "{e}"),
            Self::MathError(e) => write!(f, "{e}"),
            Self::GeometryError(e) => write!(f, "{e}"),
            Self::NetworkError(e) => write!(f, "{e}"),
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

impl From<NetworkError> for EngineError {
    fn from(e: NetworkError) -> Self {
        Self::NetworkError(e)
    }
}

#[derive(Debug)]
pub enum FileError {
    Load(io::Error),
    Parse(serde_yaml::Error),
}

impl error::Error for FileError {}

impl fmt::Display for FileError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FileError::Load(e) => write!(f, "error during file loading: {e}"),
            FileError::Parse(e) => write!(f, "error during parsing: {e}"),
        }
    }
}

impl From<io::Error> for FileError {
    fn from(e: io::Error) -> Self {
        Self::Load(e)
    }
}

impl From<serde_yaml::Error> for FileError {
    fn from(e: serde_yaml::Error) -> Self {
        Self::Parse(e)
    }
}

#[derive(Debug)]
pub enum ComponentError {
    ComponentOutOfRange(usize),
    MismatchingComponent(),
    ComponentNotFound(ecs::Entity),
    DuplicateComponent(ecs::Entity),
}

impl error::Error for ComponentError {}

impl fmt::Display for ComponentError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ComponentError::ComponentOutOfRange(index) => {
                write!(f, "no component exists for index {index}")
            }
            ComponentError::MismatchingComponent() => {
                write!(f, "tried to downcast to a mismatching type")
            }
            ComponentError::ComponentNotFound(entity) => {
                write!(f, "component not found for entity {entity}")
            }
            ComponentError::DuplicateComponent(entity) => {
                write!(f, "component already defined for entity {entity}")
            }
        }
    }
}

#[derive(Debug)]
pub enum MathError {
    NonPositive(&'static str),
}

impl error::Error for MathError {}

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
    NormalizationError,
}

impl error::Error for GeometryError {}

impl fmt::Display for GeometryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GeometryError::TooFewVertices(verts) => write!(f, "cannot build this shape with only {verts} vertices"),
            GeometryError::DuplicateVertices => write!(f, "shape has overlapping or duplicate vertices"),
            GeometryError::NotConvex => write!(f, "shape must be convex"),
            GeometryError::NormalizationError => write!(f, "number of vertices changed during normalization"),
        }
    }
}

#[derive(Debug)]
pub enum NetworkError {
    AddrNotAvailable,
    NetworkUnreachable,
    PermissionDenied,
    AddressInUse,
    SerializationError(bincode::error::EncodeError),
    Io(io::Error),
}

impl error::Error for NetworkError {}

impl fmt::Display for NetworkError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NetworkError::AddrNotAvailable => write!(f, "address is not available"),
            NetworkError::NetworkUnreachable => write!(f, "the network is unreachable"),
            NetworkError::PermissionDenied => write!(f, "permission denied"),
            NetworkError::AddressInUse => write!(f, "address is already in use"),
            NetworkError::SerializationError(e) => write!(f, "serialization error: {e}"),
            NetworkError::Io(e) => write!(f, "{e}"),
        }
    }
}

impl From<io::Error> for NetworkError {
    fn from(e: io::Error) -> Self {
        match e.kind() {
            io::ErrorKind::AddrNotAvailable => Self::AddrNotAvailable,
            io::ErrorKind::NetworkUnreachable | io::ErrorKind::HostUnreachable => Self::NetworkUnreachable,
            io::ErrorKind::PermissionDenied => Self::PermissionDenied,
            io::ErrorKind::AddrInUse => Self::AddressInUse,
            _ => Self::Io(e),
        }
    }
}

impl From<bincode::error::EncodeError> for NetworkError {
    fn from(e: bincode::error::EncodeError) -> Self {
        Self::SerializationError(e)
    }
}
