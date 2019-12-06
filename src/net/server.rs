use mio::net::TcpStream;
use mio::net::TcpListener;
use std::net::SocketAddr;
use crate::net::peer::Peer;

pub struct Server {
    listener: TcpListener
}

impl Server {
    pub fn new(addr: SocketAddr) -> Self {
        let listener = TcpListener::bind(&addr).unwrap();

        Server {
            listener
        }
    }

    pub fn listener(&self) -> &TcpListener {
        &self.listener
    }

    pub fn accept(&self) -> Peer {
        let stream = self.listener.accept().unwrap().0;
        Peer::new(stream)
    }
}