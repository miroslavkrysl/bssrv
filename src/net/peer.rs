use mio::net::TcpStream;

pub struct Peer {
    stream: TcpStream
}

impl Peer {
    pub fn new(stream: TcpStream) -> Self {
        Peer{
            stream
        }
    }
}