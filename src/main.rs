use bssrv::net::{Server};

fn main() {
    let mut server = Server::new("127.0.0.1:8191".parse().unwrap());
    server.run();
}
