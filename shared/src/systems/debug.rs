use legion::system;
use std::{collections::VecDeque, net::SocketAddr};

use crate::messages::NetworkMessage;

#[system]
pub fn tick_debug() {
    log::debug!("tick");
}

#[system]
pub fn message_dump(#[resource] messages: &mut VecDeque<(SocketAddr, NetworkMessage)>) {
    for (addr, message) in messages.drain(..) {
        log::warn!("{}: {:#?}", addr, message);
    }
}
