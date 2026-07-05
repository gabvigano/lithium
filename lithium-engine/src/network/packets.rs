use crate::{base, ecs, network};

use bincode::{Decode, Encode};

pub const MAX_PACKET_SIZE: usize = 1200;

#[derive(Debug, Encode, Decode)]
pub enum ServerPacket<S, I> {
    JoinAccept,
    Ping { send_time: u64, tick: base::Tick },
    InitialState(Snapshot),
    DeltaState { snapshot: Snapshot, ack_tick: base::Tick },
    InputState(network::InputMap<I>),
    User(S),
}

impl<S: Encode, I: Encode> ServerPacket<S, I> {
    #[inline]
    pub fn size(&self) -> Result<usize, base::NetworkError> {
        let mut buffer = [0u8; MAX_PACKET_SIZE * 2];

        Ok(bincode::encode_into_slice(self, &mut buffer, bincode::config::standard())?)
    }
}

#[derive(Debug, Encode, Decode)]
pub enum ClientPacket<C, I> {
    JoinRequest,
    Ping(u64),
    Input { input: I, tick: base::Tick },
    User(C),
}

impl<C: Encode, I: Encode> ClientPacket<C, I> {
    #[inline]
    pub fn size(&self) -> Result<usize, base::NetworkError> {
        let mut buffer = [0u8; MAX_PACKET_SIZE * 2];

        Ok(bincode::encode_into_slice(self, &mut buffer, bincode::config::standard())?)
    }
}

#[derive(Debug, Clone, Encode, Decode)]
pub struct Snapshot {
    pub tick: base::Tick,
    pub packet_id: u16,
    pub actions: Vec<NetworkAction>,
}

#[derive(Debug, Clone, Encode, Decode)]
pub struct NetworkAction {
    pub always_apply: bool,
    pub command: NetworkCommand,
}

impl NetworkAction {
    pub fn apply<const N: usize>(self, world: &mut ecs::World<N>) -> Result<(), base::ComponentError> {
        self.command.apply(world)
    }
}

#[derive(Debug, Clone, Encode, Decode)]
pub enum NetworkCommand {
    Set {
        entity: ecs::Entity,
        component: DataNetworkComponent,
    },
    Remove {
        entity: ecs::Entity,
        component: EmptyNetworkComponent,
    },
}

impl NetworkCommand {
    pub fn apply<const N: usize>(self, world: &mut ecs::World<N>) -> Result<(), base::ComponentError> {
        match self {
            Self::Set { entity, component } => {
                component.set(world, entity);
                Ok(())
            }
            Self::Remove { entity, component } => {
                component.remove(world, entity);
                Ok(())
            }
        }
    }
}

#[derive(Debug, Clone, Encode, Decode)]
pub enum EmptyNetworkComponent {
    Transform,
    RotationMatrix,
    Translation,
    Rotation,
    Surface,
    Body,
    Material,
}

impl EmptyNetworkComponent {
    fn remove<const N: usize>(&self, world: &mut ecs::World<N>, entity: ecs::Entity) {
        match self {
            Self::Transform => _ = world.engine.transform.remove(entity),
            Self::RotationMatrix => _ = world.engine.rotation_matrix.remove(entity),
            Self::Translation => _ = world.engine.translation.remove(entity),
            Self::Rotation => _ = world.engine.rotation.remove(entity),
            Self::Surface => _ = world.engine.surface.remove(entity),
            Self::Body => _ = world.engine.body.remove(entity),
            Self::Material => _ = world.engine.material.remove(entity),
        }
    }
}

#[derive(Debug, Clone, Encode, Decode)]
pub enum DataNetworkComponent {
    Transform(ecs::Transform),
    RotationMatrix(ecs::RotationMatrix),
    Translation(ecs::Translation),
    Rotation(ecs::Rotation),
    Surface(ecs::Surface),
    Body(ecs::Body),
    Material(ecs::Material),
}

impl DataNetworkComponent {
    fn set<const N: usize>(self, world: &mut ecs::World<N>, entity: ecs::Entity) {
        match self {
            Self::Transform(component) => world.engine.transform.upsert(entity, component),
            Self::RotationMatrix(component) => world.engine.rotation_matrix.upsert(entity, component),
            Self::Translation(component) => world.engine.translation.upsert(entity, component),
            Self::Rotation(component) => world.engine.rotation.upsert(entity, component),
            Self::Surface(component) => world.engine.surface.upsert(entity, component),
            Self::Body(component) => world.engine.body.upsert(entity, component),
            Self::Material(component) => world.engine.material.upsert(entity, component),
        }
    }
}
