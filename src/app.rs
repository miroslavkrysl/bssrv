use std::collections::HashMap;
use crate::types::{SessionKey, Nickname, RestoreState};
use crate::session::Session;
use crate::proto::{ClientMessage, ServerMessage};
use crate::Command;
use crate::Command::Message;
use std::cell::{RefCell, RefMut};
use log::{trace,debug,warn, info};
use crate::game::Game;
use rand::Rng;

pub struct App {
    /// Limit of maximum players.
    max_players: usize,
//    /// Sessions map indexed by session keys.
//    storage: Storage,
    /// A player waiting for opponent.
    pending_player: Option<u64>,
    /// Sessions storage indexed by session keys.
    sessions: HashMap<u64, Session>,
    /// Games storage indexed by games ids.
    games: HashMap<usize, Game>,
    /// Session-to-game map indexed by session ids.
    sessions_games: HashMap<u64, usize>,
    /// Peer-to-session map indexed by pees ids.
    peers_sessions: HashMap<usize, u64>,
    /// Session-to-peer map indexed by session ids.
    sessions_peers: HashMap<u64, usize>,
}

impl App {
    pub fn new(max_players: usize) -> Self {
        App {
            max_players,
            pending_player: None,
            sessions: Default::default(),
            games: Default::default(),
            sessions_games: Default::default(),
            peers_sessions: Default::default(),
            sessions_peers: Default::default()
        }
    }

    pub fn handle_message(&mut self, peer_id: &usize, message: ClientMessage) -> Vec<Command> {
        info!("Message from peer {:0>16X}: {}", peer_id, message);

        match message {
            ClientMessage::Alive => return self.handle_alive(&peer_id),
//            ClientMessage::RestoreSession(session_key) => return self.handle_restore_session(&peer_id, session_key),
            ClientMessage::Login(nickname) => return self.handle_login(&peer_id, nickname),
            ClientMessage::JoinGame => return self.handle_join_game(&peer_id),
//            ClientMessage::Layout(layout) => {},
//            ClientMessage::Shoot(_) => {},
            ClientMessage::LeaveGame => return self.handle_leave_game(&peer_id),
            ClientMessage::LogOut => return self.handle_logout(&peer_id),
            _ => return vec![]
        }
    }

    /// Handle the alive command from the client.
    fn handle_alive(&mut self, peer_id: &usize) -> Vec<Command> {
        debug!("peer {:0>16X} is alive", peer_id);

        match self.peers_sessions.get(peer_id) {
            None => {
                trace!("no session")
            },
            Some(session_key) => {
                trace!("with session {:0>16X}", session_key);
                let mut session = self.sessions.get_mut(&session_key).unwrap().update_last_active();
            },
        }

        vec![Message(*peer_id, ServerMessage::AliveOk)]
    }

//    fn handle_restore_session(&mut self, peer: usize, key: SessionKey) -> Vec<Command> {
//        if self.peers_sessions.contains_key(&peer) {
//            // already logged
//            return vec![Message(peer, ServerMessage::IllegalState)];
//        }
//
//        match self.sessions.get(&key.get()) {
//            None => {
//                // no session of given key found
//                vec![Message(peer, ServerMessage::RestoreSessionFail)]
//            },
//            Some(session) => {
//                // a session found
//
//                let mut session = session.borrow_mut();
//
//                if let None = session.peer() {
//                    // session already active
//
//                    return vec![Message(peer, ServerMessage::RestoreSessionFail)];
//                }
//
//                session.update_last_active();
//                session.set_peer(Some(peer));
//
//                // TODO:
//                // if in game -> send game state, notify opponent
//                // if in lobby -> send lobby state
//
//                vec![Message(peer, ServerMessage::RestoreSessionOk(RestoreState::Lobby))]
//            },
//        }
//    }
//
    /// Handle login command from the client.
    fn handle_login(&mut self, peer_id: &usize, nickname: Nickname) -> Vec<Command> {
        debug!("peer {:0>16X} wants to login", peer_id);
        let mut commands = Vec::new();

        match self.peers_sessions.get(peer_id) {
            None => {
                if self.sessions.len() >= self.max_players {
                    warn!("refused because the maximum number of players is reached: {}", self.max_players);
                    commands.push(Message(*peer_id, ServerMessage::LoginFail));
                } else {
                    let session_key = self.unique_session_key();
                    self.sessions.insert(session_key, Session::new(nickname));
                    self.peers_sessions.insert(*peer_id, session_key);
                    self.sessions_peers.insert(session_key, *peer_id);

                    trace!("logged in session {:0>16X}", session_key);

                    commands.push(Message(*peer_id, ServerMessage::LoginOk(SessionKey::new(session_key))))
                }
            },
            Some(_) => {
                warn!("already logged in");
                commands.push(Message(*peer_id, ServerMessage::IllegalState));
            },
        }

        commands
    }

