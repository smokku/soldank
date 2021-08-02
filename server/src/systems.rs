use hecs::World;
use std::{
    collections::{HashMap, VecDeque},
    net::SocketAddr,
};

use crate::{
    networking::{Connection, Networking},
    GameState,
};
use soldank_shared::{components, control::Control, messages::NetworkMessage, systems};
pub use soldank_shared::{
    math::Vec2,
    systems::{Time, *},
};

pub type ControlBuffer = HashMap<usize, (Control, Vec2)>;

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

    for (_entity, (addr, mut control)) in world.query::<(&SocketAddr, &mut ControlBuffer)>().iter()
    {
        if let Some((tick, mut ctrl)) = control_updates.remove(addr) {
            if let Some(connection) = connections.get(addr) {
                for (i, (c, v)) in ctrl.drain(..).enumerate() {
                    let t = tick + i;
                    if t > connection.last_processed_tick {
                        control.insert(tick + i, (c, v));
                    }
                }
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
            .all(|(_, conn)| conn.authorized && conn.entity.is_some() && conn.ready);

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
                    ControlBuffer::default(),
                    components::Position::new(0., 0.), // FIXME: remove this
                ),
            );
        }
    }
}

pub fn apply_input(world: &World, time: &Time) {
    let tick = time.tick;

    for (entity, buffer) in world.query::<&mut ControlBuffer>().iter() {
        // FIXME: apply all queued inputs in rollback manner
        let max_tick = buffer.keys().max().unwrap();
        if let Some((control, _)) = buffer.get(&max_tick) {
            systems::apply_input(world.entity(entity).unwrap(), *control);
        } else {
            log::warn!(
                "Missed input for tick {}({}) on entity {:?}",
                tick,
                max_tick,
                entity
            );
        }
    }
}
