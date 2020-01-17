use simplelog::{TermLogger, TerminalMode};
use log::LevelFilter;
use bssrv::{run_game_server, Config};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use log::error;

fn main() {
    let logger_config = simplelog::ConfigBuilder::new()
        .add_filter_allow(format!("{}", "bssrv"))
        .build();
    TermLogger::init(LevelFilter::Trace, logger_config, TerminalMode::Stdout).unwrap();

    let shutdown = Arc::new(AtomicBool::new(false));

    // setup ctrl-c handler
    let s = shutdown.clone();
    ctrlc::set_handler(move || {
        s.store(true, Ordering::SeqCst);
    }).expect("Error setting Ctrl-C handler.");

    match run_game_server(Config::default(), shutdown) {
        Ok(_) => {},
        Err(error) => {
            error!("Error while running the server: {}", error);
        },
    }
}
