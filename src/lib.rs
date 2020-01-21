pub mod app;
pub mod game;
pub mod net;
pub mod proto;
pub mod types;

use crate::app::App;
use crate::net::{PeerErrorKind, PollEvent, Poller, Server};
use crate::proto::ServerMessage;
use log::{debug, error, info, warn};
use std::collections::HashSet;
use std::io;
use std::net::SocketAddr;
use std::str::FromStr;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

/// A configuration values for the run_game_server function.
pub struct Config {
    address: SocketAddr,
    max_players: usize,
    peer_timeout: Duration,
    session_timeout: Duration,
}

impl Config {
    /// Get the address on which will be the server listening.
    pub fn address(&self) -> &SocketAddr {
        &self.address
    }

    /// Get the maximum number of players, that can be logged on the server
    pub fn max_players(&self) -> usize {
        self.max_players
    }

    /// Get the time after a peer is disconnected if not active.
    pub fn peer_timeout(&self) -> &Duration {
        &self.peer_timeout
    }

    /// Get the time after a session is removed if not active.
    pub fn session_timeout(&self) -> &Duration {
        &self.session_timeout
    }
}

impl Config {
    /// Create a new server config.
    pub fn new(address: SocketAddr, max_players: usize, peer_timeout: Duration) -> Self {
        Config {
            address,
            max_players,
            peer_timeout,
            session_timeout: Duration::from_secs(300),
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Config {
            address: SocketAddr::from_str("0.0.0.0:10000").unwrap(),
            max_players: 128,
            peer_timeout: Duration::from_secs(10),
            session_timeout: Duration::from_secs(60),
        }
    }
}

/// A command for the running server.
pub enum Command {
    /// Send message to the peer with particular id.
    Message(usize, ServerMessage),

    /// Close the peer with the particular id.
    Close(usize),
}

/// Run the game server.
///
/// Creates a server which listen on configured address and accepts new peers.
/// Received messages are than passed to the App where is processed, resulting
/// actions for the server are returned back and than processed too.
///
/// If the peer is inactive for a longer period than is configured, the peer is disconnected.
pub fn run_game_server(config: Config, shutdown: Arc<AtomicBool>) -> io::Result<()> {
    let mut server = Server::new(config.address().clone())?;
    let mut app = App::new(config.max_players(), config.session_timeout().clone());
    let mut poller = Poller::new(128)?;

    // register servers listener for polling
    poller.register_listener(server.listener(), 0)?;

    let peer_timeout = config.peer_timeout;

    let mut events = Vec::new();
    let mut new_peers = HashSet::new();
    let mut closed_peers = HashSet::new();
    let mut incoming_messages = Vec::new();
    let mut commands: Vec<Command> = Vec::new();
    let mut reregister_peers = HashSet::new();

    let mut end = false;

    info!("starting the server on address: {}", config.address());
    info!("maximum number of players: {}", config.max_players());
    info!(
        "sessions timeout: {} seconds",
        config.session_timeout().as_secs()
    );
    info!(
        "connection timeout: {} seconds",
        config.peer_timeout().as_secs()
    );

    // polling loop
    loop {
        poller.poll(&mut events, Some(Duration::from_secs(1)))?;

        for event in events.drain(..) {
            match event {
                PollEvent::Accept(_) => {
                    let peer = server.listener().accept_peer()?;
                    let address = peer.address().clone();

                    let id = server.add_peer(peer);
                    new_peers.insert(id);

                    debug!("new connection {} accepted from {}", id, address);
                }
                PollEvent::Read(id) => {
                    let peer = server.peer_mut(&id).unwrap();

                    match peer.do_read() {
                        Ok(messages) => {
                            for message in messages {
                                debug!("incoming message from {:0>16X}: {}", id, message);
                                incoming_messages.push((id, message));
                            }
                        }
                        Err(error) => {
                            match error.kind() {
                                PeerErrorKind::Closed => {
                                    debug!("connection {:0>16X} closed", id);
                                }
                                PeerErrorKind::Deserialization(error) => {
                                    error!("error in message stream: {}", error);
                                }
                            }
                            closed_peers.insert(id);
                        }
                    }
                }
                PollEvent::Write(id) => {
                    let peer = server.peer_mut(&id).unwrap();

                    match peer.do_write() {
                        Ok(_) => {
                            reregister_peers.insert(id);
                        }
                        Err(_) => {
                            closed_peers.insert(id);
                        }
                    }
                }
            }
        }

        if end {
            break;
        }

        // Handle new peers
        for id in new_peers.drain() {
            let peer = server.peer(&id).unwrap();
            poller.register_peer(&peer, id)?;
        }

        // Handle timeouts
        let now = Instant::now();
        for (id, peer) in server.peers() {
            if now.duration_since(peer.last_active()) >= peer_timeout {
                warn!("peer {:0>16X} is inactive for too long - closing", id);

                closed_peers.insert(id.clone());
                peer.close();
            }
        }

        // Handle closed peers
        for id in closed_peers.drain() {
            let peer = server.remove_peer(&id).unwrap();
            poller.deregister_peer(&peer, &id)?;

            let mut result = app.handle_offline(&id);
            commands.extend(result.drain(..));
        }

        // Handle incoming messages
        for (id, message) in incoming_messages.drain(..) {
            let mut result = app.handle_message(&id, message);
            commands.extend(result.drain(..));
        }

        // Do a cleanup.
        commands.extend(app.handle_cleanup());

        // If shutdown - handle shutdown
        end = shutdown.load(Ordering::SeqCst);
        if end {
            info!("shutdown requested");
            commands.extend(app.handle_shutdown());
        }

        // Handle commands from app
        for command in commands.drain(..) {
            match command {
                Command::Message(id, message) => {
                    // outgoing message

                    if let Some(peer) = server.peer_mut(&id) {
                        debug!("outgoing message to {:0>16X}: {}", id, message);
                        peer.add_message(&message);
                        reregister_peers.insert(id);
                    }
                }
                Command::Close(id) => {
                    // force close on peer

                    let peer = server.remove_peer(&id).unwrap();
                    peer.close();
                    poller.deregister_peer(&peer, &id)?;
                }
            }
        }

        // Reregister peers if needed.
        for id in reregister_peers.drain() {
            if let Some(peer) = server.peer(&id) {
                poller.reregister_peer(peer, &id)?;
            }
        }
    }

    info!("server terminated");
    Ok(())
}
