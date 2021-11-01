use crate::{
    components,
    control::Control,
    math::Vec2,
    networking::{NetCommand, NetSnapshot},
};
use bytes::Bytes;
use enum_primitive_derive::Primitive;
use hecs::Entity;
use nanoserde::{DeBin, SerBin};
use num_traits::{FromPrimitive, ToPrimitive};
use orb::timestamp::{Timestamp, Timestamped};
use std::{collections::HashMap, convert::TryFrom, mem::size_of, process::abort};

const NET_PROTOCOL_VERSION: u8 = 0x01;

#[allow(non_camel_case_types)]
#[repr(u8)]
#[derive(Debug, Eq, PartialEq, Primitive)]
pub enum OperationCode {
    // incoming
    CCREQ_CONNECT = 0x01,
    CCREQ_AUTHORIZE = 0x02,
    CCREQ_READY = 0x08,
    STT_CONTROL = 0x10,
    STT_ENTITIES = 0x11,
    STT_SNAPSHOT = 0x12,
    STT_COMMAND = 0x13,
    STT_CVARS = 0x18,
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
    Cvars(Vec<(String, String)>),
    ControlState {
        ack_tick: usize,
        begin_tick: usize,
        control: Vec<(Control, Vec2)>,
    },
    GameState {
        tick: usize,
        entities: HashMap<Entity, Vec<ComponentValue>>,
    },
    Snapshot(Timestamped<NetSnapshot>),
    Command(Timestamped<NetCommand>),
}

#[repr(u8)]
#[derive(Debug, Eq, PartialEq, Primitive)]
pub enum ComponentType {
    Soldier = 1,
    Nick = 2,
    Pos = 3,
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
        NetworkMessage::Cvars(cvars) => {
            let mut msg = vec![OperationCode::STT_CVARS as u8];
            msg.extend(SerBin::serialize_bin(&cvars));
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
                control: control
                    .iter()
                    .map(|(c, v)| {
                        let v = v.normalize_or_zero();
                        (
                            *c,
                            ((((v.x + 0.5) * 255.) as u8), ((v.y + 0.5) * 255.) as u8),
                        )
                    })
                    .collect(),
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
                msg.extend(entity.to_bits().get().to_be_bytes().to_vec());
                msg.push(components.len() as u8);
                for component in components {
                    match component {
                        ComponentValue::Soldier(soldier) => {
                            msg.push(ComponentType::Soldier.to_u8().unwrap());
                            msg.extend(SerBin::serialize_bin(&soldier));
                        }
                        ComponentValue::Nick(nick) => {
                            msg.push(ComponentType::Nick.to_u8().unwrap());
                            msg.extend(SerBin::serialize_bin(&nick));
                        }
                        ComponentValue::Pos(pos) => {
                            msg.push(ComponentType::Pos.to_u8().unwrap());
                            msg.extend(SerBin::serialize_bin(&pos));
                        }
                    }
                }
            }

