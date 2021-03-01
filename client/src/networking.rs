use gfx2d::macroquad::logging as log;
use instant::Instant;
use laminar::{
    Config as LaminarConfig, Connection, ConnectionMessenger, Packet as LaminarPacket,
    VirtualConnection,
};
use naia_client_socket::{
    find_my_ip_address, ClientSocket, ClientSocketTrait, LinkConditionerConfig, MessageSender,
    Packet as NaiaPacket,
};
use smol::channel::{unbounded, Receiver, Sender};
use std::{convert::TryFrom, net::SocketAddr};

use soldank_shared::{constants::SERVER_PORT, control::Control, messages, trace_dump_packet};

#[derive(PartialEq)]
pub enum ConnectionState {
    Disconnected,
    Connected,
    Error,
}

type ReceiveEvent = <VirtualConnection as Connection>::ReceiveEvent;

pub struct Networking {
    server_address: SocketAddr,
    client_socket: Box<dyn ClientSocketTrait>,
    messenger: PacketMessenger,
    event_receiver: Receiver<ReceiveEvent>,
    connection: VirtualConnection,
    pub connection_key: String,
    pub nick_name: String,
    state: ConnectionState,
    backoff_round: i32,
    last_message_received: f64,

    // game state
    control: Control,
}

fn backoff_enabled(round: i32) -> bool {
    (round & (round - 1)) == 0
}

struct PacketMessenger {
    config: LaminarConfig,
    sender: MessageSender,
    event_sender: Sender<ReceiveEvent>,
}

impl ConnectionMessenger<ReceiveEvent> for PacketMessenger {
    fn config(&self) -> &LaminarConfig {
        &self.config
    }

    fn send_event(&mut self, _address: &SocketAddr, event: ReceiveEvent) {
        if let Err(error) = smol::block_on(self.event_sender.send(event)) {
            panic!("{}", error);
        }
    }

    fn send_packet(&mut self, _address: &SocketAddr, payload: &[u8]) {
        log::debug!("--> Sending {} bytes", payload.len());
        trace_dump_packet(payload);
        self.sender
            .send(NaiaPacket::new(payload.to_vec()))
            .expect("send packet error");
    }
}

impl Networking {
    pub fn new(connect_to: Option<&str>) -> Networking {
        let server_socket_address = if let Some(addr) = connect_to {
            addr.parse().expect("cannot parse connect address")
        } else {
            let addr = find_my_ip_address().expect("can't find local ip address");
            SocketAddr::new(addr, SERVER_PORT)
        };

        log::info!("Will connect to server: {}", server_socket_address);
        let mut client_socket = ClientSocket::connect(server_socket_address);
        if cfg!(debug_assertions) {
            client_socket =
                client_socket.with_link_conditioner(&LinkConditionerConfig::good_condition());
        }
        let sender = client_socket.get_sender();

        let (event_sender, event_receiver) = unbounded();
        let mut messenger = PacketMessenger {
            config: LaminarConfig::default(),
            sender,
            event_sender,
        };
        let connection = VirtualConnection::create_connection(
            &mut messenger,
            server_socket_address,
            Instant::now(),
        );

        Networking {
            server_address: "0.0.0.0:0".parse().unwrap(),
            client_socket,
            messenger,
            event_receiver,
            connection,
            connection_key: "1337".to_string(),
            nick_name: "Player".to_string(),
            state: ConnectionState::Disconnected,
            backoff_round: 0,
            last_message_received: 0.,

            control: Control::default(),
        }
    }

    pub fn update(&mut self) {
        let time = Instant::now();
        let messenger = &mut self.messenger;

        // pull all newly arrived packets and handle them
        loop {
            match self.client_socket.receive() {
                Ok(event) => match event {
                    Some(packet) => {
                        self.last_message_received = instant::now();

                        let data = packet.payload();
                        log::debug!("<-- Received {} bytes", data.len());
                        trace_dump_packet(data);

                        self.connection.process_packet(messenger, data, time);
                    }
                    None => {
                        break;
                    }
                },
                Err(err) => {
                    log::error!("Client Error: {}", err);
                }
            }
        }

        // update connection
        self.connection.update(messenger, time);
    }

    pub fn process(&mut self) {
        if self.state == ConnectionState::Disconnected {
            if backoff_enabled(self.backoff_round) {
                let msg = messages::connection_request();
                log::info!("--> Connecting server (backoff {})", self.backoff_round);
                self.send(LaminarPacket::unreliable(self.server_address, msg.to_vec()));
            }
            self.backoff_round += 1;
        }

        while let Ok(event) = self.event_receiver.try_recv() {
            match event {
                laminar::SocketEvent::Packet(packet) => self.process_packet(packet),
                laminar::SocketEvent::Connect(addr) => {
                    log::info!("!! Connect {}", addr)
                }
                laminar::SocketEvent::Timeout(addr) => {
                    log::info!("!! Timeout {}", addr)
                }
                laminar::SocketEvent::Disconnect(addr) => {
                    log::info!("!! Disconnect {}", addr)
                }
            }
        }

        if self.state == ConnectionState::Connected {
            let msg = messages::control_state(self.control);
            log::debug!("--> Sending control state: {:?}", self.control);
            self.send(LaminarPacket::unreliable(self.server_address, msg.to_vec()));
        }
    }

    pub fn send(&mut self, event: LaminarPacket) {
        self.connection
            .process_event(&mut self.messenger, event, Instant::now());
    }

    fn process_packet(&mut self, packet: LaminarPacket) {
        let data = packet.payload();
        let code = data[0];
        match messages::OperationCode::try_from(code) {
            Ok(op_code) => match op_code {
                messages::OperationCode::CCREP_ACCEPT => {
                    if self.state == ConnectionState::Disconnected && messages::packet_verify(data)
                    {
                        log::info!("<-> Connection accepted");
                        self.state = ConnectionState::Connected;
                    }
                }
                messages::OperationCode::CCREP_REJECT => {
                    log::info!("<-> Connection rejected");
                    self.state = ConnectionState::Error;
                }
                _ => {
                    log::error!("Unhandled packet: 0x{:x} ({:?})", code, op_code);
                }
            },
            Err(_) => {
                log::error!("Unknown packet: 0x{:x}", code);
            }
        }
    }

    pub fn set_input_state(&mut self, control: &crate::control::Control) {
        let mut flags = Control::default();
        if control.left {
            flags.insert(Control::LEFT);
        }
        if control.right {
            flags.insert(Control::RIGHT);
        }
        if control.up {
            flags.insert(Control::UP);
        }
        if control.down {
            flags.insert(Control::DOWN);
        }
        if control.fire {
            flags.insert(Control::FIRE);
        }
        if control.jets {
            flags.insert(Control::JETS);
        }
        if control.grenade {
            flags.insert(Control::GRENADE);
        }
        if control.change {
            flags.insert(Control::CHANGE);
        }
        if control.throw {
            flags.insert(Control::THROW);
        }
        if control.drop {
            flags.insert(Control::DROP);
        }
        if control.reload {
            flags.insert(Control::RELOAD);
        }
        if control.prone {
            flags.insert(Control::PRONE);
        }
        if control.flag_throw {
            flags.insert(Control::FLAG_THROW);
        }

        self.control = flags;
    }
}
