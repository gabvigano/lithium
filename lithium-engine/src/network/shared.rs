use crate::base::time::TickMethods;
use crate::{base, ecs};

use std::collections::BTreeMap;
use std::net::{IpAddr, Ipv4Addr, SocketAddr, UdpSocket};

use bincode::{Decode, Encode};

pub fn get_local_ip() -> IpAddr {
    let fallback = IpAddr::V4(Ipv4Addr::LOCALHOST);

    let socket = match UdpSocket::bind("0.0.0.0:0") {
        Ok(socket) => socket,
        Err(_) => return fallback,
    };

    if socket.connect("8.8.8.8:80").is_err() {
        return fallback;
    }

    match socket.local_addr() {
        Ok(SocketAddr::V4(addr)) => IpAddr::V4(*addr.ip()),
        Ok(SocketAddr::V6(addr)) => IpAddr::V6(*addr.ip()),
        Err(_) => fallback,
    }
}

// pub fn get_local_ip() -> Result<std::net::IpAddr, error::NetworkError> {
//     let socket = UdpSocket::bind("0.0.0.0:0").map_err(error::NetworkError::from)?;
//     socket.connect("8.8.8.8:80").map_err(error::NetworkError::from)?;
//     Ok(socket.local_addr()?.ip())
// }

#[inline]
pub(crate) fn simulate_tick<I: PartialEq, A, P>(
    world: &mut ecs::World<0>,
    tick: base::Tick,
    input_map: &mut InputMap<I>,
    apply_input: &mut A,
    compute_physics: &mut P,
) where
    A: FnMut(&mut ecs::World<0>, ecs::Entity, &I),
    P: FnMut(&mut ecs::World<0>),
{
    if let Some(inputs) = input_map.get_tick(tick) {
        for (entity, input) in inputs {
            apply_input(world, *entity, input);
        }
    }

    compute_physics(world);
}

#[derive(Debug, Clone, Encode, Decode)]
pub struct InputMap<I> {
    inputs: BTreeMap<base::Tick, Vec<(ecs::Entity, I)>>,
}

impl<I: PartialEq> InputMap<I> {
    #[inline]
    pub fn new() -> Self {
        Self { inputs: BTreeMap::new() }
    }

    #[inline]
    pub fn get_tick(&self, tick: base::Tick) -> Option<&Vec<(ecs::Entity, I)>> {
        self.inputs.get(&tick)
    }

    #[inline]
    pub fn get_mut_tick(&mut self, tick: base::Tick) -> Option<&mut Vec<(ecs::Entity, I)>> {
        self.inputs.get_mut(&tick)
    }

    #[inline]
    pub fn prune_before_tick(&mut self, tick: base::Tick) {
        self.inputs.retain(|input_tick, _| input_tick.is_after_or_equal(tick));
    }

    #[inline]
    pub fn record(&mut self, tick: base::Tick, entity: ecs::Entity, input: I) {
        if let Some(inputs) = self.inputs.get_mut(&tick) {
            let input = (entity, input);
            if !inputs.contains(&input) {
                inputs.push(input);
            }
        } else {
            self.inputs.insert(tick, vec![(entity, input)]);
        }
    }

    #[inline]
    pub fn record_many(&mut self, tick: base::Tick, inputs_vec: Vec<(ecs::Entity, I)>) {
        if let Some(inputs) = self.inputs.get_mut(&tick) {
            for input in inputs_vec {
                if !inputs.contains(&input) {
                    inputs.push(input);
                }
            }
        } else {
            self.inputs.insert(tick, inputs_vec);
        }
    }

    #[inline]
    pub fn merge(&mut self, input_map: Self) {
        for (tick, inputs_vec) in input_map.inputs {
            self.record_many(tick, inputs_vec)
        }
    }
}
