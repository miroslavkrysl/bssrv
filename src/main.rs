use simplelog::{TermLogger, TerminalMode};
use log::LevelFilter;
use bssrv::{run_game_server, Config};

fn main() {
    TermLogger::init(LevelFilter::Trace, simplelog::Config::default(), TerminalMode::Stderr).unwrap();

    println!("main");
    run_game_server(Config::default());



//    // setup ctrl-c handler
//    ctrlc::set_handler(move || {
//        stop_handle.stop();
//    }).expect("Error setting Ctrl-C handler.");
}
