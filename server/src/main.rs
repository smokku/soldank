#[macro_use]
extern crate clap;

use color_eyre::eyre::Result;
use hecs::World;
use smol::future;
use std::{
    collections::VecDeque,
    net::SocketAddr,
    sync::{Arc, RwLock},
    time::{Duration, Instant},
};

use crate::{
    constants::*,
    cvars::{set_cli_cvars, Config, NetConfig},
    networking::Networking,
};
use soldank_shared::{messages::NetworkMessage, networking::MyWorld, orb};

mod cheat;
mod constants;
mod cvars;
mod networking;
mod state;
mod systems;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum GameState {
    Lobby,
    InGame,
}

fn main() -> Result<()> {
    color_eyre::install()?;
    env_logger::init();

    smol::block_on(async {
        let cmd = clap::app_from_crate!()
            .arg(
                clap::Arg::with_name("bind")
                    .value_name("address:port")
                    .help("IP address and port to bind")
                    .short("b")
                    .long("bind")
                    .takes_value(true)
                    .env("SOLDANK_SERVER_BIND"),
            )
            .arg(
                clap::Arg::with_name("map")
                    .value_name("map name")
                    .help("name of map to load")
                    .short("m")
                    .long("map")
                    .takes_value(true)
                    .default_value(DEFAULT_MAP)
                    .env("SOLDANK_USE_MAP"),
            )
            .arg(
                clap::Arg::with_name("key")
                    .help("server connection key")
                    .short("k")
                    .long("key")
                    .takes_value(true)
                    .env("SOLDANK_SERVER_KEY"),
            )
            .arg(
                clap::Arg::with_name("set")
                    .help("set cvar value [multiple]")
                    .long("set")
                    .takes_value(true)
                    .multiple(true)
                    .number_of_values(2)
                    .value_names(&["cvar", "value"]),
            )
            .get_matches();

        let mut map_name = cmd.value_of("map").unwrap_or(DEFAULT_MAP).to_owned();
        map_name.push_str(".pms");
        log::info!("Using map: {}", map_name);

        let mut config = Config {
            net: NetConfig {
                orb: Arc::new(RwLock::new(orb::Config {
                    timestep_seconds: TIMESTEP_RATE,
                    ..Default::default()
                })),
                ..Default::default()
            },
            ..Default::default()
        };
        set_cli_cvars(&mut config, &cmd);

        let mut networking = Networking::new(cmd.value_of("bind")).await;
        if let Some(key) = cmd.value_of("key") {
            networking.connection_key = key.to_string();
        }

        let mut messages: VecDeque<(SocketAddr, NetworkMessage)> = VecDeque::new();

        let mut world = World::new();

        let mut game_state = GameState::Lobby;

        let mut server = orb::server::Server::<MyWorld>::new(config.net.orb.clone(), 0.0);

        let startup_time = Instant::now();
        let mut previous_time = Instant::now();

        let mut running = true;
        while running {
            let timeout = Duration::from_millis(
                (config.net.orb.read().unwrap().snapshot_send_period * 1000.) as _,
            );
            future::race(
                // loop is driven by incoming packets
                networking.process(&mut world, &mut config, &mut messages),
                // or timeout
                async {
                    smol::Timer::after(timeout).await; // drop Timer result
                },
            )
            .await;

            let current_time = Instant::now();
            let delta_seconds = current_time.duration_since(previous_time).as_secs_f64();
            let seconds_since_startup = current_time.duration_since(startup_time).as_secs_f64();

            systems::process_network_messages(
                &mut world,
                &mut messages,
                &mut networking.connections,
            );
            systems::message_dump(&mut messages);

            match game_state {
                GameState::Lobby => {
                    systems::lobby(&mut world, &mut game_state, &networking);
                }
                GameState::InGame => {
                    let server_display_state = server.display_state();
                    log::info!("server_display_state: {:?}", server_display_state);

                    server.update(delta_seconds, seconds_since_startup);

                    networking.process_simulation(&mut server);

                    let time = systems::Time {
                        time: current_time,
                        tick: (server
                            .last_completed_timestamp()
                            .as_seconds(config.net.orb.read().unwrap().timestep_seconds)
                            * 1000.) as usize,
                        frame_percent: 1.,
                    };
                    systems::tick_debug(&world, &time);

                    if networking.connections.iter().count() == 0 {
                        log::info!("No connections left - exiting");
                        running = false;
                    }
                }
            }

            previous_time = current_time;
        }

        log::info!("Exiting server");
        Ok(())
    })
}
