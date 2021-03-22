#[macro_use]
extern crate clap;

use hecs::World;
use std::{collections::VecDeque, net::SocketAddr};

use networking::Networking;
use soldank_shared::{constants::*, messages::NetworkMessage};

mod cheat;
mod networking;
mod systems;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum GameState {
    Lobby,
    InGame,
}

fn main() -> smol::io::Result<()> {
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
            .get_matches();

        let mut map_name = cmd.value_of("map").unwrap_or(DEFAULT_MAP).to_owned();
        map_name.push_str(".pms");
        log::info!("Using map: {}", map_name);

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
        let mut tick: u64 = 0;

        let mut world = World::new();

        let mut game_state = GameState::Lobby;

        let running = true;
        while running {
            tick += 1;

            networking.process(&mut world, &mut messages).await; // loop is driven by incoming packets

            timecur = current_time();
            timeacc += timecur - timeprv;
            timeprv = timecur;

            match game_state {
                GameState::Lobby => {
                    timeacc = 0.; // avoid spinning unnecessary ticks outside game simulation

                    systems::process_network_messages(&mut world, &mut messages);
                    systems::message_dump(&mut messages);
                    systems::lobby(&mut world, &mut game_state, &networking);
                }
                GameState::InGame => {
                    while timeacc >= FIXED_RATE {
                        timeacc -= FIXED_RATE;

                        let time = systems::Time {
                            time: timecur,
                            tick,
                            frame_percent: f64::min(1.0, f64::max(0.0, timeacc / FIXED_RATE)),
                        };

                        // current simulation frame
                        systems::tick_debug(&time);
                        systems::process_network_messages(&mut world, &mut messages);
                        systems::message_dump(&mut messages);

                        // update time for possibly next run
                        timecur = current_time();
                        timeacc += timecur - timeprv;
                        timeprv = timecur;
                    }
                }
            }
        }

        log::info!("Exiting server");
        Ok(())
    })
}
