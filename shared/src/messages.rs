use bytes::Bytes;
use enum_primitive_derive::Primitive;

pub const PING_MSG: &str = "ping";
pub const PONG_MSG: &str = "pong";

const NET_PROTOCOL_VERSION: u8 = 0x01;

#[allow(non_camel_case_types)]
#[derive(Debug, Eq, PartialEq, Primitive)]
pub enum OperationCode {
    CCREQ_CONNECT = 0x01,
    CCREP_ACCEPT = 0x81,
    CCREP_REJECT = 0x82,
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
