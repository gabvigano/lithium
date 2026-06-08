use crate::{
    core::{
        collections, error,
        time::{self, TickMethods},
    },
    ecs::{entities, world},
    network::{packets, shared, snapshots},
};

use std::collections::HashMap;
use std::marker::PhantomData;
use std::net::{SocketAddr, UdpSocket};
use std::sync::{Arc, Mutex, mpsc};
use std::thread;

use bincode::Encode;

pub struct Server<S, C, I> {
    pub address: SocketAddr,
    socket: UdpSocket,
    pub clients: Arc<Mutex<Vec<SocketAddr>>>,
    pub clients_to_load: mpsc::Receiver<SocketAddr>,
    pub received_packets: mpsc::Receiver<(SocketAddr, packets::ClientPacket<C, I>)>,
    _marker: PhantomData<S>,
}

impl<S, C, I> Server<S, C, I>
where
    S: bincode::Encode,
    C: bincode::Decode<()> + Send + 'static,
    I: bincode::Encode + bincode::Decode<()> + Send + 'static,
{
    #[inline]
    pub fn start(port: u16, tick: Arc<Mutex<time::Tick>>) -> Result<Self, error::NetworkError> {
        let ip = shared::get_local_ip();
        let address = SocketAddr::new(ip, port);
        let socket = UdpSocket::bind(address)?;
        let address = socket.local_addr()?;

        let (clients_to_load_tx, clients_to_load_rx) = mpsc::channel();
        let (received_packets_tx, received_packets_rx) = mpsc::channel();

        let server = Self {
            address,
            socket: socket.try_clone()?,
            clients: Arc::new(Mutex::new(Vec::new())),
            clients_to_load: clients_to_load_rx,
            received_packets: received_packets_rx,
            _marker: PhantomData,
        };

        let clients_clone = server.clients.clone();
        thread::spawn(move || Self::recv_thread(socket, tick, clients_clone, clients_to_load_tx, received_packets_tx));

        Ok(server)
    }

    #[inline]
    pub fn send_packet(&self, packet: &packets::ServerPacket<S, I>, address: &SocketAddr) -> Result<(), error::NetworkError> {
        let mut buffer = [0u8; packets::MAX_PACKET_SIZE];
        let len = bincode::encode_into_slice(packet, &mut buffer, bincode::config::standard())?;
        self.socket.send_to(&buffer[..len], address)?;
        Ok(())
    }

    #[inline]
    pub fn send_bytes(&self, bytes: &Vec<u8>, address: SocketAddr) -> Result<(), error::NetworkError> {
        self.socket.send_to(bytes, address)?;
        Ok(())
    }

    #[inline]
    fn recv_thread(
        socket: UdpSocket,
        tick: Arc<Mutex<time::Tick>>,
        clients: Arc<Mutex<Vec<SocketAddr>>>,
        clients_to_load: mpsc::Sender<SocketAddr>,
        received_packets: mpsc::Sender<(SocketAddr, packets::ClientPacket<C, I>)>,
    ) {
        let mut buffer = [0u8; 1500];
        let join_accept_bytes = bincode::encode_to_vec(packets::ServerPacket::<S, I>::JoinAccept, bincode::config::standard()).unwrap();

        loop {
            match socket.recv_from(&mut buffer) {
                Ok((len, addr)) => {
                    let (packet, _): (packets::ClientPacket<C, I>, usize) =
                        match bincode::decode_from_slice(&buffer[..len], bincode::config::standard()) {
                            Ok(value) => value,
                            Err(_) => continue,
                        };

                    match packet {
                        packets::ClientPacket::JoinRequest => {
                            match socket.send_to(&join_accept_bytes, addr) {
                                Ok(_) => (),
                                Err(_) => continue,
                            };

                            match clients_to_load.send(addr) {
                                Ok(_) => (),
                                Err(_) => continue,
                            }

                            let mut clients_guard = match clients.lock() {
                                Ok(value) => value,
                                Err(_) => continue,
                            };

                            if !clients_guard.contains(&addr) {
                                clients_guard.push(addr);
                                println!("\n>> new connection from {addr}");
                            }
                        }
                        packets::ClientPacket::Ping(send_time) => {
                            let guard = match tick.lock() {
                                Ok(value) => value,
                                Err(_) => continue,
                            };

                            let ping_bytes = match bincode::encode_to_vec(
                                packets::ServerPacket::<S, I>::Ping { send_time, tick: *guard },
                                bincode::config::standard(),
                            ) {
                                Ok(value) => value,
                                Err(_) => continue,
                            };

                            let _ = socket.send_to(&ping_bytes, addr);
                        }
                        _ => {
                            let _ = received_packets.send((addr, packet));
                        }
                    }
                }
                Err(_) => (),
            }
        }
    }
}

pub struct ServerSession<S, I> {
    pub address_map: HashMap<SocketAddr, entities::Entity>, // maps addresses to their entities
    pub ack_tick_map: HashMap<SocketAddr, time::Tick>,      // maps addresses to their last acknowledged tick
    pub input_map: shared::InputMap<I>,                     // stores all the recorded inputs for each tick
    pub oldest_input: Option<time::Tick>,                   // oldest recorded input of the current tick
    pub last_sent_snapshot: world::World<0>, // cache of the last snapshot sent to clients (todo: use const generic N instead of 0)
    pub world_snapshots: collections::CappedVec<(time::Tick, world::World<0>)>, // stores snapshots of the world
    pub initial_state_packets: Vec<packets::ServerPacket<S, I>>, // buffer to build initial state packets
    pub delta_state_snapshots: Vec<packets::Snapshot>, // buffer to build delta state snapshots
}

