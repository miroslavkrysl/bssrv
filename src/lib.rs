pub mod proto;

pub mod types;
pub mod net;
//pub mod app;


use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::net::SocketAddr;
use std::time::Duration;
use std::str::FromStr;
use crate::net::{Server, Poller, PollEvent, PeerErrorKind, Peer};
use crate::proto::{ClientMessage, ServerMessage};
use std::io::Error;


/// A configuration values for the server.
pub struct Config {
    address: SocketAddr,
    max_players: usize,
    peer_timeout: Duration
}

impl Config {
    pub fn address(&self) -> &SocketAddr {
        &self.address
    }

    pub fn max_players(&self) -> usize {
        self.max_players
    }

    pub fn peer_timeout(&self) -> &Duration {
        &self.peer_timeout
    }
}

impl Default for Config {
    fn default() -> Self {
        Config {
            address: SocketAddr::from_str("127.0.0.1:8191").unwrap(),
            max_players: 128,
            peer_timeout: Duration::from_secs(5)
        }
    }
}

pub fn run_server(config: Config) {
    let mut server = Server::new(config.address().clone()).unwrap();

    let mut poller = Poller::new(128).unwrap();

    poller.register_listener(server.listener(), 0).unwrap();

    let mut new_peers = Vec::new();
    let mut closed_peers = Vec::new();
    let mut reregister_peers = Vec::new();
    let mut incoming_messages = Vec::new();

    loop {
        let events = poller.poll(Some(Duration::from_secs(1))).unwrap();

        println!("poll");

        for event in events {
            println!("event: {:?}", event);

            match event {
                PollEvent::Accept(_) => {
                    println!("accept peer");
                    let peer = server.listener().accept_peer().unwrap();
                    let id = server.add_peer(peer);
                    new_peers.push(id);
                },
                PollEvent::Read(id) => {
                    let peer = server.peer_mut(id).unwrap();

                    match peer.do_read() {
                        Ok(mut messages) => {
                            incoming_messages.extend(messages.drain(..).map(|m| (*id, m)));
                        },
                        Err(error) => {
                            match error.kind() {
                                PeerErrorKind::Closed => {},
                                PeerErrorKind::Deserialization(e) => {
                                    println!("error: {}", e)
                                },
                            }
                            closed_peers.push(*id);
                        },
                    }
                },
                PollEvent::Write(id) => {
                    println!("write");
                    let peer = server.peer_mut(id).unwrap();

                    match peer.do_write() {
                        Ok(_) => {
                            reregister_peers.push(*id);
                        },
                        Err(error) => {
                            closed_peers.push(*id);
                        },
                    }
                },
            }
        }

        for id in closed_peers.drain(..) {
            println!("close peer");
            let peer = server.remove_peer(&id).unwrap();
            peer.close();
            poller.deregister_peer(&peer, &id).unwrap();
        }

        for id in new_peers.drain(..) {
            println!("new peer");
            let peer = server.peer(&id).unwrap();
            poller.register_peer(&peer, id).unwrap();
        }

        for (id, message) in incoming_messages.drain(..) {
            println!("{}: {}", id, message);

            if let ClientMessage::Alive = message {
                let peer = server.peer_mut(&id).unwrap();
                peer.add_message(&ServerMessage::AliveOk);
                reregister_peers.push(id);
            }
        }

        for id in reregister_peers.drain(..) {
            let peer = server.peer(&id).unwrap();
            poller.reregister_peer(peer, &id).unwrap();
        }
    }
}