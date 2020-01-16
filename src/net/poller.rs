use mio::{Poll, Events, Token};
use std::io;
use crate::net::listener::Listener;
use crate::net::peer::Peer;
use std::time::Duration;
use std::collections::HashSet;

/// An event which can happen on a listener or a peer.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum PollEvent {
    /// A new peer can be accepted on listener with the particular id.
    Accept(usize),
    /// Peer with the particular id has incoming data ready to be read.
    Read(usize),
    /// Peer with the particular id can be written into.
    Write(usize)
}

/// Polls for readiness events on all registered listeners and peers.
pub struct Poller {
    poll: Poll,
    events: Events,
    listeners: HashSet<usize>,
    peers: HashSet<usize>,
}

impl Poller {
    /// Create a new Poller with the given events capacity.
    pub fn new(capacity: usize) -> io::Result<Poller> {
        Ok(Poller {
            poll: Poll::new()?,
            events: Events::with_capacity(capacity),
            listeners: HashSet::new(),
            peers: HashSet::new()
        })
    }

    /// Register a listener for polling.
    pub fn register_listener(&mut self, listener: &Listener, id: usize) -> io::Result<()> {
        if self.listeners.contains(&id) || self.peers.contains(&id) {
            panic!("A poller instance has already registered id {}", id);
        }

        listener.register(&self.poll, Token(id))?;
        self.listeners.insert(id);
        Ok(())
    }

    /// Deregister a listener from polling.
    pub fn deregister_listener(&mut self, listener: &Listener, id: &usize) -> io::Result<()> {
        if !self.listeners.remove(id) {
            panic!("listener with id {} is not present in this poller instance", id);
        }

        listener.deregister(&self.poll)?;
        Ok(())
    }

    /// Register a peer for polling.
    pub fn register_peer(&mut self, peer: &Peer, id: usize) -> io::Result<()> {
        if self.listeners.contains(&id) || self.peers.contains(&id) {
            panic!("A poller instance has already registered id {}", id);
        }

        peer.register(&self.poll, Token(id))?;
        self.peers.insert(id);
        Ok(())
    }

    /// Reregister a peer for polling.
    pub fn reregister_peer(&self, peer: &Peer, id: &usize) -> io::Result<()> {
        if !self.listeners.contains(id) && !self.peers.contains(id) {
            panic!("A poller instance has not registered id {} yet", id);
        }

        peer.reregister(&self.poll, Token::from(*id))?;
        Ok(())
    }

    /// Deregister a peer from polling.
    pub fn deregister_peer(&mut self, peer: &Peer, id: &usize) -> io::Result<()> {
        if !self.peers.remove(id) {
            panic!("listener with id {} is not present in this poller instance", id);
        }

        peer.deregister(&self.poll)?;
        Ok(())
    }

    /// Poll for events on registered listeners and peers.
    /// Events is stored in the provided vector, which is cleared before.
    pub fn poll(&mut self, events: &mut Vec<PollEvent>, timeout: Option<Duration>) -> io::Result<()> {
        // clear events list from previous call
        events.clear();

        self.poll.poll(&mut self.events, timeout)?;

        for event in self.events.iter() {
            let id = event.token().0;

            if self.listeners.contains(&id) {
                // a listener can accept a new peer

                events.push(PollEvent::Accept(id))
            } else if self.peers.contains(&id) {
                // a peer event

                if event.readiness().is_readable() {
                    // peer has incoming data to read
                    events.push(PollEvent::Read(id));
                }

                if event.readiness().is_writable() {
                    // peer can be written into
                    events.push(PollEvent::Write(id))
                }
            } else {
                // sporadic events happen
            }
        }
        
        Ok(())
    }
}