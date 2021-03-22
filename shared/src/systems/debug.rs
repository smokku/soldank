use std::{collections::VecDeque, net::SocketAddr};

use crate::{messages::NetworkMessage, systems};

pub fn tick_debug(time: &systems::Time) {
    log::debug!("tick {}", time.tick);
}

pub fn message_dump(messages: &mut VecDeque<(SocketAddr, NetworkMessage)>) {
    for (addr, message) in messages.drain(..) {
        log::warn!("{}: {:#?}", addr, message);
    }
}
