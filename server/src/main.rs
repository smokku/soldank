#[macro_use]
extern crate clap;

use legion::{systems::CommandBuffer, Resources, Schedule, World};
use simple_logger::SimpleLogger;
use std::{collections::VecDeque, net::SocketAddr};

use networking::Networking;
use soldank_shared::{constants::DEFAULT_MAP, messages::NetworkMessage, systems::*};

mod cheat;
mod networking;
mod systems;
use systems::*;

fn main() -> smol::io::Result<()> {
    smol::block_on(async {
        SimpleLogger::from_env()
            .init()
            .expect("A logger was already initialized");

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

        let mut world = World::default();
        let mut resources = Resources::default();

        let mut schedule = Schedule::builder()
            .add_system(tick_debug_system())
            .add_system(process_network_messages_system())
            .add_system(message_dump_system())
            .build();

        resources.insert(networking);
        let messages: VecDeque<(SocketAddr, NetworkMessage)> = VecDeque::new();
        resources.insert(messages);

        let mut command_buffer = CommandBuffer::new(&world);

        loop {
            {
                let networking = &mut resources.get_mut::<Networking>().unwrap();
                let messages = &mut resources
                    .get_mut::<VecDeque<(SocketAddr, NetworkMessage)>>()
                    .unwrap();
                networking.process(messages, &mut command_buffer).await; // loop is driven by incoming packets
            }
            command_buffer.flush(&mut world, &mut resources);
            schedule.execute(&mut world, &mut resources); // TODO: limit to 30 ticks per second
        }

        // log::info!("Exiting server");
        // Ok(())
    })
}
