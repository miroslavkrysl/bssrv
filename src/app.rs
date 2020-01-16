use std::collections::HashMap;
use crate::types::{SessionKey, Nickname, RestoreState};
use crate::session::Session;
use crate::proto::{ClientMessage, ServerMessage};
use crate::Command;
use crate::Command::Message;
use rand::Rng;
use std::cell::{RefCell};
use crate::game::Game;
use log::{trace,debug,warn};

pub struct App {
    /// Limit of maximum players.
    max_players: usize,
    /// Sessions map indexed by session keys.
    sessions: HashMap<u64, RefCell<Session>>,
    /// Games map indexed by games ids.
    games: HashMap<usize, Game>,
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

    pub fn handle_message(&mut self, peer: usize, message: ClientMessage) -> Vec<Command> {
        match message {
            ClientMessage::Alive => return self.handle_alive(peer),
            ClientMessage::RestoreSession(session_key) => return self.handle_restore_session(peer, session_key),
            ClientMessage::Login(nickname) => return self.handle_login(peer, nickname),
            ClientMessage::JoinGame => return self.handle_join_game(peer),
//            ClientMessage::Layout(layout) => {},
//            ClientMessage::Shoot(_) => {},
//            ClientMessage::LeaveGame => {},
            ClientMessage::LogOut => return self.handle_logout(peer),
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
    fn handle_logout(&mut self, peer: usize) -> Vec<Command> {
        if let Some(key) = self.peers_sessions.remove(&peer) {
            // is logged

            let mut cmds = Vec::new();

            {
                let mut session = self.sessions.get(&key).unwrap().borrow_mut();

                match session.game() {
                    None => {
                        // not in any game

                        if let Some(player) = self.pending_player {
                            if player == key {
                                // but is already waiting for a game
                                self.pending_player = None;
                            }
                        }
                    },
                    Some(id) => {
                        // in a game

                        let game = self.games.remove(&id).unwrap();
                        let mut opponent = self.sessions.get(&game.other_player(key)).unwrap().borrow_mut();

                        opponent.set_game(None);
                        session.set_game(None);

                        if let Some(opponent_peer) = opponent.peer() {
                            cmds.push(Message(opponent_peer, ServerMessage::OpponentLeft))
                        }
                    },
                }
            }

            self.sessions.remove(&key);
            cmds.push(Message(peer, ServerMessage::LogoutOk));
            cmds
        } else {
            vec![Message(peer, ServerMessage::IllegalState)]
        }
    }




    fn unique_session_key(&self) -> u64 {
        loop {
            let key = rand::thread_rng().gen();
            if !self.sessions.contains_key(&key) {
                break key;
            }
        }
    }

    fn unique_game_id(&self) -> usize {
        loop {
            let id = rand::thread_rng().gen();
            if !self.games.contains_key(&id) {
                break id;
            }
        }
    }
}