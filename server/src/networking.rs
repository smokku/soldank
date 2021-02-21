use naia_server_socket::{MessageSender, Packet};
use std::convert::TryFrom;

use soldank_shared::{messages, trace_dump_packet};

pub async fn process_packet(packet: Packet, sender: &mut MessageSender) {
    let address = packet.address();
    let data = packet.payload();
    log::debug!("<--- Received {} bytes from [{}]", data.len(), address);
    trace_dump_packet(data);

    let code = data[0];
    match messages::OperationCode::try_from(code) {
        Ok(op_code) => match op_code {
            messages::OperationCode::CCREQ_CONNECT => {
                log::info!("---> Accepting connection from [{:?}]", address);
                let msg = messages::connection_accept();
                trace_dump_packet(&msg);

                match sender.send(Packet::new(address, msg.to_vec())).await {
                    Ok(()) => {}
                    Err(error) => {
                        log::error!("Error accepting connection from [{}]: {}", address, error);
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
