use hecs::World;
use std::{
    collections::{HashMap, VecDeque},
    net::SocketAddr,
};

use crate::{GameState, Networking};
pub use soldank_shared::systems::*;
use soldank_shared::{components, control::Control, messages::NetworkMessage};

pub type ControlComponent = HashMap<u64, (Control, i32, i32)>;

pub fn process_network_messages(
    world: &mut World,
    messages: &mut VecDeque<(SocketAddr, NetworkMessage)>,
) {
    let mut updates = HashMap::new();
    let mut unprocessed = Vec::new();

    for (addr, message) in messages.drain(..) {
        match message {
            NetworkMessage::ControlState { control } => {
                // TODO: check constraints and update connection.cheats
                updates.insert(addr, control);
            }
            _ => {
                unprocessed.push((addr, message));
            }
        }
    }

    for (_entity, (addr, control)) in world.query::<(&SocketAddr, &mut ControlComponent)>().iter() {
        if let Some(mut ctrl) = updates.remove(addr) {
            for (tick, c, x, y) in ctrl.drain(..) {
                control.insert(tick, (c, x, y));
            }
        }
    }

    messages.extend(unprocessed);
}

pub fn lobby(world: &mut World, game_state: &mut GameState, networking: &Networking) {
    if *game_state != GameState::Lobby {
        log::error!("Running lobby system outside Lobby GameState");
    }

    let ready = networking.connections.len() > 0
        && networking
            .connections
            .iter()
            .all(|(_, conn)| conn.authorized && conn.entity.is_some());

    if ready {
        log::info!("All players ready - switching to InGame state");
        *game_state = GameState::InGame;

        for (&addr, conn) in networking.connections.iter() {
            let entity = conn.entity.unwrap();
            world.spawn_at(
                entity,
                (
                    components::Soldier {},
                    components::Nick(conn.nick.clone()),
                    addr,
                    ControlComponent::default(),
                ),
            );
        }
    }
}
