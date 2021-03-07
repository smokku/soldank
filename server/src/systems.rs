use legion::{system, world::SubWorld, Query};
use std::{
    collections::{HashMap, VecDeque},
    net::SocketAddr,
};

use soldank_shared::{control::Control, messages::NetworkMessage};

pub type ControlComponent = (Control, i32, i32);

#[system]
pub fn process_network_messages(
    #[resource] messages: &mut VecDeque<(SocketAddr, NetworkMessage)>,
    world: &mut SubWorld,
    query: &mut Query<(&SocketAddr, &mut ControlComponent)>,
) {
    let mut updates = HashMap::new();
    let mut unprocessed = Vec::new();

    for (addr, message) in messages.drain(..) {
        match message {
            NetworkMessage::ControlState {
                control,
                aim_x,
                aim_y,
            } => {
                // TODO: check constraints and update connection.cheats
                updates.insert(addr, (control, aim_x, aim_y));
            }
            _ => {
                unprocessed.push((addr, message));
            }
        }
    }

    for (addr, control) in query.iter_mut(world) {
        if let Some(ctrl) = updates.get(addr) {
            *control = *ctrl;
        }
    }

    messages.extend(unprocessed);
}
