use legion::{system, systems::CommandBuffer, world::SubWorld, Query};
use std::{
    collections::{HashMap, VecDeque},
    net::SocketAddr,
};

use crate::{systems, GameState, Networking};
use soldank_shared::{components, control::Control, messages::NetworkMessage};

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

#[system]
pub fn lobby(
    #[resource] game_state: &mut GameState,
    #[resource] networking: &mut Networking,
    cmd: &mut CommandBuffer,
) {
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

        for (addr, conn) in networking.connections.iter() {
            let entity = conn.entity.unwrap();
            cmd.add_component(entity, components::Soldier {});
            cmd.add_component(entity, components::Nick(conn.nick.clone()));
            cmd.add_component(entity, *addr);
            cmd.add_component(
                entity,
                (Control::default(), 0, 0) as systems::ControlComponent,
            );
        }
    }
}
