use legion::system;
use std::collections::VecDeque;

use soldank_shared::messages::NetworkMessage;

#[system]
pub fn process_network_messages(#[resource] messages: &mut VecDeque<NetworkMessage>) {
    let mut unprocessed = Vec::new();

    for message in messages.drain(..) {
        match message {
            NetworkMessage::ControlState(_control) => {
                // TODO: check constraints and update entity and connection.cheats
            }
            _ => {
                unprocessed.push(message);
            }
        }
    }

    messages.extend(unprocessed);
}
