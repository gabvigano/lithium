use crate::{
    core::{error, time},
    ecs::world,
    network::packets,
};

use bincode::Encode;

// macro_rules! push_actions {
//     (
//         $packets_to_send:expr,
//         $tick:expr,
//         $action:expr,
//         $packet_variant:ident
//     ) => {{
//         if let Some(packets::ServerPacket::$packet_variant(last_snapshot)) = $packets_to_send.last_mut() {
//             last_snapshot.actions.push($action);

//             if Self::encoded_size(&packets::ServerPacket::<S>::$packet_variant(last_snapshot.clone()))? <= packets::MAX_PACKET_SIZE {
//                 Ok::<(), error::NetworkError>(())
//             } else {
//                 let last_action = last_snapshot.actions.pop().unwrap();

//                 let new_packet = packets::ServerPacket::$packet_variant(packets::Snapshot {
//                     tick: $tick,
//                     packet_id: $packets_to_send.len() as u16,
//                     actions: vec![last_action],
//                 });

//                 $packets_to_send.push(new_packet);
//                 Ok(())
//             }
//         } else {
//             $packets_to_send.push(packets::ServerPacket::$packet_variant(packets::Snapshot {
//                 tick: $tick,
//                 packet_id: $packets_to_send.len() as u16,
//                 actions: vec![$action],
//             }));

//             Ok(())
//         }
//     }};
// }

macro_rules! load_initial_state {
    (
        $world:expr,
        $packets:expr,
        $S:ty,
        $I:ty,
        $tick:expr,
        $component_field:ident,
        $component_variant:ident
    ) => {{
        for (entity, component) in $world.engine.$component_field.iter() {
            push_actions_initial_state::<$S, $I>(
                $packets,
                $tick,
                packets::NetworkAction {
                    always_apply: false,
                    command: packets::NetworkCommand::Set {
                        entity,
                        component: packets::DataNetworkComponent::$component_variant(component.clone()),
                    },
                },
            )?;
        }
    }};
}

macro_rules! load_delta_state {
    (
        $world:expr,
        $world_cache:expr,
        $snapshots:expr,
        $S:ty,
        $I:ty,
        $tick:expr,
        $component_field:ident,
        $component_variant:ident
    ) => {{
        for (entity, component) in $world.engine.$component_field.iter() {
            if $world_cache.engine.$component_field.get(entity) != Some(component) {
                // component doesn't exist in world_cache or it exists but its different (it has changed)
                push_actions_delta_state::<$S, $I>(
                    $snapshots,
                    $tick,
                    packets::NetworkAction {
                        always_apply: false,
                        command: packets::NetworkCommand::Set {
                            entity,
                            component: packets::DataNetworkComponent::$component_variant(component.clone()),
                        },
                    },
                )?;
            }
        }

        for (entity, _) in $world_cache.engine.$component_field.iter() {
            if $world.engine.$component_field.get(entity).is_none() {
                // component exists in world_cache but not in world
                push_actions_delta_state::<$S, $I>(
                    $snapshots,
                    $tick,
                    packets::NetworkAction {
                        always_apply: false,
                        command: packets::NetworkCommand::Remove {
                            entity,
                            component: packets::EmptyNetworkComponent::$component_variant,
                        },
                    },
                )?;
            }
        }
    }};
}

// #[inline]
// fn push_actions(
//     packets_to_send: &mut Vec<packets::ServerPacket<S>>,
//     clock: time::Clock,
//     action: packets::NetworkAction,
// ) -> Result<(), error::NetworkError> {
//     if let Some(packets::ServerPacket::Snapshot(last_snapshot)) = packets_to_send.last_mut() {
//         last_snapshot.actions.push(action);

//         if Self::encoded_size(&packets::ServerPacket::<S>::Snapshot(last_snapshot.clone()))? <= packets::MAX_PACKET_SIZE {
//             return Ok(());
//         }

//         let last_action = last_snapshot.actions.pop().unwrap();

//         let new_packet = packets::ServerPacket::Snapshot(packets::SnapshotPacket {
//             clock,
//             packet_id: packets_to_send.len() as u16,
//             actions: vec![last_action],
//         });

//         packets_to_send.push(new_packet);
//         return Ok(());
//     }

