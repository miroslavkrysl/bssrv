use std::collections::HashMap;
use crate::types::{SessionKey, Nickname, RestoreState};
use crate::session::Session;
use crate::proto::{ClientMessage, ServerMessage};
use crate::Command;
use crate::Command::Message;
use std::cell::{RefCell, RefMut};
use log::{trace,debug,warn};
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
        match message {
            ClientMessage::Alive => return self.handle_alive(&peer_id),
//            ClientMessage::RestoreSession(session_key) => return self.handle_restore_session(&peer_id, session_key),
            ClientMessage::Login(nickname) => return self.handle_login(&peer_id, nickname),
//            ClientMessage::JoinGame => return self.handle_join_game(&peer_id),
//            ClientMessage::Layout(layout) => {},
//            ClientMessage::Shoot(_) => {},
//            ClientMessage::LeaveGame => {},
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
        debug!("peer {:0>16X}: logging in", peer_id);
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
//
//    /// Handle join game command from the client.
//    fn handle_join_game(&mut self, peer: usize) -> Vec<Command> {
//        debug!("handling joining game for peer {}", peer);
//
//        match self.peers_sessions.get(&peer).cloned() {
//            Some(key) => {
//                // is logged
//                trace!("peer {:0>16X} is logged with session {:0>16X}", peer, key);
//
//                let mut session = self.sessions.get(&key).unwrap().borrow_mut();
//
//                match session.game() {
//                    None => {
//                        // not in any game
//                        trace!("session {:0>16X} is not in any game", key);
//
//                        if let Some(player) = self.pending_player {
//                            if player == key {
//                                warn!("session {:0>16X} is already waiting for a game", key);
//
//                                // but is already waiting for a game
//                                return vec![Message(peer, ServerMessage::IllegalState)];
//                            }
//                        }
//
//                        match self.pending_player {
//                            None => {
//                                // no pending player
//                                trace!("no pending player - session {:0>16X} is set as pending", key);
//
//                                self.pending_player = Some(key);
//
//                                vec![Message(peer, ServerMessage::JoinGameWait)]
//                            },
//                            Some(opponent) => {
//                                // a pending player is present
//                                let game = Game::new(opponent, key);
//                                let id = self.unique_game_id();
//                                self.games.insert(id, game);
//
//                                let mut opponent = self.sessions.get(&opponent).unwrap().borrow_mut();
//                                opponent.set_game(Some(id));
//                                session.set_game(Some(id));
//
//                                trace!("a pending player is present - creating game for session {:0>16X} and {:0>16X}", key, opponent.peer().unwrap());
//
//                                self.pending_player = None;
//
//                                vec![
//                                    Message(opponent.peer().unwrap(), ServerMessage::OpponentJoined(session.nickname().clone())),
//                                    Message(peer, ServerMessage::JoinGameOk(opponent.nickname().clone()))
//                                ]
//                            },
//                        }
//                    },
//                    Some(id) => {
//                        // already in a game
//                        warn!("session {:0>16X} is already in game {}", key, id);
//                        vec![Message(peer, ServerMessage::IllegalState)]
//                    },
//                }
//            }
//            None => {
//                // not logged
//                warn!("peer {} is not logged - can't join a game", peer);
//                vec![Message(peer, ServerMessage::IllegalState)]
//            }
//        }
//    }

    /// Handle logout command from the client.
    fn handle_logout(&mut self, peer_id: &usize) -> Vec<Command> {
        debug!("peer {:0>16X}: logging out", peer_id);

        let mut commands = Vec::new();

        match self.peers_sessions.get(peer_id) {
            None => {
                warn!("can't log out because not logged yet");
                commands.push(Message(*peer_id, ServerMessage::IllegalState))
            },
            Some(session_key) => {
                trace!("session {:0>16X}", session_key);
                let session = self.sessions.get(&session_key);

                // handle if the session is in any game
                match self.sessions_games.get(&session_key) {
                    None => {
                        trace!("not in any game");

                        if self.pending_player.is_some() && self.pending_player.as_ref().unwrap() == session_key {
                            self.pending_player = None;
                            trace!("waiting for a game - removing");
                        }
                    },
                    Some(game_id) => {
                        let game = self.games.remove(&game_id).unwrap();
                        let opponent_id = game.other_player(&session_key);

                        let opponent = self.sessions.get(&opponent_id).unwrap();

                        trace!("in a game {:0>16X} - removing and notifying opponent {}", game_id, opponent_id);

                        self.sessions_games.remove(&session_key);
                        self.sessions_games.remove(&opponent_id);

                        if let Some(opponent_peer_id) = self.sessions_peers.get(&opponent_id) {
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
        debug!("peer {:0>16X}: switching to offline", peer_id);

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
                        let game = self.games.remove(&game_id).unwrap();
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