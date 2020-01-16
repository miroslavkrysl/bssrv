use simplelog::{TermLogger, TerminalMode};
use log::LevelFilter;
use bssrv::{run_game_server, Config};

fn main() {
    let logger_config = simplelog::ConfigBuilder::new()
        .add_filter_allow(format!("{}", "bssrv"))
        .build();
    TermLogger::init(LevelFilter::Trace, logger_config, TerminalMode::Stderr).unwrap();

    run_game_server(Config::default());



//    // setup ctrl-c handler
//    ctrlc::set_handler(move || {
//        stop_handle.stop();
//    }).expect("Error setting Ctrl-C handler.");
}
