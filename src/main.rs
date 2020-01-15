use bssrv::proto::{ClientMessage, ServerMessage, Deserializer};
use bssrv::types::{Position, RestoreState, Who, Hits};
use std::net::{SocketAddr, IpAddr, Ipv4Addr};
use std::time::Duration;
use log::{Level, LevelFilter};
use simplelog::{Config, SimpleLogger, ConfigBuilder, TermLogger, TerminalMode};
use bssrv::net::{ServerManager, Server};
//
//struct Ser {
//}
//
//struct Des {
//}
//
//impl Serializer<ServerMessage> for Ser {
//    fn new() -> Self {
//        unimplemented!()
//    }
//
//    fn serialize(message: ServerMessage) -> Vec<u8> {
//        unimplemented!()
//    }
//}
//
//impl Deserializer<ClientMessage> for Des {
//    fn new() -> Self {
//        unimplemented!()
//    }
//
//    fn deserialize(serialized: &str) -> Option<Vec<ClientMessage>> {
//        unimplemented!()
//    }
//}

fn main() {
    TermLogger::init(LevelFilter::Trace, Config::default(), TerminalMode::Stderr).unwrap();
    let man = ServerManager::new();

    let mut server: Server = Server::new(
        &SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 20000),
        Duration::from_secs(1),
        man
    ).unwrap();

    server.run().unwrap();

}
