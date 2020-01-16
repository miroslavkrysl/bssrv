use std::collections::HashMap;
use crate::types::{SessionKey, Nickname, RestoreState};
use crate::session::Session;
use crate::proto::{ClientMessage, ServerMessage};
use crate::Command;
use crate::Command::Message;
use rand::Rng;
use std::cell::{RefCell, RefMut};
use crate::game::Game;
use log::{trace,debug,warn};
use std::borrow::BorrowMut;

pub struct App {
    /// Limit of maximum players.
    max_players: usize,
    /// Sessions map indexed by session keys.
    sessions: HashMap<u64, RefCell<Session>>,
    /// Games map indexed by games ids.
    games: HashMap<usize, RefCell<Game>>,
    /// Sessions map indexed by peers ids.
    peers_sessions: HashMap<usize, u64>,
    /// A player waiting for opponent.
    pending_player: Option<u64>,
}

impl App {
    pub fn new(max_players: usize) -> Self {
        App {
            max_players,
            sessions: HashMap::new(),
            games: HashMap::new(),
            peers_sessions: HashMap::new(),
            pending_player: None,
        }
    }

    pub fn handle_message(&mut self, peer: &usize, message: ClientMessage) -> Vec<Command> {
        match message {
            ClientMessage::Alive => return self.handle_alive(&peer),
            ClientMessage::RestoreSession(session_key) => return self.handle_restore_session(&peer, session_key),
            ClientMessage::Login(nickname) => return self.handle_login(&peer, nickname),
            ClientMessage::JoinGame => return self.handle_join_game(&peer),
//            ClientMessage::Layout(layout) => {},
//            ClientMessage::Shoot(_) => {},
//            ClientMessage::LeaveGame => {},
            ClientMessage::LogOut => return self.handle_logout(&peer),
            _ => return vec![]
        }
    }

    /// Handle the alive command from the client.
    fn handle_alive(&mut self, peer: usize) -> Vec<Command> {
        if let Some(session_key) = self.peers_sessions.get(&peer) {
            let mut session = self.sessions.get(session_key).unwrap().borrow_mut();
            session.update_last_active();
        }

        vec![Message(peer, ServerMessage::AliveOk)]
    }

    fn handle_restore_session(&mut self, peer: usize, key: SessionKey) -> Vec<Command> {
        if self.peers_sessions.contains_key(&peer) {
            // already logged
            return vec![Message(peer, ServerMessage::IllegalState)];
        }

        match self.sessions.get(&key.get()) {
            None => {
                // no session of given key found
                vec![Message(peer, ServerMessage::RestoreSessionFail)]
            },
            Some(session) => {
                // a session found

                let mut session = session.borrow_mut();

                if let None = session.peer() {
                    // session already active

                    return vec![Message(peer, ServerMessage::RestoreSessionFail)];
                }

                session.update_last_active();
                session.set_peer(Some(peer));

                // TODO:
                // if in game -> send game state, notify opponent
                // if in lobby -> send lobby state

                vec![Message(peer, ServerMessage::RestoreSessionOk(RestoreState::Lobby))]
            },
        }
    }

    /// Handle login command from the client.
    fn handle_login(&mut self, peer: usize, nickname: Nickname) -> Vec<Command> {
        if self.peers_sessions.contains_key(&peer) {
            // already logged

            return vec![Message(peer, ServerMessage::IllegalState)];
        } else {
            if self.sessions.len() >= self.max_players {
                return vec![Message(peer, ServerMessage::LoginFail)]
            }

            let key = self.unique_session_key();
            self.sessions.insert(key, Session::new(key, nickname, peer).into());
            self.peers_sessions.insert(peer, key);

            vec![Message(peer, ServerMessage::LoginOk(SessionKey::new(key)))]
        }
    }

