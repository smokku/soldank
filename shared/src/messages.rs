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
    STT_ENTITIES = 0x11,
    // outgoing
    CCREP_ACCEPT = 0x81,
    CCREP_REJECT = 0x82,
    CCREP_AUTHORIZED = 0x83,
}

#[derive(Debug)]
pub enum NetworkMessage {
    ConnectionAuthorize {
        nick: String,
        key: String,
    },
    ControlState {
        ack_tick: usize,
        begin_tick: usize,
        control: Vec<(Control, i32, i32)>,
    },
    GameState {
        tick: usize,
    },
}

pub fn encode_message(msg: NetworkMessage) -> Bytes {
    match msg {
        NetworkMessage::ConnectionAuthorize { nick, key } => {
            let mut msg = vec![OperationCode::CCREQ_AUTHORIZE as u8];
            let pkt = AuthPacket { nick, key };
            msg.extend(SerBin::serialize_bin(&pkt));
            msg.into()
        }
        NetworkMessage::ControlState {
            ack_tick,
            begin_tick,
            control,
        } => {
            let mut msg = vec![OperationCode::STT_CONTROL as u8];
            let pkt = ControlPacket {
                ack_tick,
                begin_tick,
                control,
            };
            msg.extend(SerBin::serialize_bin(&pkt));
            msg.into()
        }
        NetworkMessage::GameState { tick } => {
            let mut msg = vec![OperationCode::STT_ENTITIES as u8];
            let pkt = StatePacket { tick };
            msg.extend(SerBin::serialize_bin(&pkt));
            msg.into()
        }
    }
}

pub fn decode_message(data: &[u8]) -> Option<NetworkMessage> {
    let code = data[0];
    if let Ok(op_code) = OperationCode::try_from(code) {
        match op_code {
            OperationCode::CCREQ_CONNECT
            | OperationCode::CCREP_ACCEPT
            | OperationCode::CCREP_REJECT
            | OperationCode::CCREP_AUTHORIZED => {
                panic!("Should not handle packet: 0x{:x} ({:?})", code, op_code)
            }
            OperationCode::CCREQ_AUTHORIZE => {
                if let Ok(AuthPacket { nick, key }) = DeBin::deserialize_bin(&data[1..]) {
                    return Some(NetworkMessage::ConnectionAuthorize { nick, key });
                }
            }
            OperationCode::STT_CONTROL => {
                if let Ok(ControlPacket {
                    ack_tick,
                    begin_tick,
                    control,
                }) = DeBin::deserialize_bin(&data[1..])
                {
                    return Some(NetworkMessage::ControlState {
                        ack_tick,
                        begin_tick,
                        control,
                    });
                }
            }
            OperationCode::STT_ENTITIES => {
                if let Ok(StatePacket { tick }) = DeBin::deserialize_bin(&data[1..]) {
                    return Some(NetworkMessage::GameState { tick });
                }
            }
        }
    }
    None
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

pub fn connection_authorized() -> Bytes {
    vec![OperationCode::CCREP_AUTHORIZED as u8].into()
    // TODO: send server info
}

#[derive(DeBin, SerBin)]
struct AuthPacket {
    nick: String,
    key: String,
}

#[derive(DeBin, SerBin)]
struct ControlPacket {
    ack_tick: usize,
    begin_tick: usize,
    control: Vec<(Control, i32, i32)>,
}

#[derive(DeBin, SerBin)]
struct StatePacket {
    tick: usize,
}
