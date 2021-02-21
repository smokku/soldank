use gfx2d::macroquad::logging as log;
use naia_client_socket::{
    ClientSocket, ClientSocketTrait, LinkConditionerConfig, MessageSender, Packet,
};
use std::net::{IpAddr, SocketAddr};

use soldank_shared::{constants::SERVER_PORT, messages::*};

pub struct Networking {
    client_socket: Box<dyn ClientSocketTrait>,
    message_sender: MessageSender,
    got_one_response: bool,
    backoff_round: i32,
    message_count: u8,
}

fn backoff_enabled(round: i32) -> bool {
    (round & (round - 1)) == 0
}

impl Networking {
    pub fn new() -> Networking {
        let server_ip_address: IpAddr = "127.0.0.1"
            .parse()
            .expect("couldn't parse input IP address"); // Put your Server's IP Address here!, can't easily find this automatically from the browser

        let server_socket_address = SocketAddr::new(server_ip_address, SERVER_PORT);

        let mut client_socket = ClientSocket::connect(server_socket_address)
            .with_link_conditioner(&LinkConditionerConfig::good_condition());
        let mut message_sender = client_socket.get_sender();

        message_sender
            .send(Packet::new(PING_MSG.to_string().into_bytes()))
            .unwrap();

        Networking {
            client_socket,
            message_sender,
            got_one_response: false,
            backoff_round: 0,
            message_count: 0,
        }
    }

    pub fn update(&mut self) {
        loop {
            match self.client_socket.receive() {
                Ok(event) => match event {
                    Some(packet) => {
                        let message = String::from_utf8_lossy(packet.payload());
                        log::debug!("Client recv: {}", message);

                        if message.eq(PONG_MSG) && self.message_count < 128 {
                            if !self.got_one_response {
                                log::debug!("Got first PONG");
                                self.got_one_response = true;
                            }
                            self.message_count += 1;
                            let to_server_message: String = PING_MSG.to_string();
                            log::debug!(
                                "Client send: {} (count {})",
                                to_server_message,
                                self.message_count
                            );
                            self.message_sender
                                .send(Packet::new(to_server_message.into_bytes()))
                                .expect("send error");
                        }
                    }
                    None => {
                        if !self.got_one_response {
                            if backoff_enabled(self.backoff_round) {
                                let to_server_message: String = PING_MSG.to_string();
                                log::debug!(
                                    "Client send: {} (backoff {})",
                                    to_server_message,
                                    self.backoff_round
                                );
                                self.message_sender
                                    .send(Packet::new(to_server_message.into_bytes()))
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
