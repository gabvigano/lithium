pub mod base;
pub mod ecs;
pub mod math;
pub mod physics;

#[cfg(feature = "render")]
pub mod render;

#[cfg(feature = "network")]
pub mod network;

pub mod prelude {
    pub use crate::base::*;
    pub use crate::ecs::*;
    pub use crate::math::*;
    pub use crate::physics::*;

    #[cfg(feature = "render")]
    pub use crate::render::*;

    #[cfg(feature = "network")]
    pub use crate::network::*;
}