    /// Handle join game command from the client.
    fn handle_join_game(&mut self, peer_id: &usize) -> Vec<Command> {
        debug!("peer {} wants to join a game", peer_id);
        let mut commands = Vec::new();

        match self.peers_sessions.get(peer_id).cloned() {
            Some(session_key) => {
                trace!("logged with session {:0>16X}", session_key);
                self.sessions.get_mut(&session_key).unwrap().update_last_active();

                match self.sessions_games.get(&session_key) {
                    None => {
                        trace!("not in any game");

                        match self.pending_player {
                            None => {
                                trace!("no pending player - set as pending");

                                self.pending_player = Some(session_key);

                                commands.push(Message(*peer_id, ServerMessage::JoinGameWait))
                            },
                            Some(opponent_session_key) => {
                                if opponent_session_key == session_key {
                                    warn!("already waiting for a game");

                                    commands.push(Message(*peer_id, ServerMessage::IllegalState));
                                } else {
                                    let game = Game::new(opponent_session_key, session_key);
                                    let game_id = self.unique_game_id();
                                    self.games.insert(game_id, game);

                                    self.sessions_games.insert(session_key, game_id);
                                    self.sessions_games.insert(opponent_session_key, game_id);

                                    trace!("a pending player {:0>16X} is present - creating game", opponent_session_key);

                                    self.pending_player = None;

                                    let opponent_peer_id = self.sessions_peers.get(&opponent_session_key).unwrap();
                                    let session = self.sessions.get(&session_key).unwrap();
                                    let opponent = self.sessions.get(&opponent_session_key).unwrap();

                                    commands.push(Message(*opponent_peer_id, ServerMessage::OpponentJoined(session.nickname().clone())));
                                    commands.push(Message(*peer_id, ServerMessage::JoinGameOk(opponent.nickname().clone())));
                                }
                            },
                        }
                    },
                    Some(game_id) => {
                        warn!("already in game {}", game_id);
                        commands.push(Message(*peer_id, ServerMessage::IllegalState));
                    },
                }
            }
            None => {
                warn!("not logged - can't join a game");
                commands.push(Message(*peer_id, ServerMessage::IllegalState))
            }
        }

        return commands
    }

    fn handle_leave_game(&mut self, peer_id: &usize) -> Vec<Command> {
        debug!("peer {} wants to leave the game", peer_id);
        let mut commands = Vec::new();

        match self.peers_sessions.get(peer_id).cloned() {
            Some(session_key) => {
                trace!("logged with session {:0>16X}", session_key);
                self.sessions.get_mut(&session_key).unwrap().update_last_active();

                match self.sessions_games.get(&session_key) {
                    None => {
                        trace!("not in game");

                        match self.pending_player {
                            None => {
                                trace!("not waiting for a game");
                                warn!("can't leave - not in any a game");

                                commands.push(Message(*peer_id, ServerMessage::IllegalState))
                            },
                            Some(pending_session_key) => {
                                if pending_session_key == session_key {
                                    trace!("waiting for a game - removing");

                                    self.pending_player = None;

                                    commands.push(Message(*peer_id, ServerMessage::LeaveGameOk));
                                }
                            },
                        }
                    },
                    Some(game_id) => {
                        trace!("in game {} - removing game and notifying opponent", game_id);

                        let game = self.games.get(game_id).unwrap();
                        let opponent_session_key = &game.other_player(&session_key);

                        self.sessions_games.remove(&session_key);
                        self.sessions_games.remove(opponent_session_key);

                        if let Some(opponent_peer_id) = self.sessions_peers.get(&opponent_session_key) {
                            commands.push(Message(*opponent_peer_id, ServerMessage::OpponentLeft))
                        }

                        commands.push(Message(*peer_id, ServerMessage::LeaveGameOk));
                    },
                }
            }
            None => {
                warn!("not logged - can't join a game");
                commands.push(Message(*peer_id, ServerMessage::IllegalState))
            }
        }

        return commands
    }

