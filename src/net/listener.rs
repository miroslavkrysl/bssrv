use crate::net::peer::Peer;
use mio::net::TcpListener;
use mio::{Poll, PollOpt, Ready, Token};
use std::io;
use std::net::SocketAddr;

pub struct Listener {
    address: SocketAddr,
    listener: TcpListener,
}

impl Listener {
    /// Create a new listener.
    pub fn new(address: SocketAddr) -> io::Result<Self> {
        Ok(Listener {
            address,
            listener: TcpListener::bind(&address)?,
        })
    }

    /// Get the address on which this listener listens.
    pub fn address(&self) -> &SocketAddr {
        &self.address
    }

    /// Register the listener for polling.
    pub fn register(&self, poll: &Poll, token: Token) -> io::Result<()> {
        poll.register(&self.listener, token, Ready::readable(), PollOpt::level())
    }

    /// Deregister the listener from polling.
    pub fn deregister(&self, poll: &Poll) -> io::Result<()> {
        poll.deregister(&self.listener)
    }

    /// Accepts a new waiting peer.
    pub fn accept_peer(&self) -> io::Result<Peer> {
        let (stream, address) = self.listener.accept()?;
        Ok(Peer::new(stream, address))
    }
}
