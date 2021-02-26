use instant::Instant;
use laminar::{Config as LaminarConfig, ConnectionManager, DatagramSocket, VirtualConnection};
use naia_server_socket::{
    find_my_ip_address, LinkConditionerConfig, MessageSender, Packet, ServerSocket,
    ServerSocketTrait,
};
use smol::channel::{unbounded, Receiver, Sender, TryRecvError};
use std::{convert::TryFrom, io, net::SocketAddr};

use soldank_shared::{constants::SERVER_PORT, messages, trace_dump_packet};

pub struct Networking {
    server_socket: Box<dyn ServerSocketTrait>,
    sender: MessageSender,
    packet_sender: Sender<Packet>,
    payload_receiver: Receiver<Packet>,
    handler: ConnectionManager<PacketSocket, VirtualConnection>,
    pub connection_key: String,
    last_message_received: f64,
}

#[derive(Debug)]
struct PacketSocket {
    bind_address: SocketAddr,
    packet_receiver: Receiver<Packet>,
    payload_sender: Sender<Packet>,
}

impl DatagramSocket for PacketSocket {
    fn send_packet(&mut self, addr: &SocketAddr, payload: &[u8]) -> io::Result<usize> {
        match smol::block_on(
            self.payload_sender
                .send(Packet::new(*addr, payload.to_vec())),
        ) {
            Ok(()) => Ok(payload.len()),
            Err(error) => {
                panic!(error);
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
                    panic!(error);
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
        }
    }

    pub async fn process(&mut self) {
        loop {
            match self.server_socket.receive().await {
                Ok(packet) => {
                    self.last_message_received = instant::now();

                    let address = packet.address();
                    let data = packet.payload();
                    log::debug!("<--- Received {} bytes from [{}]", data.len(), address);
                    trace_dump_packet(data);

                    if let Err(error) = self.packet_sender.send(packet).await {
                        log::error!("Error processing packet from [{}]: {}", address, error);
                    }
                }
                Err(error) => {
                    log::error!("Server error: {}", error);
                }
            }

            self.handler.manual_poll(Instant::now());

            match self.payload_receiver.try_recv() {
                Ok(packet) => {
                    let address = packet.address();
                    if let Err(error) = self.sender.send(packet).await {
                        log::error!("Error sending payload to [{}]: {}", address, error);
                    }
                }
                Err(error) => match error {
                    TryRecvError::Empty => {}
                    TryRecvError::Closed => {
                        panic!(error);
                    }
                },
            }

            while let Ok(event) = self.handler.event_receiver().try_recv() {
                println!("{:?}", event);
            }
        }
    }

    // pub async fn process_message(&mut self, packet: Packet) {
    //     let code = data[0];
    //     match messages::OperationCode::try_from(code) {
    //         Ok(op_code) => match op_code {
    //             messages::OperationCode::CCREQ_CONNECT => {
    //                 let msg = if messages::packet_verify(data) {
    //                     log::info!("---> Accepting connection from [{:?}]", address);
    //                     messages::connection_accept()
    //                 } else {
    //                     log::info!("---> Rejecting connection from [{:?}]", address);
    //                     messages::connection_reject()
    //                 };
    //                 trace_dump_packet(&msg);

    //                 match self.sender.send(Packet::new(address, msg.to_vec())).await {
    //                     Ok(()) => {}
    //                     Err(error) => {
    //                         log::error!("Error receiving connection from [{}]: {}", address, error);
    //                     }
    //                 }
    //             }
    //             messages::OperationCode::LAMINAR_PKT => {}
    //             _ => {
    //                 log::error!("Unhandled packet: 0x{:x} ({:?})", code, op_code);
    //             }
    //         },
    //         Err(_) => {
    //             log::error!("Unknown packet: 0x{:x}", code);
    //         }
    //     }
    // }
}
