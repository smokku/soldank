use hecs::{Entity, World};
use laminar::{
    Config as LaminarConfig, ConnectionManager, DatagramSocket, Packet as LaminarPacket,
    SocketEvent, VirtualConnection,
};
use naia_server_socket::{
    find_my_ip_address, LinkConditionerConfig, MessageSender, NaiaServerSocketError,
    Packet as NaiaPacket, ServerSocket, ServerSocketTrait,
};
use smol::channel::{unbounded, Receiver, Sender, TryRecvError};
use std::{
    collections::{HashMap, VecDeque},
    convert::TryFrom,
    io,
    net::SocketAddr,
    time::{Duration, Instant},
};

use crate::{cheat::Cheats, constants::*, cvars::Config, state::build_state_message, systems};
use orb::server::Server;
use soldank_shared::{
    constants::SERVER_PORT,
    messages::{self, encode_message, NetworkMessage},
    networking::{GameWorld, PacketStats},
    trace_dump_packet,
};

pub struct Networking {
    server_socket: Box<dyn ServerSocketTrait>,
    sender: MessageSender,
    packet_sender: Sender<NaiaPacket>,
    payload_receiver: Receiver<NaiaPacket>,
    handler: ConnectionManager<PacketSocket, VirtualConnection>,
    pub connection_key: String,
    pub stats: PacketStats,

    pub connections: HashMap<SocketAddr, Connection>,
}

#[derive(Debug)]
pub struct Connection {
    pub stats: PacketStats,
    pub ack_tick: usize,
    pub last_processed_tick: usize,
    pub last_broadcast: Instant,
    pub authorized: bool,
    pub ready: bool,
    pub nick: String,
    pub cheats: Cheats,
    pub entity: Option<Entity>,
}

impl Connection {
    pub fn new() -> Connection {
        Connection {
            stats: Default::default(),
            ack_tick: 0,
            last_processed_tick: 0,
            last_broadcast: Instant::now(),
            authorized: false,
            ready: false,
            nick: Default::default(),
            cheats: Default::default(),
            entity: None,
        }
    }
}

#[derive(Debug)]
struct PacketSocket {
    bind_address: SocketAddr,
    packet_receiver: Receiver<NaiaPacket>,
    payload_sender: Sender<NaiaPacket>,
}

impl DatagramSocket for PacketSocket {
    fn send_packet(&mut self, addr: &SocketAddr, payload: &[u8]) -> io::Result<usize> {
        match smol::block_on(
            self.payload_sender
                .send(NaiaPacket::new(*addr, payload.to_vec())),
        ) {
            Ok(()) => Ok(payload.len()),
            Err(error) => {
                panic!("Error sending via payload channel: {}", error);
            }
        }
    }

    fn receive_packet<'a>(&mut self, buffer: &'a mut [u8]) -> io::Result<(&'a [u8], SocketAddr)> {
        match self.packet_receiver.try_recv() {
            Ok(packet) => {
                let payload = packet.payload();
                buffer[..payload.len()].clone_from_slice(payload);
                Ok((&buffer[..payload.len()], packet.address()))
            }
            Err(error) => match error {
                TryRecvError::Empty => Err(io::Error::new(io::ErrorKind::WouldBlock, error)),
                TryRecvError::Closed => {
                    panic!("Error receiving from packet channel: {}", error);
                }
            },
        }
    }

    fn local_addr(&self) -> io::Result<SocketAddr> {
        Ok(self.bind_address)
    }

    fn is_blocking_mode(&self) -> bool {
        false
    }
}

impl Networking {
    pub async fn new(bind: Option<&str>) -> Networking {
        let bind_address = if let Some(addr) = bind {
            addr.parse().expect("cannot parse bind address")
        } else {
            let addr = find_my_ip_address().expect("can't find local ip address");
            SocketAddr::new(addr, SERVER_PORT)
        };

        let mut webrtc_listen_address = bind_address;
        webrtc_listen_address.set_port(webrtc_listen_address.port() + 1);
        let public_webrtc_address = webrtc_listen_address;

        let mut server_socket =
            ServerSocket::listen(bind_address, webrtc_listen_address, public_webrtc_address).await;
        if cfg!(debug_assertions) {
            server_socket =
                server_socket.with_link_conditioner(&LinkConditionerConfig::good_condition());
        }

        log::info!("Bound listener socket: [{}]", bind_address);

        let sender = server_socket.get_sender();

        let (packet_sender, packet_receiver) = unbounded();
        let (payload_sender, payload_receiver) = unbounded();

        let handler = ConnectionManager::new(
            PacketSocket {
                bind_address,
                packet_receiver,
                payload_sender,
            },
            LaminarConfig::default(),
        );

        Networking {
            server_socket,
            sender,
            packet_sender,
            payload_receiver,
            handler,
            connection_key: "1337".to_string(),
            stats: Default::default(),

            connections: HashMap::new(),
        }
    }

