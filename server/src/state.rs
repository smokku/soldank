use bytes::Bytes;
use hecs::{Entity, World};

use crate::systems;
use soldank_shared::messages::*;

pub fn build_state_message(_world: &World, _entity: Entity, time: &systems::Time) -> Bytes {
    encode_message(NetworkMessage::GameState { tick: time.tick })
}