            msg.into()
        }
        NetworkMessage::Snapshot(snapshot) => {
            let mut msg = vec![OperationCode::STT_SNAPSHOT as u8];
            let pkt = SnapshotPacket {
                timestamp: snapshot.timestamp().to_i16(),
                snapshot: snapshot.inner().clone(),
            };
            msg.extend(SerBin::serialize_bin(&pkt));
            msg.into()
        }
        NetworkMessage::Command(command) => {
            let mut msg = vec![OperationCode::STT_COMMAND as u8];
            let pkt = CommandPacket {
                timestamp: command.timestamp().to_i16(),
                command: command.inner().clone(),
            };
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
            | OperationCode::CCREP_AUTHORIZED
            | OperationCode::CCREQ_READY => {
                panic!("Should not handle packet: 0x{:x} ({:?})", code, op_code)
            }
            OperationCode::CCREQ_AUTHORIZE => {
                if let Ok(AuthPacket { nick, key }) = DeBin::deserialize_bin(&data[1..]) {
                    return Some(NetworkMessage::ConnectionAuthorize { nick, key });
                }
            }
            OperationCode::STT_CVARS => {
                if let Ok(cvars) = DeBin::deserialize_bin(&data[1..]) {
                    return Some(NetworkMessage::Cvars(cvars));
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
                        control: control
                            .iter()
                            .map(|&(c, (x, y))| {
                                (
                                    c,
                                    Vec2::new((x as f32) / 255. - 0.5, (y as f32) / 255. - 0.5),
                                )
                            })
                            .collect(),
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
                                Entity::from_bits(u64::from_be_bytes(data)).unwrap()
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
                                        if let Some(component_type) =
                                            ComponentType::from_u8(component_type)
                                        {
                                            match component_type {
                                                ComponentType::Soldier => {
                                                    if let Ok(soldier) = components::Soldier::de_bin(
                                                        &mut offset,
                                                        data,
                                                    ) {
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
                                                ComponentType::Nick => {
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
                                                ComponentType::Pos => {
                                                    if let Ok(pos) = components::Position::de_bin(
                                                        &mut offset,
                                                        data,
                                                    ) {
                                                        components.push(ComponentValue::Pos(pos))
                                                    } else {
                                                        log::error!(
                                                        "@{}: Cannot deserialize Position component",
                                                        offset
                                                    );
                                                        return None;
                                                    }
                                                }
                                            }
                                        } else {
                                            log::error!(
                                                "@{}: Unhandled component type: {}",
                                                offset,
                                                component_type
                                            );
                                            return None;
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
            OperationCode::STT_SNAPSHOT => {
                if let Ok(SnapshotPacket {
                    timestamp,
                    snapshot,
                }) = DeBin::deserialize_bin(&data[1..])
                {
                    return Some(NetworkMessage::Snapshot(Timestamped::<NetSnapshot>::new(
                        snapshot,
                        Timestamp::from_i16(timestamp),
                    )));
                }
            }
            OperationCode::STT_COMMAND => {
                if let Ok(CommandPacket { timestamp, command }) = DeBin::deserialize_bin(&data[1..])
                {
                    return Some(NetworkMessage::Command(Timestamped::<NetCommand>::new(
                        command,
                        Timestamp::from_i16(timestamp),
                    )));
                }
            }
        }
    }
    None
}

pub fn connection_request() -> Bytes {
    vec![
        OperationCode::CCREQ_CONNECT as u8,
        b'S',
        b'L',
        b'D',
        b'T',
        NET_PROTOCOL_VERSION,
    ]
    .into()
}

pub fn connection_accept() -> Bytes {
    vec![
        OperationCode::CCREP_ACCEPT as u8,
        b'S',
        b'L',
        b'D',
        b'T',
        NET_PROTOCOL_VERSION,
    ]
    .into()
}

pub fn connection_reject() -> Bytes {
    vec![
        OperationCode::CCREP_REJECT as u8,
        b'S',
        b'L',
        b'D',
        b'T',
        NET_PROTOCOL_VERSION,
    ]
    .into()
}

pub fn packet_verify(packet: &[u8]) -> bool {
    packet[1] == b'S'
        && packet[2] == b'L'
        && packet[3] == b'D'
        && packet[4] == b'T'
        && packet[5] == NET_PROTOCOL_VERSION
}

pub fn connection_authorized<S: AsRef<str>>(motd: S) -> Bytes {
    let motd = motd.as_ref().as_bytes();
    if motd.len() > u8::MAX as usize {
        log::error!("Server MOTD is longer than {} bytes", u8::MAX);
        abort();
    }
    let mut msg = vec![OperationCode::CCREP_AUTHORIZED as u8];
    msg.push(motd.len() as u8);
    msg.extend_from_slice(motd);
    msg.into()
}

pub fn connection_ready() -> Bytes {
    vec![OperationCode::CCREQ_READY as u8].into()
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
    control: Vec<(Control, (u8, u8))>,
}

#[derive(DeBin, SerBin)]
struct StatePacket {
    tick: usize,
}

#[derive(DeBin, SerBin)]
struct SnapshotPacket {
    timestamp: i16,
    snapshot: NetSnapshot,
}

#[derive(DeBin, SerBin)]
struct CommandPacket {
    timestamp: i16,
    command: NetCommand,
}