    pub fn send(&mut self, packet: LaminarPacket) {
        if let Some(connection) = self.connections.get_mut(&packet.addr()) {
            connection.stats.add_tx(packet.payload().len());
        }
        if let Err(error) = self.handler.event_sender().send(packet) {
            panic!("Error sending via event channel: {}", error);
        }
    }

    pub async fn process(
        &mut self,
        world: &mut World,
        config: &mut Config,
        messages: &mut VecDeque<(SocketAddr, NetworkMessage)>,
    ) {
        match self.server_socket.receive().await {
            Ok(packet) => {
                let address = packet.address();
                let data = packet.payload();
                let len = data.len();
                log::debug!("<-- Received {} bytes from [{}]", len, address);
                self.stats.add_rx(len);
                if let Some(connection) = self.connections.get_mut(&address) {
                    connection.stats.add_rx(len);
                }
                trace_dump_packet(data);

                if len > 0 {
                    if let Err(error) = self.packet_sender.send(packet).await {
                        log::error!("Error processing packet from [{}]: {}", address, error);
                    }
                }
            }
            Err(error) => {
                match error {
                    NaiaServerSocketError::Wrapped(error) => {
                        log::error!("Naia socket error: {}", error);
                    }
                    NaiaServerSocketError::SendError(addr) => {
                        log::info!("Error sending packet to [{}] - disconnecting", addr);
                        self.connections.remove(&addr);
                    }
                };
            }
        }

        self.handler.manual_poll(Instant::now());

        while let Ok(event) = self.handler.event_receiver().try_recv() {
            match event {
                SocketEvent::Packet(packet) => {
                    let addr = packet.addr();
                    if let Some(message) = self.process_packet(packet, world, config) {
                        messages.push_back((addr, message));
                    }
                }
                SocketEvent::Connect(addr) => {
                    log::info!("!! Connect {}", addr);
                    self.connections.entry(addr).or_insert_with(Connection::new);
                }
                SocketEvent::Timeout(addr) | SocketEvent::Disconnect(addr) => {
                    log::info!("!! Disconnect {}", addr);
                    self.connections.remove(&addr);
                }
            }
        }

        match self.payload_receiver.try_recv() {
            Ok(packet) => {
                let address = packet.address();
                let data = packet.payload();
                log::debug!("--> Sending {} bytes to [{}]", data.len(), address);
                self.stats.add_tx(data.len());
                trace_dump_packet(data);

                if let Err(error) = self.sender.send(packet).await {
                    log::error!("Error sending payload to [{}]: {}", address, error);
                }
            }
            Err(error) => match error {
                TryRecvError::Empty => {}
                TryRecvError::Closed => {
                    panic!("Error receiving from payload channel: {}", error);
                }
            },
        }
    }

    fn process_packet(
        &mut self,
        packet: LaminarPacket,
        world: &mut World,
        config: &mut Config,
    ) -> Option<NetworkMessage> {
        let address = packet.addr();
        let data = packet.payload();
        if data.is_empty() {
            return None;
        }

        let code = data[0];
        match messages::OperationCode::try_from(code) {
            Ok(op_code) => match op_code {
                messages::OperationCode::CCREQ_CONNECT => {
                    let msg = if messages::packet_verify(data) {
                        log::info!("<-> ACCEPT connection from [{:?}]", address);
                        messages::connection_accept()
                    } else {
                        log::info!("<-> REJECT connection from [{:?}]", address);
                        messages::connection_reject()
                    };
                    self.send(LaminarPacket::unreliable(address, msg.to_vec()));
                }
                _ => match self.connections.get_mut(&address) {
                    Some(connection) => {
                        if op_code == messages::OperationCode::CCREQ_READY {
                            log::info!("<-> READY from [{:?}]", address);
                            connection.ready = true;
                        } else {
                            match messages::decode_message(data) {
                                Some(message) => match message {
                                    NetworkMessage::ConnectionAuthorize { nick, key } => {
                                        let msg = if key == self.connection_key {
                                            if !connection.authorized {
                                                connection.authorized = true;
                                                connection.nick = nick;
                                                log::info!(
                                                    "<-> AUTH connection from [{:?}]",
                                                    address
                                                );

                                                connection.entity.replace(world.reserve_entity());
                                            }
                                            messages::connection_authorized(
                                                config.server.motd.clone(),
                                            )
                                        } else {
                                            log::info!(
                                                "<-> REJECT connection from [{:?}]",
                                                address
                                            );
                                            messages::connection_reject()
                                        };
                                        self.send(LaminarPacket::reliable_unordered(
                                            address,
                                            msg.to_vec(),
                                        ));

                                        let mut cvars = Vec::new();
                                        cvar::console::walk(config, |path, node| {
                                            if !path.starts_with("server.") {
                                                if let cvar::Node::Prop(prop) = node.as_node() {
                                                    cvars.push((path.to_owned(), prop.get()));
                                                }
                                            }
                                        });
                                        self.send(LaminarPacket::reliable_unordered(
                                            address,
                                            encode_message(NetworkMessage::Cvars(cvars)).to_vec(),
                                        ))
                                    }
                                    msg => {
                                        if connection.authorized {
                                            return Some(msg);
                                        }
                                    }
                                },
                                None => {
                                    log::error!(
                                        "Unhandled packet: 0x{:x} ({:?}) {} bytes",
                                        code,
                                        op_code,
                                        data.len()
                                    );
                                    trace_dump_packet(data);
                                }
                            }
                        }
                    }
                    None => {
                        log::error!("Received packet for unknown connection: [{}]", address);
                    }
                },
            },
            Err(_) => {
                log::error!("Unknown packet: 0x{:x}", code);
            }
        }

        None
    }

