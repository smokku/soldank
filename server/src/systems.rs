use hecs::World;
use std::{
    collections::{HashMap, VecDeque},
    net::SocketAddr,
};

use crate::{systems, GameState, Networking};
pub use soldank_shared::systems::*;
use soldank_shared::{components, control::Control, messages::NetworkMessage};

pub type ControlComponent = (Control, i32, i32);

pub fn process_network_messages(
    world: &mut World,
    messages: &mut VecDeque<(SocketAddr, NetworkMessage)>,
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

    for (_entity, (addr, control)) in world.query::<(&SocketAddr, &mut ControlComponent)>().iter() {
        if let Some(ctrl) = updates.get(addr) {
            *control = *ctrl;
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
                    (Control::default(), 0, 0) as systems::ControlComponent,
                ),
            );
        }
    }
}
