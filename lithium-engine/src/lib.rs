pub mod core;
pub mod ecs;
pub mod math;
pub mod renderer;

pub mod prelude {
    pub use crate::core::debug::*;
    pub use crate::core::error::*;
    pub use crate::core::loader::*;

    pub use crate::ecs::components::*;
    pub use crate::ecs::entities::*;
    pub use crate::ecs::storage::*;
    pub use crate::ecs::systems::collisions::*;
    pub use crate::ecs::systems::dynamics::*;
    pub use crate::ecs::world::*;

    pub use crate::math::*;

    pub use crate::renderer::mq_adapter::*;
    pub use crate::renderer::scene::*;
}
