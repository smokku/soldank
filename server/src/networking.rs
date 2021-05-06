use hecs::{Entity, World};
use instant::Instant;
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
};

use crate::{cheat::Cheats, constants::*, state::build_state_message, systems};
use soldank_shared::{
    constants::SERVER_PORT,
    messages::{self, NetworkMessage},
    trace_dump_packet,
};

pub struct Networking {
    server_socket: Box<dyn ServerSocketTrait>,
    sender: MessageSender,
    packet_sender: Sender<NaiaPacket>,
    payload_receiver: Receiver<NaiaPacket>,
    handler: ConnectionManager<PacketSocket, VirtualConnection>,
    pub connection_key: String,
    last_message_received: f64,

    pub connections: HashMap<SocketAddr, Connection>,
}

#[derive(Debug)]
pub struct Connection {
    pub last_message_received: f64,
    pub ack_tick: usize,
    pub last_processed_tick: usize,
    pub last_broadcast: f64,
    pub authorized: bool,
    pub nick: String,
    pub cheats: Cheats,
    pub entity: Option<Entity>,
}

impl Connection {
    pub fn new() -> Connection {
        Connection {
            last_message_received: instant::now(),
            ack_tick: 0,
            last_processed_tick: 0,
            last_broadcast: 0.0,
            authorized: false,
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
                panic!("{}", error);
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
                    panic!("{}", error);
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

        let mut server_socket = ServerSocket::listen(bind_address).await;
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
            last_message_received: 0.,

            connections: HashMap::new(),
        }
    }

    pub fn send(&mut self, packet: LaminarPacket) {
        if let Err(error) = self.handler.event_sender().send(packet) {
            panic!("{}", error);
        }
    }

    pub async fn process(
        &mut self,
        world: &mut World,
        messages: &mut VecDeque<(SocketAddr, NetworkMessage)>,
    ) {
        match self.server_socket.receive().await {
            Ok(packet) => {
                self.last_message_received = instant::now();

                let address = packet.address();
                let data = packet.payload();
                log::debug!("<-- Received {} bytes from [{}]", data.len(), address);
                trace_dump_packet(data);

                if let Err(error) = self.packet_sender.send(packet).await {
                    log::error!("Error processing packet from [{}]: {}", address, error);
                }
            }
            Err(error) => {
                log::error!("Server error: {}", error);
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

        match self.payload_receiver.try_recv() {
            Ok(packet) => {
                let address = packet.address();
                let data = packet.payload();
                log::debug!("--> Sending {} bytes to [{}]", data.len(), address);
                trace_dump_packet(data);

                if let Err(error) = self.sender.send(packet).await {
                    log::error!("Error sending payload to [{}]: {}", address, error);
                }
            }
            Err(error) => match error {
                TryRecvError::Empty => {}
                TryRecvError::Closed => {
                    panic!("{}", error);
                }
            },
        }

        while let Ok(event) = self.handler.event_receiver().try_recv() {
            match event {
                SocketEvent::Packet(packet) => {
                    let addr = packet.addr();
                    if let Some(message) = self.process_packet(packet, world) {
                        messages.push_back((addr, message));
                    }
                }
                SocketEvent::Connect(addr) => {
                    log::info!("!! Connect {}", addr);
                    if !self.connections.contains_key(&addr) {
                        self.connections.insert(addr, Connection::new());
                    }
                }
                SocketEvent::Timeout(addr) | SocketEvent::Disconnect(addr) => {
                    log::info!("!! Disconnect {}", addr);
                    self.connections.remove(&addr);
                }
            }
        }
    }

    fn process_packet(
        &mut self,
        packet: LaminarPacket,
        world: &mut World,
    ) -> Option<NetworkMessage> {
        let address = packet.addr();
        let data = packet.payload();
        if data.len() < 1 {
            return None;
        }

        let code = data[0];
        match messages::OperationCode::try_from(code) {
            Ok(op_code) => match op_code {
                messages::OperationCode::CCREQ_CONNECT => {
                    let msg = if messages::packet_verify(data) {
                        log::info!("<-> Accepting connection from [{:?}]", address);
                        messages::connection_accept()
                    } else {
                        log::info!("<-> Rejecting connection from [{:?}]", address);
                        messages::connection_reject()
                    };
                    self.send(LaminarPacket::unreliable(address, msg.to_vec()));
                }
                _ => match self.connections.get_mut(&address) {
                    Some(connection) => {
                        connection.last_message_received = instant::now();
                        match messages::decode_message(data) {
                            Some(message) => match message {
                                NetworkMessage::ConnectionAuthorize { nick, key } => {
                                    let msg = if key == self.connection_key {
                                        if !connection.authorized {
                                            connection.authorized = true;
                                            connection.nick = nick;
                                            log::info!(
                                                "<-> Authorized connection from [{:?}]",
                                                address
                                            );

                                            connection.entity.replace(world.reserve_entity());
                                        }

                                        messages::connection_authorized()
                                    } else {
                                        log::info!("<-> Rejecting connection from [{:?}]", address);
                                        messages::connection_reject()
                                    };
                                    self.send(LaminarPacket::reliable_unordered(
                                        address,
                                        msg.to_vec(),
                                    ));
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

    pub fn broadcast_state(&mut self, world: &World, time: &systems::Time) {
        let mut packets = Vec::new();

        for (&address, connection) in self.connections.iter_mut() {
            if let Some(entity) = connection.entity {
                let next_broadcast = connection.last_broadcast + BROADCAST_RATE;
                if next_broadcast < time.time {
                    connection.last_broadcast = next_broadcast;

                    let msg = build_state_message(world, entity, time);
                    packets.push(LaminarPacket::unreliable(address, msg.to_vec()));
                }
            }
        }

        for packet in packets.drain(..) {
            self.send(packet);
        }
    }
}
