use crate::{
    core::{
        collections, error,
        time::{self, TickMethods},
    },
    ecs::{entities, world},
    network::{packets, shared},
};

use std::marker::PhantomData;
use std::net::{SocketAddr, UdpSocket};
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
    mpsc,
};
use std::thread;
use std::time::{Duration, Instant};

pub struct Client<S, C, I> {
    pub address: SocketAddr,
    pub server_address: SocketAddr,
    socket: UdpSocket,
    pub connected: Arc<AtomicBool>,
    ping_epoch: Instant,
    pings: mpsc::Receiver<(Duration, u64, time::Tick)>,
    ping_history: collections::CappedVec<(Duration, u64, time::Tick)>,
    pub delay: Duration,
    pub received_packets: mpsc::Receiver<packets::ServerPacket<S, I>>,
    _marker: PhantomData<C>,
}

impl<S, C, I> Client<S, C, I>
where
    S: bincode::Decode<()> + Send + 'static,
    C: bincode::Encode,
    I: bincode::Encode + bincode::Decode<()> + Send + 'static,
{
    #[inline]
    pub fn start(port: u16, server_address: SocketAddr) -> Result<Self, error::NetworkError> {
        let ip = shared::get_local_ip();
        let address = SocketAddr::new(ip, port);
        let socket = UdpSocket::bind(address)?;
        let address = socket.local_addr()?;

        let connected = Arc::new(AtomicBool::new(false));
        let ping_epoch = Instant::now();
        let (pings_tx, pings_rx) = mpsc::channel();
        let (received_packets_tx, received_packets_rx) = mpsc::channel();

        let client = Self {
            address,
            server_address,
            socket: socket.try_clone()?,
            connected: connected.clone(),
            ping_epoch,
            pings: pings_rx,
            ping_history: collections::CappedVec::new(20),
            delay: Duration::from_millis(50),
            received_packets: received_packets_rx,
            _marker: PhantomData,
        };

        thread::spawn(move || Self::recv_thread(server_address, socket, connected, ping_epoch, pings_tx, received_packets_tx));

        client.connect()?;

        Ok(client)
    }

    #[inline]
    pub fn connect(&self) -> Result<(), error::NetworkError> {
        println!("\n>> connecting to server...");
        self.socket.connect(self.server_address)?;
        let join_bytes = bincode::encode_to_vec(packets::ClientPacket::<C, I>::JoinRequest, bincode::config::standard())?;
        self.socket.send(&join_bytes)?;
        Ok(())
    }

    #[inline]
    pub fn send_packet(&self, packet: &packets::ClientPacket<C, I>) -> Result<(), error::NetworkError> {
        let mut buffer = [0u8; packets::MAX_PACKET_SIZE];
        let len = bincode::encode_into_slice(packet, &mut buffer, bincode::config::standard())?;
        self.socket.send(&buffer[..len])?;
        Ok(())
    }

    #[inline]
    pub fn send_bytes(&self, bytes: &Vec<u8>) -> Result<(), error::NetworkError> {
        self.socket.send(bytes)?;
        Ok(())
    }

    #[inline]
    fn recv_thread(
        server_address: SocketAddr,
        socket: UdpSocket,
        connected: Arc<AtomicBool>,
        ping_epoch: Instant,
        pings: mpsc::Sender<(Duration, u64, time::Tick)>,
        received_packets: mpsc::Sender<packets::ServerPacket<S, I>>,
    ) {
        let mut buffer = [0u8; 1500];

        loop {
            match socket.recv_from(&mut buffer) {
                Ok((len, addr)) => {
                    if addr != server_address {
                        continue;
                    }

                    let (packet, _): (packets::ServerPacket<S, I>, usize) =
                        match bincode::decode_from_slice(&buffer[..len], bincode::config::standard()) {
                            Ok(value) => value,
                            Err(_) => continue,
                        };

                    match packet {
                        packets::ServerPacket::JoinAccept => {
                            connected.store(true, Ordering::Relaxed);
                            println!(">> connection established")
                        }
                        packets::ServerPacket::Ping { send_time, tick } => {
                            let recv_time = ping_epoch.elapsed().as_nanos() as u64;

                            let elapsed = recv_time.wrapping_sub(send_time);

                            let _ = pings.send((Duration::from_nanos(elapsed), recv_time, tick));
                        }
                        _ => {
                            let _ = received_packets.send(packet);
                        }
                    }
                }
                Err(_) => (),
            }
        }
    }

    #[inline]
    pub fn ping_server(&mut self, tick: time::Tick) -> Result<(), error::NetworkError> {
        // send pings
        if (tick <= 60 && tick % 6 == 0) || tick % 15 == 0 {
            // during first second: every 100 ms
            // the rest of the time: every 250 ms
            self.send_packet(&packets::ClientPacket::Ping(self.ping_epoch.elapsed().as_nanos() as u64))?;
        }

        // add old pings to history
        while let Ok(ping) = self.pings.try_recv() {
            self.ping_history.push_back(ping);
        }

        // compute delay
        let mut best_pings = self.ping_history.data().clone();

        let pings_number = best_pings.len();

        if pings_number == 0 {
            return Ok(());
        }

        let best_pings = {
            let slice = best_pings.make_contiguous();
            slice.sort_unstable_by_key(|(ping_time, _, _)| *ping_time);
            &slice[..pings_number.min(5)]
        };

        let mut sum = Duration::from_millis(0);
        for &ping in best_pings {
            sum += ping.0
        }

        let average = sum / pings_number.min(5) as u32;

        self.delay = average / 2;

        Ok(())
    }

    #[inline]
    pub fn sync_tick(&self, tick: &mut time::Tick, tick_time: Duration) -> Option<()> {
        let (_, recv_time, server_tick) = self.ping_history.data().iter().max_by_key(|(_, recv_time, _)| *recv_time)?;

        let now = self.ping_epoch.elapsed().as_nanos() as u64;
        let elapsed_since_recv = Duration::from_nanos(now - recv_time);
        let elapsed_since_send = self.delay + elapsed_since_recv;

        let ticks_since_send = (elapsed_since_send.as_nanos() / tick_time.as_nanos()) as u32;
        let server_tick = server_tick.wrapping_add(ticks_since_send);

        if server_tick.abs_diff(*tick) > 1 {
            // more than one off, resync client's tick
            *tick = server_tick
        }

        Some(())
    }
}

