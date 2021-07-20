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
use std::{collections::HashMap, convert::TryFrom, net::SocketAddr};

use crate::cvars::Config;
use soldank_shared::{
    constants::SERVER_PORT,
    control::Control,
    math::vec2,
    messages::{self, NetworkMessage},
    networking::MyWorld,
    orb::client::Client,
    trace_dump_packet,
};

#[derive(PartialEq)]
pub enum ConnectionState {
    Disconnected,
    Connected,
    Error,
}

type ReceiveEvent = <VirtualConnection as Connection>::ReceiveEvent;

const MAX_INPUTS_RETAIN: usize = 60;

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
    authorized: bool,
    cvars_received: bool,

    pub tick: usize,
    server_tick_received: usize,

    // game state
    control: HashMap<usize, (Control, i32, i32)>,
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
            authorized: false,
            cvars_received: false,

            tick: 0,
            server_tick_received: 0,

            control: Default::default(),
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

    pub fn process(&mut self, config: &mut Config, client: &mut Client<MyWorld>) {
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
                laminar::SocketEvent::Packet(packet) => self.process_packet(packet, config, client),
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
            let mut inputs = self
                .control
                .iter()
                .map(|(&key, v)| (key, v.0, v.1, v.2))
                .collect::<Vec<(usize, Control, i32, i32)>>();

            if !inputs.is_empty() {
                inputs.sort_by_key(|v| v.0);

                let msg = NetworkMessage::ControlState {
                    ack_tick: self.server_tick_received,
                    begin_tick: inputs[0].0,
                    control: inputs
                        .iter()
                        .map(|&(_t, c, x, y)| (c, vec2(x as f32, y as f32)))
                        .collect(),
                };
                log::debug!("--> Sending {:?}", msg);
                self.send(LaminarPacket::unreliable(
                    self.server_address,
                    messages::encode_message(msg).to_vec(),
                ));
            }
        }
    }

    pub fn send(&mut self, event: LaminarPacket) {
        self.connection
            .process_event(&mut self.messenger, event, Instant::now());
    }

    fn process_packet(
        &mut self,
        packet: LaminarPacket,
        config: &mut Config,
        client: &mut Client<MyWorld>,
    ) {
        let data = packet.payload();
        if data.len() < 1 {
            return;
        }

        let code = data[0];
        match messages::OperationCode::try_from(code) {
            Ok(op_code) => match op_code {
                messages::OperationCode::CCREP_ACCEPT => {
                    if self.state == ConnectionState::Disconnected && messages::packet_verify(data)
                    {
                        log::info!("<-> Connection accepted");
                        self.state = ConnectionState::Connected;

                        self.send(LaminarPacket::reliable_unordered(
                            self.server_address,
                            messages::encode_message(NetworkMessage::ConnectionAuthorize {
                                nick: self.nick_name.clone(),
                                key: self.connection_key.clone(),
                            })
                            .to_vec(),
                        ));
                    }
                }
                messages::OperationCode::CCREP_REJECT => {
                    log::info!("<-> Connection rejected");
                    self.state = ConnectionState::Error;
                }
                messages::OperationCode::CCREP_AUTHORIZED => {
                    self.authorized = true;
                    if data.len() > 1 {
                        let motd_len = data[1] as usize;
                        if data.len() >= motd_len + 2 {
                            let motd = String::from_utf8_lossy(&data[2..motd_len + 2]);
                            log::info!("Got server MOTD: {}", motd);
                        }
                    }
                }
                _ => {
                    if !self.process_message(data, config, client) {
                        log::error!(
                            "Unhandled packet: 0x{:x} ({:?}) {} bytes",
                            code,
                            op_code,
                            data.len()
                        );
                        trace_dump_packet(data);
                    }
                }
            },
            Err(_) => {
                log::error!("Unknown packet: 0x{:x}", code);
            }
        }
    }

    fn process_message(
        &mut self,
        data: &[u8],
        config: &mut Config,
        client: &mut Client<MyWorld>,
    ) -> bool {
        if let Some(msg) = messages::decode_message(data) {
            match msg {
                NetworkMessage::ConnectionAuthorize { .. }
                | NetworkMessage::ControlState { .. } => {
                    log::error!("Should not receive message: {:?}", msg);
                }
                NetworkMessage::Cvars(cvars) => {
                    log::info!("--- cvars server sync:");
                    for (path, val) in cvars {
                        if let Some(old_val) = cvar::console::get(config, path.as_str()) {
                            if old_val == val {
                                continue;
                            }
                            match cvar::console::set(config, path.as_str(), val.as_str()) {
                                Ok(set) => {
                                    if set {
                                        log::info!("{} = `{}`", path, val);
                                    }
                                }
                                Err(err) => {
                                    log::error!("Error for {} = `{}`: {}", path, val, err)
                                }
                            }
                        }
                    }

                    self.cvars_received = true;
                    self.send(LaminarPacket::reliable_unordered(
                        self.server_address,
                        messages::connection_ready().to_vec(),
                    ));
                }
                NetworkMessage::GameState { tick, entities } => {
                    if tick > self.server_tick_received {
                        self.server_tick_received = tick;
                        self.tick = tick;

                        // TODO: integrate game state to ECS world
                        log::debug!("Entity sync: {:?}", entities);
                    }
                }
                NetworkMessage::Snapshot(snapshot) => {
                    log::debug!("Got snapshot {}", snapshot.timestamp());
                    client.client.enqueue_incoming_snapshot(snapshot);
                }
                NetworkMessage::Command(command) => {
                    log::debug!("Got command {}: {:?}", command.timestamp(), command.inner());
                    client.client.enqueue_incoming_command(command);
                }
            }
            return true;
        }

        return false;
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

        self.control
            .insert(self.tick, (flags, control.mouse_aim_x, control.mouse_aim_y));
    }

    pub fn post_process(&mut self) {
        let low_tick = usize::max(
            self.server_tick_received,
            self.tick - usize::min(self.tick, MAX_INPUTS_RETAIN),
        );
        let high_tick = self.tick;

        self.control
            .retain(move |&t, _| t > low_tick && t <= high_tick);
    }
}