    pub fn process_simulation(&mut self, server: &mut Server<GameWorld>) {
        for snapshot in server.take_outgoing_snapshots().drain(..) {
            log::trace!("outgoing snapshot: {:?}", snapshot);
            let mut packets = Vec::new();
            for (&address, connection) in self.connections.iter_mut() {
                // FIXME: state snapshot should be localized to entity
                if let Some(_entity) = connection.entity {
                    // TODO: snapshotting can be distributed, does not need to be all at the same time
                    // let next_broadcast = connection.last_broadcast + BROADCAST_RATE;
                    // if next_broadcast < time.time {
                    // connection.last_broadcast = next_broadcast;
                    connection.last_broadcast = Instant::now();
                    packets.push(LaminarPacket::unreliable(
                        address,
                        encode_message(NetworkMessage::Snapshot(snapshot.clone())).to_vec(),
                    ));
                }
            }
            for packet in packets.drain(..) {
                self.send(packet);
            }
        }

        for (from, to, command) in server.take_outgoing_commands().drain(..) {
            log::trace!("outgoing command: {:?} -> {:?}: {:?}", from, to, command);
            let destinations = self.connections.iter().map(|(addr, _conn)| *addr);
            let destinations: Vec<SocketAddr> = if let Some(to) = to {
                // find if 'to' is connected
                destinations.filter(|addr| *addr == to).collect()
            } else if let Some(from) = from {
                // do not broadcast to oneself
                destinations.filter(|addr| *addr != from).collect()
            } else {
                // broadcast to all
                destinations.collect()
            };
            let msg = encode_message(NetworkMessage::Command(command));
            for address in destinations {
                self.send(LaminarPacket::reliable_unordered(address, msg.to_vec()));
            }
        }
    }

    pub fn broadcast_state(&mut self, world: &World, time: &systems::Time) {
        let mut packets = Vec::new();

        for (&address, connection) in self.connections.iter_mut() {
            if let Some(entity) = connection.entity {
                let next_broadcast = connection.last_broadcast
                    + Duration::from_millis((BROADCAST_RATE * 1000.) as u64);
                if next_broadcast < time.time {
                    connection.last_broadcast = next_broadcast;

                    let msg = build_state_message(world, entity, time);
                    packets.push(LaminarPacket::unreliable(address, msg.to_vec()));
                }
            }
        }
    }

    pub fn post_process(&mut self, config: &Config) {
        let mut to_disconnect = Vec::new();

        for (address, connection) in &self.connections {
            if connection.ready {
                if config.net.send_keepalive > 0
                    && connection.stats.last_tx.elapsed().as_secs_f32()
                        > config.net.send_keepalive as f32
                {
                    log::warn!("Sending keepalive to [{}]", address);
                    let packet = NaiaPacket::new(*address, Vec::new());
                    let sender = &mut self.sender;
                    smol::block_on(async {
                        if let Err(error) = sender.send(packet).await {
                            log::error!("Error sending keepalive to [{}]: {}", address, error);
                        }
                    });
                }
                if config.net.keepalive_timeout > 0
                    && connection.stats.last_rx.elapsed().as_secs_f32()
                        > config.net.keepalive_timeout as f32
                {
                    log::error!("Client [{}] connection timeout - dropping", address);
                    to_disconnect.push(*address);
                }
            }
        }

        for address in to_disconnect.drain(..) {
            self.connections.remove(&address);
        }
    }
}
