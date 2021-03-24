use hecs::World;
use std::{
    collections::{HashMap, VecDeque},
    net::SocketAddr,
};

use crate::{
    networking::{Connection, Networking},
    GameState,
};
pub use soldank_shared::systems::*;
use soldank_shared::{components, control::Control, messages::NetworkMessage};

pub type ControlComponent = HashMap<usize, (Control, i32, i32)>;

pub fn process_network_messages(
    world: &mut World,
    messages: &mut VecDeque<(SocketAddr, NetworkMessage)>,
    connections: &mut HashMap<SocketAddr, Connection>,
) {
    let mut control_updates = HashMap::new();
    let mut unprocessed = Vec::new();

    for (addr, message) in messages.drain(..) {
        match message {
            NetworkMessage::ControlState {
                ack_tick,
                begin_tick,
                control,
            } => {
                if let Some(conn) = connections.get_mut(&addr) {
                    conn.ack_tick = ack_tick;
                } else {
                    log::error!("Processing message from unknown connection: [{}]", addr);
                }

                // TODO: check constraints and update connection.cheats
                control_updates.insert(addr, (begin_tick, control));
            }
            _ => {
                unprocessed.push((addr, message));
            }
        }
    }

    for (_entity, (addr, control)) in world.query::<(&SocketAddr, &mut ControlComponent)>().iter() {
        if let Some((tick, mut ctrl)) = control_updates.remove(addr) {
            for (i, (c, x, y)) in ctrl.drain(..).enumerate() {
                control.insert(tick + i, (c, x, y));
            }
        }
    }

    if !control_updates.is_empty() {
        log::error!(
            "Still have {} control updates for not existing entities",
            control_updates.len()
        );
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
