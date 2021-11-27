use crate::constants;

pub fn parse_cli_args<'a>() -> clap::ArgMatches<'a> {
    clap::app_from_crate!()
        .arg(
            clap::Arg::with_name("map")
                .help("name of map to load")
                .short("m")
                .long("map")
                .takes_value(true)
                .default_value(constants::DEFAULT_MAP),
        )
        .arg(
            clap::Arg::with_name("debug")
                .help("display debug UI on start (^` to toggle)")
                .long("debug"),
        )
        .arg(
            clap::Arg::with_name("connect")
                .value_name("address:port")
                .help("server address and port to connect")
                .short("c")
                .long("connect")
                .takes_value(true),
        )
        .arg(
            clap::Arg::with_name("key")
                .help("server connection key")
                .short("k")
                .long("key")
                .takes_value(true),
        )
        .arg(
            clap::Arg::with_name("nick")
                .help("user nickname")
                .short("n")
                .long("nick")
                .takes_value(true),
        )
        .arg(
            clap::Arg::with_name("set")
                .help("set cvar value [multiple]")
                .long("set")
                .takes_value(true)
                .allow_hyphen_values(true)
                .multiple(true)
                .number_of_values(2)
                .value_names(&["cvar", "value"]),
        )
        .get_matches()
}