//     packets_to_send.push(packets::ServerPacket::Snapshot(packets::SnapshotPacket {
//         clock,
//         packet_id: 0,
//         actions: vec![action],
//     }));

//     Ok(())
// }

#[inline]
pub fn push_actions_initial_state<S: Encode, I: Encode>(
    packets_to_send: &mut Vec<packets::ServerPacket<S, I>>,
    tick: time::Tick,
    action: packets::NetworkAction,
) -> Result<(), error::NetworkError> {
    if let Some(packets::ServerPacket::InitialState(last_snapshot)) = packets_to_send.last_mut() {
        last_snapshot.actions.push(action);

        if packets::ServerPacket::<S, I>::InitialState(last_snapshot.clone()).size()? <= packets::MAX_PACKET_SIZE {
            return Ok::<(), error::NetworkError>(());
        }

        let last_action = last_snapshot.actions.pop().unwrap();

        let new_packet = packets::ServerPacket::InitialState(packets::Snapshot {
            tick: tick,
            packet_id: packets_to_send.len() as u16,
            actions: vec![last_action],
        });

        packets_to_send.push(new_packet);
        return Ok(());
    }

    packets_to_send.push(packets::ServerPacket::InitialState(packets::Snapshot {
        tick: tick,
        packet_id: packets_to_send.len() as u16,
        actions: vec![action],
    }));

    Ok(())
}

#[inline]
pub fn push_actions_delta_state<S: Encode, I: Encode>(
    snapshots: &mut Vec<packets::Snapshot>,
    tick: time::Tick,
    action: packets::NetworkAction,
) -> Result<(), error::NetworkError> {
    if let Some(last_snapshot) = snapshots.last_mut() {
        last_snapshot.actions.push(action);

        if (packets::ServerPacket::<S, I>::DeltaState {
            snapshot: last_snapshot.clone(),
            ack_tick: 0, // dummy ack_tick to check packet size, since field's size is fixed (u32)
        })
        .size()?
            <= packets::MAX_PACKET_SIZE
        {
            return Ok::<(), error::NetworkError>(());
        }
        let last_action = last_snapshot.actions.pop().unwrap();

        let new_snapshot = packets::Snapshot {
            tick: tick,
            packet_id: snapshots.len() as u16,
            actions: vec![last_action],
        };

        snapshots.push(new_snapshot);
        return Ok(());
    }

    snapshots.push(packets::Snapshot {
        tick: tick,
        packet_id: snapshots.len() as u16,
        actions: vec![action],
    });

    Ok(())
}

#[inline]
pub fn initial_state_packets<const N: usize, S: Encode, I: Encode>(
    world: &world::World<N>,
    tick: time::Tick,
    packets: &mut Vec<packets::ServerPacket<S, I>>,
) -> Result<(), error::NetworkError> {
    load_initial_state!(world, packets, S, I, tick, transform, Transform);
    load_initial_state!(world, packets, S, I, tick, rotation_matrix, RotationMatrix);
    load_initial_state!(world, packets, S, I, tick, translation, Translation);
    load_initial_state!(world, packets, S, I, tick, rotation, Rotation);
    load_initial_state!(world, packets, S, I, tick, surface, Surface);
    load_initial_state!(world, packets, S, I, tick, body, Body);
    load_initial_state!(world, packets, S, I, tick, material, Material);

    Ok(())
}

#[inline]
pub fn delta_state_snapshots<const N: usize, S: Encode, I: Encode>(
    world: &world::World<N>,
    world_cache: &world::World<N>,
    tick: time::Tick,
    snapshots: &mut Vec<packets::Snapshot>,
) -> Result<(), error::NetworkError> {
    load_delta_state!(world, world_cache, snapshots, S, I, tick, transform, Transform);
    load_delta_state!(world, world_cache, snapshots, S, I, tick, rotation_matrix, RotationMatrix);
    load_delta_state!(world, world_cache, snapshots, S, I, tick, translation, Translation);
    load_delta_state!(world, world_cache, snapshots, S, I, tick, rotation, Rotation);
    load_delta_state!(world, world_cache, snapshots, S, I, tick, surface, Surface);
    load_delta_state!(world, world_cache, snapshots, S, I, tick, body, Body);
    load_delta_state!(world, world_cache, snapshots, S, I, tick, material, Material);

    Ok(())
}
