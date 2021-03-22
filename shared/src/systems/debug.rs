use legion::system;
use std::{collections::VecDeque, net::SocketAddr};

use crate::{messages::NetworkMessage, systems};

#[system]
pub fn tick_debug(#[resource] time: &systems::Time) {
    log::debug!("tick {}", time.tick);
}

#[system]
pub fn message_dump(#[resource] messages: &mut VecDeque<(SocketAddr, NetworkMessage)>) {
    for (addr, message) in messages.drain(..) {
        log::warn!("{}: {:#?}", addr, message);
    }
}