pub struct ClientSession<I: PartialEq> {
    pub assigned_entity: entities::Entity,                     // entity assigned by server for this client
    pub input_map: shared::InputMap<I>,                        // stores all the recorded inputs for each tick
    pub last_sent_tick: time::Tick,                            // tick of last input sent, to check against ack_tick
    pub last_received_snapshot: (time::Tick, world::World<0>), // cache of the last snapshot received from server (todo: use const generic N instead of 0)
    pub last_rewind_snapshot: (time::Tick, world::World<0>), // cache of the last snapshot use for rewind (todo: use const generic N instead of 0)
}

impl<I: PartialEq> ClientSession<I> {
    #[inline]
    pub fn new() -> Self {
        Self {
            assigned_entity: 0,
            input_map: shared::InputMap::new(),
            last_sent_tick: 0,
            last_received_snapshot: (0, world::World::default()),
            last_rewind_snapshot: (0, world::World::default()),
        }
    }

    #[inline]
    pub fn prune_input_map(&mut self) {
        self.input_map.prune_before_tick(self.last_rewind_snapshot.0);
    }

    #[inline]
    pub fn record_input(&mut self, tick: time::Tick, input: I) {
        self.last_sent_tick = tick;
        self.input_map.record(tick, self.assigned_entity, input);
    }

    #[inline]
    pub fn apply_initial_state(&mut self, world: &mut world::World<0>, snapshot: packets::Snapshot) {
        let packets::Snapshot {
            tick: packet_tick,
            packet_id: _,
            actions: packet_actions,
        } = snapshot;

        self.last_received_snapshot.0 = packet_tick;
        for action in packet_actions {
            action.apply(&mut self.last_received_snapshot.1).unwrap()
        }

        self.last_rewind_snapshot.0 = self.last_received_snapshot.0;
        self.last_rewind_snapshot.1.engine = self.last_received_snapshot.1.engine().clone();

        world.engine = self.last_received_snapshot.1.engine().clone();

        println!(">> initial state downloaded from server");
    }

    #[inline]
    pub fn apply_delta_state<A, P>(
        &mut self,
        world: &mut world::World<0>,
        tick: time::Tick,
        snapshot: packets::Snapshot,
        ack_tick: time::Tick,
        mut apply_input: A,
        mut compute_physics: P,
    ) where
        P: FnMut(&mut world::World<0>),
        A: FnMut(&mut world::World<0>, entities::Entity, &I),
    {
        let packets::Snapshot {
            tick: packet_tick,
            packet_id: _,
            actions: packet_actions,
        } = snapshot;

        if packet_tick.is_before(self.last_received_snapshot.0) {
            return;
        }

        let mut has_always_apply = false;

        // apply changes to last received snapshot
        self.last_received_snapshot.0 = packet_tick;
        for action in &packet_actions {
            action.clone().apply(&mut self.last_received_snapshot.1).unwrap();
            has_always_apply |= action.always_apply;
        }

        if ack_tick.is_after_or_equal(self.last_sent_tick) {
            // complete rewind
            self.last_rewind_snapshot.0 = self.last_received_snapshot.0;
            self.last_rewind_snapshot.1.engine = self.last_received_snapshot.1.engine().clone();
        } else if has_always_apply {
            // partial rewind
            for action in packet_actions {
                if action.always_apply {
                    action.apply(&mut self.last_rewind_snapshot.1).unwrap();
                }
            }
        } else {
            // skip rewind
            return;
        }

        // rewind
        let mut rewind_tick = self.last_rewind_snapshot.0;
        world.engine = self.last_rewind_snapshot.1.engine().clone();

        while rewind_tick.is_before(tick) {
            shared::simulate_tick(world, rewind_tick, &mut self.input_map, &mut apply_input, &mut compute_physics);
            rewind_tick.next();
        }
    }
}
