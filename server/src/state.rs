use bytes::Bytes;
use hecs::{Entity, World};

use crate::systems;

pub fn build_state_message(_world: &World, _entity: Entity, _time: &systems::Time) -> Bytes {
    Bytes::new()
}