impl<S, I: PartialEq> ServerSession<S, I> {
    #[inline]
    pub fn new(max_world_snapshots: usize) -> Self {
        Self {
            address_map: HashMap::new(),
            ack_tick_map: HashMap::new(),
            input_map: shared::InputMap::new(),
            oldest_input: None,
            last_sent_snapshot: world::World::default(),
            world_snapshots: collections::CappedVec::new(max_world_snapshots),
            initial_state_packets: Vec::new(),
            delta_state_snapshots: Vec::new(),
        }
    }

    #[inline]
    pub fn reset_oldest_input(&mut self) {
        self.oldest_input = None;
    }

    #[inline]
    pub fn record_input(&mut self, tick: time::Tick, address: SocketAddr, input: I, input_tick: time::Tick) {
        let entity = self.address_map.get(&address).unwrap();

        // update oldest_input
        self.oldest_input = match self.oldest_input {
            Some(oldest_input) => Some(tick.oldest_between(oldest_input, input_tick)),
            None => Some(input_tick),
        };

        // add input to input_map
        self.input_map.record(input_tick, *entity, input);

        // add input to ack_tick_map
        if let Some(ack_tick) = self.ack_tick_map.get_mut(&address) {
            if input_tick.is_after(*ack_tick) {
                *ack_tick = input_tick;
            }
        } else {
            self.ack_tick_map.insert(address, input_tick);
        }
    }

    #[inline]
    pub fn take_snapshot(&mut self, world: &mut world::World<0>, tick: time::Tick) {
        // todo: fix this temporary workaround to clone world
        let mut snapshot = world::World::default();
        snapshot.engine = world.engine.clone();

        self.world_snapshots.push_back((tick, snapshot));

        if let Some(oldest_snapshot) = self.world_snapshots.first() {
            self.input_map.prune_before_tick(oldest_snapshot.0); // remove inputs older than the oldest snapshot
        }
    }

    #[inline]
    pub fn rewind_and_simulate<A, P>(&mut self, world: &mut world::World<0>, tick: time::Tick, mut apply_input: A, mut compute_physics: P)
    where
        P: FnMut(&mut world::World<0>),
        A: FnMut(&mut world::World<0>, entities::Entity, &I),
    {
        if let Some(oldest_input) = self.oldest_input {
            // an input has been received during this tick, so rewind is necessary
            if oldest_input.is_before_or_equal(tick) {
                // the received input is older than the current frame (this is nearly impossible in real life because of delay, unless server and client ticks are out of sync)
                let number_of_snapshots = self.world_snapshots.data().len();

                for (snapshot_idx, (snapshot_tick, snapshot)) in self.world_snapshots.data().iter().rev().enumerate() {
                    if snapshot_tick.is_before_or_equal(oldest_input) || snapshot_idx == number_of_snapshots - 1 {
                        // this snapshot is old enough or its the oldest one
                        world.engine = snapshot.engine.clone(); // todo: fix this temporary workaround to clone world

                        let mut rewind_tick = *snapshot_tick;

                        while rewind_tick.is_before_or_equal(tick) {
                            // overwrite snapshots that are more recent than the one we rolled back to, because otherwise they wouldn't be valid anymore
                            if let Some((_, outdated_snapshot)) = self.world_snapshots.iter_mut().find(|(t, _)| *t == rewind_tick) {
                                outdated_snapshot.engine = world.engine.clone();
                            }

                            // do not apply input and compute physics for current tick, it will be done right outside the rewind loop
                            if rewind_tick == tick {
                                break;
                            }

                            // apply inputs and physics
                            shared::simulate_tick(world, rewind_tick, &mut self.input_map, &mut apply_input, &mut compute_physics);

                            // update clock
                            rewind_tick.next();
                        }

                        break;
                    }
                }
            }
        }

        // apply inputs and physics
        shared::simulate_tick(world, tick, &mut self.input_map, &mut apply_input, &mut compute_physics);
    }
}

impl<S: Encode, I: Encode> ServerSession<S, I> {
    #[inline]
    pub fn send_initial_state<F>(
        &mut self,
        world: &world::World<0>,
        tick: time::Tick,
        client: &SocketAddr,
        mut send_packet: F,
    ) -> Result<(), error::NetworkError>
    where
        F: FnMut(&packets::ServerPacket<S, I>, &SocketAddr) -> Result<(), error::NetworkError>,
    {
        if self.initial_state_packets.is_empty() {
            snapshots::initial_state_packets::<_, S, I>(world, tick, &mut self.initial_state_packets)?;
        }
        for packet in self.initial_state_packets.iter_mut() {
            send_packet(packet, client)?;
        }
        println!(">> initial state uploaded to newly connected client");

        Ok(())
    }

    #[inline]
    pub fn send_delta_state<F>(
        &mut self,
        world: &world::World<0>,
        tick: time::Tick,
        clients: &Vec<SocketAddr>,
        mut send_packet: F,
    ) -> Result<(), error::NetworkError>
    where
        F: FnMut(&packets::ServerPacket<S, I>, &SocketAddr) -> Result<(), error::NetworkError>,
    {
        snapshots::delta_state_snapshots::<_, S, I>(world, &self.last_sent_snapshot, tick, &mut self.delta_state_snapshots)?;
        self.last_sent_snapshot.engine = world.engine.clone(); // todo: fix temporary workaround

        // send packets to clients
        for snapshot in self.delta_state_snapshots.iter() {
            for client in clients.iter() {
                send_packet(
                    &packets::ServerPacket::DeltaState {
                        snapshot: snapshot.clone(),
                        ack_tick: *self.ack_tick_map.get(client).unwrap_or(&tick), // if there is no ack_tick available, default to the current tick
                    },
                    client,
                )?;
            }
        }

        Ok(())
    }
}
