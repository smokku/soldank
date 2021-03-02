use bytes::Bytes;
use enum_primitive_derive::Primitive;
use nanoserde::{DeBin, SerBin};
use std::convert::TryFrom;

use crate::control::Control;

const NET_PROTOCOL_VERSION: u8 = 0x01;

#[allow(non_camel_case_types)]
#[derive(Debug, Eq, PartialEq, Primitive)]
pub enum OperationCode {
    // incoming
    CCREQ_CONNECT = 0x01,
    CCREQ_AUTHORIZE = 0x02,
    STT_CONTROL = 0x10,
    // outgoing
    CCREP_ACCEPT = 0x81,
    CCREP_REJECT = 0x82,
    CCREP_AUTHORIZED = 0x83,
}

pub enum NetworkMessage {
    ConnectionAuthorize { nick: String, key: String },
}

pub fn encode_message(msg: NetworkMessage) -> Option<Bytes> {
    match msg {
        NetworkMessage::ConnectionAuthorize { nick, key } => {
            let mut msg = vec![OperationCode::CCREQ_AUTHORIZE as u8];
            let pkt = AuthPacket { nick, key };
            msg.extend(SerBin::serialize_bin(&pkt));
            Some(msg.into())
        }
    }
}

pub fn decode_message(data: &[u8]) -> Option<NetworkMessage> {
    let code = data[0];
    match OperationCode::try_from(code) {
        Ok(op_code) => match op_code {
            OperationCode::CCREQ_AUTHORIZE => match DeBin::deserialize_bin(&data[1..]) {
                Ok(AuthPacket { nick, key }) => {
                    Some(NetworkMessage::ConnectionAuthorize { nick, key })
                }
                Err(_) => None,
            },
            _ => None,
        },
        Err(_) => None,
    }
}

pub fn connection_request() -> Bytes {
    vec![
        OperationCode::CCREQ_CONNECT as u8,
        'S' as u8,
        'L' as u8,
        'D' as u8,
        'T' as u8,
        NET_PROTOCOL_VERSION,
    ]
    .into()
}

pub fn connection_accept() -> Bytes {
    vec![
        OperationCode::CCREP_ACCEPT as u8,
        'S' as u8,
        'L' as u8,
        'D' as u8,
        'T' as u8,
        NET_PROTOCOL_VERSION,
    ]
    .into()
}

pub fn connection_reject() -> Bytes {
    vec![
        OperationCode::CCREP_REJECT as u8,
        'S' as u8,
        'L' as u8,
        'D' as u8,
        'T' as u8,
        NET_PROTOCOL_VERSION,
    ]
    .into()
}

pub fn packet_verify(packet: &[u8]) -> bool {
    packet[1] == 'S' as u8
        && packet[2] == 'L' as u8
        && packet[3] == 'D' as u8
        && packet[4] == 'T' as u8
        && packet[5] == NET_PROTOCOL_VERSION
}

pub fn control_state(control: Control) -> Bytes {
    let mut msg = vec![OperationCode::STT_CONTROL as u8];
    msg.extend(control.bits().to_be_bytes().to_vec());
    msg.into()
}

#[derive(DeBin, SerBin)]
struct AuthPacket {
    nick: String,
    key: String,
}
