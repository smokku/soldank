use legion::system;
use std::collections::VecDeque;

use crate::messages::NetworkMessage;

#[system]
pub fn tick_debug() {
    log::info!("tick");
}

#[system]
pub fn message_dump(#[resource] messages: &mut VecDeque<NetworkMessage>) {
    for message in messages.drain(..) {
        log::warn!("{:#?}", message);
    }
}
