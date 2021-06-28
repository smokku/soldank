#[macro_use]
extern crate clap;

use color_eyre::eyre::Result;
use hecs::World;
use smol::future;
use std::{collections::VecDeque, net::SocketAddr};

use crate::{
    constants::*,
    cvars::{set_cli_cvars, Config},
    networking::Networking,
};
use soldank_shared::messages::NetworkMessage;

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

        let mut config = Config::default();
        set_cli_cvars(&mut config, &cmd);

        let mut networking = Networking::new(cmd.value_of("bind")).await;
        if let Some(key) = cmd.value_of("key") {
            networking.connection_key = key.to_string();
        }

        let mut messages: VecDeque<(SocketAddr, NetworkMessage)> = VecDeque::new();

        let time_start = instant::now();
        let current_time = || (instant::now() - time_start) / 1000.;

        let mut timecur: f64 = current_time();
        let mut timeprv: f64 = timecur;
        let mut timeacc: f64 = 0.0;
        let mut tick: usize = 0;

        let mut world = World::new();

        let mut game_state = GameState::Lobby;

        let mut running = true;
        while running {
            future::race(
                networking.process(&mut world, &mut messages), // loop is driven by incoming packets
                async {
                    smol::Timer::after(MAX_NETWORK_IDLE).await; // or timeout
                },
            )
            .await;

            timecur = current_time();
            timeacc += timecur - timeprv;
            timeprv = timecur;

            match game_state {
                GameState::Lobby => {
                    timeacc = 0.; // avoid spinning unnecessary ticks after starting game simulation

                    systems::process_network_messages(
                        &mut world,
                        &mut messages,
                        &mut networking.connections,
                    );
                    systems::message_dump(&mut messages);
                    systems::lobby(&mut world, &mut game_state, &networking);
                }
                GameState::InGame => {
                    while timeacc >= TIMESTEP_RATE {
                        timeacc -= TIMESTEP_RATE;
                        tick += 1;

                        let time = systems::Time {
                            time: timecur,
                            tick,
                            frame_percent: f64::min(1.0, f64::max(0.0, timeacc / TIMESTEP_RATE)),
                        };

                        // current simulation frame
                        systems::tick_debug(&world, &time);
                        systems::process_network_messages(
                            &mut world,
                            &mut messages,
                            &mut networking.connections,
                        );
                        systems::message_dump(&mut messages);
                        systems::apply_input(&mut world, &time);

                        // update clients
                        networking.broadcast_state(&world, &time);

                        // update time for possibly next run
                        timecur = current_time();
                        timeacc += timecur - timeprv;
                        timeprv = timecur;
                    }

                    if networking.connections.iter().count() == 0 {
                        log::info!("No connections left - exiting");
                        running = false;
                    }
                }
            }
        }

        log::info!("Exiting server");
        Ok(())
    })
}
