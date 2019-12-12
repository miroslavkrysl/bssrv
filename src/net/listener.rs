use std::net::SocketAddr;
use std::io;
use mio::{Evented, Poll, Token, Ready, PollOpt};
use mio::net::TcpListener;
use crate::net::peer::Peer;

pub struct Listener {
    listener: TcpListener
}

impl Listener {
    pub fn new(addr: SocketAddr) -> Self {
        // TODO: bind error handling
        let listener = TcpListener::bind(&addr).unwrap();

        Listener {
            listener
        }
    }

    pub fn accept(&self) -> Peer {
        // TODO: accept error handling
        let stream = self.listener.accept().unwrap().0;
        Peer::new(stream)
    }
}

impl Evented for Listener {
    fn register(&self, poll: &Poll, token: Token, interest: Ready, opts: PollOpt) -> io::Result<()> {
        self.listener.register(poll, token, interest, opts)
    }

    fn reregister(&self, poll: &Poll, token: Token, interest: Ready, opts: PollOpt) -> io::Result<()> {
        self.listener.reregister(poll, token, interest, opts)
    }

    fn deregister(&self, poll: &Poll) -> io::Result<()> {
        self.listener.deregister(poll)
    }
}