use bytes::Bytes;
use hecs::{Entity, World};
use std::collections::HashMap;

use crate::systems;
use soldank_shared::{components, messages::*};

pub fn build_state_message(world: &World, _client_entity: Entity, time: &systems::Time) -> Bytes {
    // TODO: scope updates to client_entity visibility range
    // FIXME: send only entities changed since last client acknowledged tick

    let mut entities = HashMap::new();
    for (entity, entity_ref) in world.iter() {
        let components = entities.entry(entity).or_insert_with(Vec::new);
        if let Some(soldier) = entity_ref.get::<components::Soldier>() {
            components.push(ComponentValue::Soldier((*soldier).clone()));
        }
        if let Some(nick) = entity_ref.get::<components::Nick>() {
            components.push(ComponentValue::Nick((*nick).clone()));
        }
        if let Some(pos) = entity_ref.get::<components::Position>() {
            components.push(ComponentValue::Pos((*pos).clone()));
        }
    }

    encode_message(NetworkMessage::GameState {
        tick: time.tick,
        entities,
    })
}
