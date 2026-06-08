pub mod core;
pub mod ecs;
pub mod math;
pub mod network;
pub mod renderer;

pub mod prelude {
    pub use crate::core::collections::*;
    pub use crate::core::debug::*;
    pub use crate::core::error::*;
    pub use crate::core::loader::*;
    pub use crate::core::time::*;

    pub use crate::ecs::components::*;
    pub use crate::ecs::entities::*;
    pub use crate::ecs::storage::*;
    pub use crate::ecs::systems::collisions::*;
    pub use crate::ecs::systems::dynamics::*;
    pub use crate::ecs::world::*;

    pub use crate::math::algebra::*;
    pub use crate::math::geometry::*;
    pub use crate::math::renderer::*;

    pub use crate::network::client::*;
    pub use crate::network::packets::*;
    pub use crate::network::server::*;
    pub use crate::network::shared::*;
    pub use crate::network::snapshots::*;

    pub use crate::renderer::mq_adapter::*;
    pub use crate::renderer::scene::*;
}