    /// Handle join game command from the client.
    fn handle_join_game(&mut self, peer: usize) -> Vec<Command> {
        debug!("handling joining game for peer {}", peer);

        match self.peers_sessions.get(&peer).cloned() {
            Some(key) => {
                // is logged
                trace!("peer {} is logged with session {:0>16X}", peer, key);

                let mut session = self.sessions.get(&key).unwrap().borrow_mut();

                match session.game() {
                    None => {
                        // not in any game
                        trace!("session {:0>16X} is not in any game", key);

                        if let Some(player) = self.pending_player {
                            if player == key {
                                warn!("session {:0>16X} is already waiting for a game", key);

                                // but is already waiting for a game
                                return vec![Message(peer, ServerMessage::IllegalState)];
                            }
                        }

                        match self.pending_player {
                            None => {
                                // no pending player
                                trace!("no pending player - session {:0>16X} is set as pending", key);

                                self.pending_player = Some(key);

                                vec![Message(peer, ServerMessage::JoinGameWait)]
                            },
                            Some(opponent) => {
                                // a pending player is present
                                let game = Game::new(opponent, key);
                                let id = self.unique_game_id();
                                self.games.insert(id, game);

                                let mut opponent = self.sessions.get(&opponent).unwrap().borrow_mut();
                                opponent.set_game(Some(id));
                                session.set_game(Some(id));

                                trace!("a pending player is present - creating game for session {:0>16X} and {:0>16X}", key, opponent.peer().unwrap());

                                self.pending_player = None;

                                vec![
                                    Message(opponent.peer().unwrap(), ServerMessage::OpponentJoined(session.nickname().clone())),
                                    Message(peer, ServerMessage::JoinGameOk(opponent.nickname().clone()))
                                ]
                            },
                        }
                    },
                    Some(id) => {
                        // already in a game
                        warn!("session {:0>16X} is already in game {}", key, id);
                        vec![Message(peer, ServerMessage::IllegalState)]
                    },
                }
            }
            None => {
                // not logged
                warn!("peer {} is not logged - can't join a game", peer);
                vec![Message(peer, ServerMessage::IllegalState)]
            }
        }
    }

    /// Handle logout command from the client.
    fn handle_logout(&mut self, peer_id: &usize) -> Vec<Command> {
        debug!("peer {:0>16X}: logging out", peer_id);

        let mut commands = Vec::new();

        match self.peer_session(&peer_id) {
            None => {
                warn!("peer {:0>16X}: can't log out because not logged yet", peer_id);
                commands.push(Message(*peer_id, ServerMessage::IllegalState))
            },
            Some(session_id) => {
                trace!("peer {:0>16X}: session {:0>16X}", peer_id, session_id);
                let mut commands = Vec::new();
                let mut session = self.session(session_id);

                // handle if the session is in any game
                match session.game() {
                    None => {
                        trace!("session {:0>16X}: not in any game", session_id);

                        if let Some(player) = self.pending_player {
                            if player == *session_id {
                                trace!("session {:0>16X}: waiting for a game - removing", session_id);
                                self.pending_player = None;
                            }
                        }
                    },
                    Some(game_id) => {
                        trace!("session {:0>16X}: in a game {:0>16X} - removing", session_id, game_id);

                        let game = self.remove_game(game_id);
                        let mut opponent = self.session(&game.other_player(session_id));

                        opponent.set_game(None);
                        session.set_game(None);

                        if let Some(opponent_peer_id) = opponent.peer() {
                            commands.push(Message(*opponent_peer_id, ServerMessage::OpponentLeft))
                        }
                    },
                }

                self.remove_session(session_id);
                self.remove_peer_session(peer_id);
                commands.push(Message(*peer_id, ServerMessage::LogoutOk));
            },
        }

        commands
    }

    /// Handle the peer socket disconnection.
    pub fn handle_offline(&mut self, peer_id: &usize) -> Vec<Command> {
        debug!("peer {:0>16X}: switching to offline", peer_id);

        let mut commands = Vec::new();

        match self.peer_session(&peer_id) {
            None => {
                trace!("no session");
            },
            Some(session_id) => {
                trace!("session {:0>16X}", peer_id);
                let mut session = self.session(session_id);

                // handle if the session is in any game
                match session.game() {
                    None => {
                        trace!("not in any game");

                        if let Some(player) = self.pending_player {
                            if player == *session_id {
                                trace!("waiting for a game - removing");
                                self.pending_player = None;
                            }
                        }
                    },
                    Some(game_id) => {
                        let game = self.remove_game(game_id);
                        let mut opponent = self.session(&game.other_player(session_id));

                        trace!("in a game {:0>16X} - notifying opponent {:0>16X}", game_id, opponent.key());

                        if let Some(opponent_peer_id) = opponent.peer() {
                            commands.push(Message(*opponent_peer_id, ServerMessage::OpponentOffline))
                        }
                    },
                }

                session.set_peer(None);
            },
        }

        commands
    }

    fn peer_session(&self, peer_id: &usize) -> Option<&u64> {
        self.peers_sessions.get(peer_id)
    }

    fn remove_peer_session(&mut self, peer_id: &usize) -> u64 {
        self.peers_sessions.remove(peer_id).unwrap()
    }

    fn game(&self, game_id: &usize) -> RefMut<Game> {
        self.games.get(game_id).unwrap().borrow_mut()
    }

    fn remove_game(&mut self, game_id: &usize) -> Game {
        self.games.remove(game_id).unwrap().into_inner()
    }

    fn session(&self, session_key: &u64) -> RefMut<Session> {
        self.sessions.get(session_key).unwrap().borrow_mut()
    }

    fn remove_session(&mut self, session_key: &u64) -> Session {
        self.sessions.remove(session_key).unwrap().into_inner()
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