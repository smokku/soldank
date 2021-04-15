use hecs::World;
use std::{collections::VecDeque, net::SocketAddr};

use crate::{messages::NetworkMessage, systems};

pub fn tick_debug(world: &World, time: &systems::Time) {
    log::debug!("tick {}, entities: {}", time.tick, world.len());
    for (entity, entity_ref) in world.iter() {
        log::debug!("{:?}, components: {:?}", entity, entity_ref.len());
    }
}

pub fn message_dump(messages: &mut VecDeque<(SocketAddr, NetworkMessage)>) {
    for (addr, message) in messages.drain(..) {
        log::warn!("{}: {:#?}", addr, message);
    }
}
