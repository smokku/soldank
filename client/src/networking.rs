use gfx2d::macroquad::logging as log;
use naia_client_socket::{
    find_my_ip_address, ClientSocket, ClientSocketTrait, LinkConditionerConfig, MessageSender,
    Packet,
};
use std::{convert::TryFrom, net::SocketAddr};

use soldank_shared::{constants::SERVER_PORT, messages, trace_dump_packet};

#[derive(PartialEq)]
pub enum ConnectionState {
    Disconnected,
    Connected,
    Error,
}

pub struct Networking {
    client_socket: Box<dyn ClientSocketTrait>,
    sender: MessageSender,
    pub connection_key: String,
    pub nick_name: String,
    state: ConnectionState,
    backoff_round: i32,
    last_message_received: f64,
}

fn backoff_enabled(round: i32) -> bool {
    (round & (round - 1)) == 0
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
        let mut client_socket = ClientSocket::connect(server_socket_address)
            .with_link_conditioner(&LinkConditionerConfig::good_condition());
        let sender = client_socket.get_sender();

        Networking {
            client_socket,
            sender,
            connection_key: "1337".to_string(),
            nick_name: "Player".to_string(),
            state: ConnectionState::Disconnected,
            backoff_round: 0,
            last_message_received: 0.,
        }
    }

    pub fn update(&mut self) {
        loop {
            match self.client_socket.receive() {
                Ok(event) => match event {
                    Some(packet) => {
                        self.last_message_received = instant::now();

                        let data = packet.payload();
                        log::debug!("<--- Received {} bytes", data.len());
                        trace_dump_packet(data);

                        let code = data[0];
                        match messages::OperationCode::try_from(code) {
                            Ok(op_code) => match op_code {
                                messages::OperationCode::CCREP_ACCEPT => {
                                    if self.state == ConnectionState::Disconnected
                                        && messages::packet_verify(data)
                                    {
                                        log::info!("---> Connection accepted");
                                        self.state = ConnectionState::Connected;
                                    }
                                }
                                messages::OperationCode::CCREP_REJECT => {
                                    log::info!("---> Connection rejected");
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
                    None => {
                        if self.state == ConnectionState::Disconnected {
                            if backoff_enabled(self.backoff_round) {
                                let msg = messages::connection_request();
                                log::info!(
                                    "---> Connecting server (backoff {})",
                                    self.backoff_round
                                );
                                trace_dump_packet(&msg);
                                self.sender
                                    .send(Packet::new(msg.to_vec()))
                                    .expect("send error");
                            }
                            self.backoff_round += 1;
                        }
                        return;
                    }
                },
                Err(err) => {
                    log::error!("Client Error: {}", err);
                }
            }
        }
    }
}
