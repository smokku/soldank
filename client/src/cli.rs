use crate::constants;

pub fn parse_cli_args<'a>() -> clap::ArgMatches {
    clap::Command::new(env!("CARGO_PKG_NAME"))
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .arg(
            clap::Arg::new("map")
                .help("name of map to load")
                .short('m')
                .long("map")
                .num_args(1)
                .default_value(constants::DEFAULT_MAP),
        )
        .arg(
            clap::Arg::new("debug")
                .help("display debug UI on start (^` to toggle)")
                .long("debug"),
        )
        .arg(
            clap::Arg::new("connect")
                .value_name("address:port")
                .help("server address and port to connect")
                .short('c')
                .long("connect")
                .num_args(1),
        )
        .arg(
            clap::Arg::new("key")
                .help("server connection key")
                .short('k')
                .long("key")
                .num_args(1),
        )
        .arg(
            clap::Arg::new("nick")
                .help("user nickname")
                .short('n')
                .long("nick")
                .num_args(1),
        )
        .arg(
            clap::Arg::new("set")
                .help("set cvar value [multiple]")
                .long("set")
                .num_args(2)
                .allow_hyphen_values(true)
                .action(clap::ArgAction::Append)
                .value_names(&["cvar", "value"]),
        )
        .get_matches()
}
