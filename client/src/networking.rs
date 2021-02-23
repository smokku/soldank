use gfx2d::macroquad::logging as log;
use naia_client_socket::{
    ClientSocket, ClientSocketTrait, LinkConditionerConfig, MessageSender, Packet,
};
use std::{
    convert::TryFrom,
    net::{IpAddr, SocketAddr},
};

use soldank_shared::{constants::SERVER_PORT, messages, trace_dump_packet};

pub struct Networking {
    client_socket: Box<dyn ClientSocketTrait>,
    message_sender: MessageSender,
    connecting: bool,
    backoff_round: i32,
    last_message_received: f64,
}

fn backoff_enabled(round: i32) -> bool {
    (round & (round - 1)) == 0
}

impl Networking {
    pub fn new() -> Networking {
        let server_ip_address: IpAddr = "127.0.0.1" // Put your Server's IP Address here!, can't easily find this automatically from the browser
            .parse()
            .expect("couldn't parse input IP address");

        let server_socket_address = SocketAddr::new(server_ip_address, SERVER_PORT);

        let mut client_socket = ClientSocket::connect(server_socket_address)
            .with_link_conditioner(&LinkConditionerConfig::good_condition());
        let message_sender = client_socket.get_sender();

        Networking {
            client_socket,
            message_sender,
            connecting: true, // TODO: make it state: enum
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
                                    if self.connecting && messages::packet_verify(data) {
                                        log::info!("---> Connection accepted");
                                        self.connecting = false;
                                    }
                                }
                                messages::OperationCode::CCREP_REJECT => {
                                    log::info!("---> Connection rejected");
                                    self.connecting = false;
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
                        if self.connecting {
                            if backoff_enabled(self.backoff_round) {
                                let msg = messages::connection_request();
                                log::info!(
                                    "---> Connecting server (backoff {})",
                                    self.backoff_round
                                );
                                trace_dump_packet(&msg);
                                self.message_sender
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
