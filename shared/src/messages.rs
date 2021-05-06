use bytes::Bytes;
use enum_primitive_derive::Primitive;
use hecs::Entity;
use nanoserde::{DeBin, SerBin};
use std::{collections::HashMap, convert::TryFrom, mem::size_of};

use crate::components;
use crate::control::Control;

const NET_PROTOCOL_VERSION: u8 = 0x01;

#[allow(non_camel_case_types)]
#[repr(u8)]
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
        entities: HashMap<Entity, Vec<ComponentValue>>,
    },
}

#[derive(Debug)]
pub enum ComponentValue {
    Soldier(components::Soldier),
    Nick(components::Nick),
    Pos(components::Position),
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
        NetworkMessage::GameState { tick, entities } => {
            let mut msg = vec![OperationCode::STT_ENTITIES as u8];
            let pkt = StatePacket { tick };

            msg.extend(SerBin::serialize_bin(&pkt));

            msg.push(entities.len() as u8); // max 255 entities in packet
            for (entity, components) in entities {
                msg.extend(entity.to_bits().to_be_bytes().to_vec());
                msg.push(components.len() as u8);
                for component in components {
                    match component {
                        ComponentValue::Soldier(soldier) => {
                            msg.push(1);
                            msg.extend(SerBin::serialize_bin(&soldier));
                        }
                        ComponentValue::Nick(nick) => {
                            msg.push(2);
                            msg.extend(SerBin::serialize_bin(&nick));
                        }
                        ComponentValue::Pos(pos) => {
                            msg.push(3);
                            msg.extend(SerBin::serialize_bin(&pos));
                        }
                    }
                }
            }

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
                let mut offset = 1;
                if let Ok(StatePacket { tick }) = DeBin::de_bin(&mut offset, data) {
                    let mut entities = HashMap::new();
                    if offset < data.len() {
                        let mut entities_count = data[offset];
                        offset += 1;
                        while entities_count > 0 {
                            entities_count -= 1;

                            let entity = if offset + size_of::<u64>() < data.len() {
                                let data: [u8; size_of::<u64>()] = [
                                    data[offset],
                                    data[offset + 1],
                                    data[offset + 2],
                                    data[offset + 3],
                                    data[offset + 4],
                                    data[offset + 5],
                                    data[offset + 6],
                                    data[offset + 7],
                                ];
                                offset += size_of::<u64>();
                                Entity::from_bits(u64::from_be_bytes(data))
                            } else {
                                log::error!(
                                    "@{}: Cannot deserialize {} entity id",
                                    offset,
                                    entities_count
                                );
                                return None;
                            };

                            if offset < data.len() {
                                let mut components = Vec::new();

                                let mut components_count = data[offset];
                                offset += 1;
                                while components_count > 0 {
                                    components_count -= 1;

                                    if offset < data.len() {
                                        let component_type = data[offset];
                                        offset += 1;
                                        match component_type {
                                            1 => {
                                                if let Ok(soldier) =
                                                    components::Soldier::de_bin(&mut offset, data)
                                                {
                                                    components
                                                        .push(ComponentValue::Soldier(soldier))
                                                } else {
                                                    log::error!(
                                                        "@{}: Cannot deserialize Soldier component",
                                                        offset
                                                    );
                                                    return None;
                                                }
                                            }
                                            2 => {
                                                if let Ok(nick) =
                                                    components::Nick::de_bin(&mut offset, data)
                                                {
                                                    components.push(ComponentValue::Nick(nick))
                                                } else {
                                                    log::error!(
                                                        "@{}: Cannot deserialize Nick component",
                                                        offset
                                                    );
                                                    return None;
                                                }
                                            }
                                            3 => {
                                                if let Ok(pos) =
                                                    components::Position::de_bin(&mut offset, data)
                                                {
                                                    components.push(ComponentValue::Pos(pos))
                                                } else {
                                                    log::error!(
                                                        "@{}: Cannot deserialize Position component",
                                                        offset
                                                    );
                                                    return None;
                                                }
                                            }
                                            t => {
                                                log::error!(
                                                    "@{}: Unhandled component type: {}",
                                                    offset,
                                                    t
                                                );
                                                return None;
                                            }
                                        }
                                    } else {
                                        log::error!(
                                            "@{}: Not enough data to deserialize {} component",
                                            offset,
                                            components_count
                                        );
                                        return None;
                                    }
                                }

                                entities.insert(entity, components);
                            } else {
                                log::error!("@{}: Not enough data to get components count", offset);
                                return None;
                            }
                        }

                        return Some(NetworkMessage::GameState { tick, entities });
                    } else {
                        log::error!("@{}: Not enough data to get entities count", offset);
                        return None;
                    }
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
