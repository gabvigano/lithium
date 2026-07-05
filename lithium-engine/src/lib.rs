pub mod base;
pub mod ecs;
pub mod math;
pub mod network;
pub mod physics;
pub mod render;

pub mod prelude {
    pub use crate::base::*;
    pub use crate::ecs::*;
    pub use crate::math::*;
    pub use crate::network::*;
    pub use crate::physics::*;
    pub use crate::render::*;
}
