use naia_server_socket::{LinkConditionerConfig, ServerSocket};
use simple_logger::SimpleLogger;
use std::net::{IpAddr, SocketAddr};

use soldank_shared::constants::{DEFAULT_MAP, SERVER_PORT};

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
            .get_matches();

        let bind_address = if let Some(addr) = cmd.value_of("bind") {
            addr.parse().expect("can't parse bind address")
        } else {
            let server_ip_address: IpAddr = "127.0.0.1" // Put your Server's IP Address here!, can't easily find this automatically from the browser
                .parse()
                .expect("couldn't parse input IP address");
            SocketAddr::new(server_ip_address, SERVER_PORT)
        };

        let mut map_name = cmd.value_of("map").unwrap_or(DEFAULT_MAP).to_owned();
        map_name.push_str(".pms");
        log::info!("Using map: {}", map_name);

        let mut server_socket = ServerSocket::listen(bind_address)
            .await
            .with_link_conditioner(&LinkConditionerConfig::good_condition());

        log::info!("Bound listener socket: {}", bind_address);

        let mut sender = server_socket.get_sender();

        loop {
            match server_socket.receive().await {
                Ok(packet) => {
                    networking::process_packet(packet, &mut sender).await;
                }
                Err(error) => {
                    log::error!("Server Error: {}", error);
                }
            }
        }
    })
}
