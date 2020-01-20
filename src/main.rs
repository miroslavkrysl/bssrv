use simplelog::{TermLogger, TerminalMode};
use log::LevelFilter;
use bssrv::{run_game_server, Config};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use log::error;
use clap::{App, Arg};
use std::net::{SocketAddr, IpAddr};
use std::str::FromStr;

fn main() {
    let matches = App::new("Battleships game server")
        .version("0.1.0")
        .author("Miroslav Krýsl <mkrysl@protonmail.com>")
        .about("Runs a Battleships game server on given or default address.")
        .arg(Arg::with_name("ip")
            .short("i")
            .long("ip")
            .value_name("IP_ADDRESS")
            .help("Sets an ip address on which the server listens.")
            .takes_value(true)
            .validator(validate_ip)
            .default_value("0.0.0.0"))
        .arg(Arg::with_name("port")
            .short("p")
            .long("port")
            .value_name("PORT")
            .help("Sets a port on which the server listens.")
            .takes_value(true)
            .validator(validate_port)
            .default_value("10000"))
        .arg(Arg::with_name("players")
            .short("m")
            .long("players")
            .value_name("MAX_PLAYERS")
            .help("Sets a maximum number of players logged into the server.")
            .takes_value(true)
            .validator(validate_players)
            .default_value("1024"))
        .arg(Arg::with_name("log_level")
            .short("l")
            .long("log")
            .possible_values(&["off", "error", "warn", "info", "debug", "trace"])
            .default_value("off")
            .help("Sets the level of logging"))
        .get_matches();


    // get commandline arguments
    let log_level = matches.value_of("log_level").unwrap();
    let ip = matches.value_of("ip").unwrap();
    let port = matches.value_of("port").unwrap();
    let players = matches.value_of("players").unwrap();


    // setup logging
    let log_level = match log_level {
        "off" => LevelFilter::Off,
        "error" => LevelFilter::Error,
        "warn" => LevelFilter::Warn,
        "info" => LevelFilter::Info,
        "debug" => LevelFilter::Debug,
        "trace" => LevelFilter::Trace,
        _ => unreachable!()
    };

    let logger_config = simplelog::ConfigBuilder::new()
        .add_filter_allow(format!("{}", "bssrv"))
        .build();
    TermLogger::init(log_level, logger_config, TerminalMode::Stdout).unwrap();


    // setup ctrl-c handler
    let shutdown = Arc::new(AtomicBool::new(false));

    let s = shutdown.clone();
    ctrlc::set_handler(move || {
        s.store(true, Ordering::SeqCst);
    }).expect("Error setting Ctrl-C handler.");



    // run the server
    let address = SocketAddr::new(ip.parse().unwrap(), port.parse().unwrap());
    let max_players = players.parse().unwrap();
    let config = Config::new(address, max_players);

    match run_game_server(config, shutdown) {
        Ok(_) => {},
        Err(error) => {
            error!("Error while running the server: {}", error);
        },
    }
}

/// Validate the ip address.
fn validate_ip(v: String) -> Result<(), String> {
    let ip = IpAddr::from_str(&v);

    match ip {
        Ok(_) => Ok(()),
        Err(error) => Err(error.to_string()),
    }
}

/// Validate the port.
fn validate_port(v: String) -> Result<(), String> {
    let port = v.parse::<u16>();

    match port {
        Ok(_) => Ok(()),
        Err(error) => Err(error.to_string()),
    }
}


/// Validate the number of players
fn validate_players(v: String) -> Result<(), String> {
    let players = v.parse::<usize>();

    match players {
        Ok(_) => Ok(()),
        Err(error) => Err(error.to_string()),
    }
}