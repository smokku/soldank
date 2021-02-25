use naia_server_socket::{
    find_my_ip_address, LinkConditionerConfig, MessageSender, Packet, ServerSocket,
    ServerSocketTrait,
};
use std::{convert::TryFrom, net::SocketAddr};

use soldank_shared::{constants::SERVER_PORT, messages, trace_dump_packet};

pub struct Networking {
    pub(crate) server_socket: Box<dyn ServerSocketTrait>,
    sender: MessageSender,
    pub connection_key: String,
    last_message_received: f64,
}

impl Networking {
    pub async fn new(bind: Option<&str>) -> Networking {
        let bind_address = if let Some(addr) = bind {
            addr.parse().expect("cannot parse bind address")
        } else {
            let addr = find_my_ip_address().expect("can't find local ip address");
            SocketAddr::new(addr, SERVER_PORT)
        };

        let mut server_socket = ServerSocket::listen(bind_address)
            .await
            .with_link_conditioner(&LinkConditionerConfig::good_condition());

        log::info!("Bound listener socket: {}", bind_address);

        let sender = server_socket.get_sender();

        Networking {
            server_socket,
            sender,
            connection_key: "1337".to_string(),
            last_message_received: 0.,
        }
    }

    pub async fn process_packet(&mut self, packet: Packet) {
        self.last_message_received = instant::now();

        let address = packet.address();
        let data = packet.payload();
        log::debug!("<--- Received {} bytes from [{}]", data.len(), address);
        trace_dump_packet(data);

        let code = data[0];
        match messages::OperationCode::try_from(code) {
            Ok(op_code) => match op_code {
                messages::OperationCode::CCREQ_CONNECT => {
                    let msg = if messages::packet_verify(data) {
                        log::info!("---> Accepting connection from [{:?}]", address);
                        messages::connection_accept()
                    } else {
                        log::info!("---> Rejecting connection from [{:?}]", address);
                        messages::connection_reject()
                    };
                    trace_dump_packet(&msg);

                    match self.sender.send(Packet::new(address, msg.to_vec())).await {
                        Ok(()) => {}
                        Err(error) => {
                            log::error!("Error receiving connection from [{}]: {}", address, error);
                        }
                    }
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
}
