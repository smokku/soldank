#[macro_use]
extern crate log;

use naia_server_socket::{LinkConditionerConfig, Packet, ServerSocket};
use simple_logger::SimpleLogger;
use std::net::{IpAddr, SocketAddr};

use soldank_shared::{constants::SERVER_PORT, messages::*};

fn main() -> smol::io::Result<()> {
    smol::block_on(async {
        SimpleLogger::from_env()
            .init()
            .expect("A logger was already initialized");

        info!("Soldank Server Started");

        let server_ip_address: IpAddr = "127.0.0.1" // Put your Server's IP Address here!, can't easily find this automatically from the browser
            .parse()
            .expect("couldn't parse input IP address");
        let current_socket_address = SocketAddr::new(server_ip_address, SERVER_PORT);

        let mut server_socket = ServerSocket::listen(current_socket_address)
            .await
            .with_link_conditioner(&LinkConditionerConfig::good_condition());

        let mut sender = server_socket.get_sender();

        loop {
            match server_socket.receive().await {
                Ok(packet) => {
                    let address = packet.address();
                    let message = String::from_utf8_lossy(packet.payload());
                    info!("Server recv <- {}: {}", address, message);

                    if message.eq(PING_MSG) {
                        let to_client_message: String = PONG_MSG.to_string();
                        info!("Server send -> {}: {}", address, to_client_message);
                        sender
                            .send(Packet::new(address, to_client_message.into_bytes()))
                            .await
                            .expect("send error");
                    }
                }
                Err(error) => {
                    info!("Server Error: {}", error);
                }
            }
        }
    })
}