    /// Handle logout command from the client.
    fn handle_logout(&mut self, peer_id: &usize) -> Vec<Command> {
        debug!("peer {:0>16X} wants to logout", peer_id);

        let mut commands = Vec::new();

        match self.peers_sessions.get(peer_id).cloned() {
            None => {
                warn!("can't log out because not logged yet");
                commands.push(Message(*peer_id, ServerMessage::IllegalState))
            },
            Some(session_key) => {
                trace!("session {:0>16X}", session_key);

                // handle if the session is in any game
                match self.sessions_games.get(&session_key) {
                    None => {
                        trace!("not in any game");

                        if let Some(player) = self.pending_player {
                            self.pending_player = None;
                            trace!("waiting for a game - removing");
                        }
                    },
                    Some(game_id) => {
                        let game = self.games.remove(&game_id).unwrap();
                        let opponent_session_key = game.other_player(&session_key);

                        trace!("in a game {:0>16X} - removing and notifying opponent {}", game_id, opponent_session_key);

                        self.sessions_games.remove(&session_key);
                        self.sessions_games.remove(&opponent_session_key);

                        if let Some(opponent_peer_id) = self.sessions_peers.get(&opponent_session_key) {
                            commands.push(Message(*opponent_peer_id, ServerMessage::OpponentLeft))
                        }
                    },
                }

                self.sessions.remove(&session_key);
                self.sessions_peers.remove(&session_key);
                self.peers_sessions.remove(&peer_id);

                commands.push(Message(*peer_id, ServerMessage::LogoutOk));
            },
        }

        commands
    }

    /// Handle the peer socket disconnection.
    pub fn handle_offline(&mut self, peer_id: &usize) -> Vec<Command> {
        debug!("peer {:0>16X} switching to offline", peer_id);

        let mut commands = Vec::new();

        match self.peers_sessions.get(&peer_id) {
            None => {
                trace!("no session");
            },
            Some(session_key) => {
                trace!("session {:0>16X}", session_key);

                // handle if the session is in any game
                match self.sessions_games.get(&session_key) {
                    None => {
                        trace!("not in any game");

                        if let Some(player) = self.pending_player {
                            if player == *session_key {
                                trace!("waiting for a game - removing");
                                self.pending_player = None;
                            }
                        }
                    },
                    Some(game_id) => {
                        let game = self.games.get(&game_id).unwrap();
                        let opponent_session_key = game.other_player(&session_key);

                        trace!("in a game {:0>16X} - notifying opponent {:0>16X}", game_id, &opponent_session_key);

                        if let Some(opponent_peer_id) = self.sessions_peers.get(&opponent_session_key) {
                            commands.push(Message(*opponent_peer_id, ServerMessage::OpponentOffline))
                        }
                    },
                }

                self.sessions_peers.remove(&session_key);
                self.peers_sessions.remove(&peer_id);
            },
        }

        commands
    }

        /// Get a unique id for a session.
    fn unique_session_key(&self) -> u64 {
        loop {
            let key = rand::thread_rng().gen();
            if !self.sessions.contains_key(&key) {
                break key;
            }
        }
    }

    /// Get a unique id for a game.
    fn unique_game_id(&self) -> usize {
        loop {
            let id = rand::thread_rng().gen();
            if !self.games.contains_key(&id) {
                break id;
            }
        }
    }
}