use std::collections::HashMap;
use std::net::SocketAddr;
use mio::{Poll, Events, Token, Ready, PollOpt, Evented, Event};
use rand::Rng;
use crate::net::{Listener, Peer};

pub struct Server {
    poll: Poll,
    listener: Listener,
    listener_token: Token,
    peers: HashMap<Token, Peer>
}

impl Server {
    pub fn new(addr: SocketAddr) -> Self {
        let poll = Poll::new().unwrap();
        let listener = Listener::new(addr);

        Server {
            poll,
            listener,
            listener_token: Token(0),
            peers: HashMap::new()
        }
    }

    pub fn run(&mut self) {
        // TODO: register error handling
        self.poll.register(&self.listener,
                           self.listener_token,
                           Ready::readable(),
                           PollOpt::level()).unwrap();

        let mut events = Events::with_capacity(1024);

        loop {
            // TODO: poll error handling
            self.poll.poll(&mut events, None).unwrap();

            for event in events.iter() {
                if event.token() == self.listener_token {
                    self.listener_event(&event);
                } else {
                    self.peer_event(&event);
                }
            }
        }
    }

    fn listener_event(&mut self, event: &Event) {
        // TODO: implement max peers limit

        let peer = self.listener.accept();
        self.register_peer(peer);
    }

    fn peer_event(&mut self, event: &Event) {
        let token = event.token();

        if let Some(mut peer) = self.peers.get_mut(&token) {
            peer.handle_io(&event);

            if peer.is_closed() {
                self.deregister_peer(&token);
            }
        }
    }

    fn register_peer(&mut self, peer: Peer) {
        let mut token;

        loop {
            token = Token(rand::thread_rng().gen());

            if token != self.listener_token
                && !self.peers.contains_key(&token) {
                break;
            }
        }

        // TODO: register error handling
        self.poll.register(&peer,
                      token,
                      Ready::readable(),
                      PollOpt::level()).unwrap();

        self.peers.insert(token, peer);
    }


    fn deregister_peer(&mut self, token: &Token) {
        // TODO: deregister error handling
        self.poll.deregister(&self.peers[&token]).unwrap();
        self.peers.remove(token);
    }
}