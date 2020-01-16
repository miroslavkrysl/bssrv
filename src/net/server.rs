use crate::net::listener::Listener;
use std::collections::HashMap;
use crate::net::peer::Peer;
use rand::Rng;
use std::net::SocketAddr;
use std::io;
use std::collections::hash_map;

pub struct Server {
    listener: Listener,
    listener_id: usize,
    peers: HashMap<usize, Peer>,
}

impl Server {
    pub fn new(address: SocketAddr) -> io::Result<Self>{
        Ok(Server {
            listener: Listener::new(address)?,
            listener_id: 0,
            peers: HashMap::new()
        })
    }

    /// Get unique id for a new peer.
    fn unique_id(&self) -> usize {
        loop {
            let id = rand::thread_rng().gen();
            if id != self.listener_id && !self.peers.contains_key(&id) {
                break id
            }
        }
    }

    pub fn add_peer(&mut self, peer: Peer) -> usize {
        let id = self.unique_id();
        self.peers.insert(id, peer);
        id
    }

    pub fn remove_peer(&mut self, id: &usize) -> Option<Peer> {
        self.peers.remove(id)
    }

    pub fn listener(&self) -> &Listener {
        &self.listener
    }

    pub fn listener_mut(&mut self) -> &mut Listener {
        &mut self.listener
    }

    pub fn peer(&self, id: &usize) -> Option<&Peer> {
        self.peers.get(id)
    }

    pub fn peer_mut(&mut self, id: &usize) -> Option<&mut Peer> {
        self.peers.get_mut(id)
    }

    pub fn peers(&self) -> hash_map::Iter<usize, Peer> {
        self.peers.iter()
    }
}