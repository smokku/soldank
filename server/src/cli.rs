use crate::constants;

pub fn parse_cli_args<'a>() -> clap::ArgMatches<'a> {
    clap::app_from_crate!()
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
                .default_value(constants::DEFAULT_MAP)
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
        .get_matches()
}
