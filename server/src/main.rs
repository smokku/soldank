use legion::{Resources, Schedule, World};
use simple_logger::SimpleLogger;

use networking::Networking;
use soldank_shared::{constants::DEFAULT_MAP, systems::*};

mod connections;
mod networking;

fn main() -> smol::io::Result<()> {
    smol::block_on(async {
        SimpleLogger::from_env()
            .init()
            .expect("A logger was already initialized");

        let cmd = clap::App::new("Soldank Server")
            .about(clap::crate_description!())
            .version(clap::crate_version!())
            .author(clap::crate_authors!("\n"))
            .arg(
                clap::Arg::with_name("bind")
                    .value_name("address:port")
                    .help("IP address and port to bind")
                    .short("b")
                    .long("bind")
                    .takes_value(true),
            )
            .arg(
                clap::Arg::with_name("map")
                    .value_name("map name")
                    .help(&format!(
                        "Name of map to load (Defaults to `{}`)",
                        DEFAULT_MAP
                    ))
                    .short("m")
                    .long("map")
                    .takes_value(true),
            )
            .arg(
                clap::Arg::with_name("key")
                    .help("server connection key")
                    .short("k")
                    .long("key")
                    .takes_value(true),
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
        let mut schedule = Schedule::builder().add_system(tick_debug_system()).build();

        loop {
            networking.process().await; // loop is driven by incoming packets

            schedule.execute(&mut world, &mut resources); // TODO: limit to 30 ticks per second
        }

        // log::info!("Exiting server");
        // Ok(())
    })
}
