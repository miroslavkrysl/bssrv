use std::collections::HashMap;
use crate::types::{SessionKey, Nickname, RestoreState};
use crate::session::Session;
use crate::proto::{ClientMessage, ServerMessage};
use crate::Command;
use crate::Command::Message;
use rand::Rng;
use std::cell::{RefCell};
use crate::game::Game;

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


    fn handle_join_game(&mut self, peer: usize) -> Vec<Command> {
        match self.peers_sessions.get(&peer).cloned() {
            Some(key) => {
                // is logged

                let mut session = self.sessions.get(&key).unwrap().borrow_mut();

                match session.game() {
                    None => {
                        // not in any game

                        if let Some(player) = self.pending_player {
                            if player == key {
                                // but is already waiting for a game
                                return vec![Message(peer, ServerMessage::JoinGameWait)];
                            }
                        }

                        match self.pending_player {
                            None => {
                                // no pending player
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

                                vec![
                                    Message(opponent.peer().unwrap(), ServerMessage::OpponentJoined(session.nickname().clone())),
                                    Message(peer, ServerMessage::JoinGameOk(opponent.nickname().clone()))
                                ]
                            },
                        }
                    },
                    Some(_) => {
                        // already in a game
                        vec![Message(peer, ServerMessage::IllegalState)]
                    },
                }
            }
            None => {
                // not logged
                vec![Message(peer, ServerMessage::IllegalState)]
            }
        }
    }

    fn handle_logout(&mut self, peer: usize) -> Vec<Command> {
        if let Some(key) = self.peers_sessions.remove(&peer) {
            // is logged

            self.sessions.remove(&key);

            // TODO:
            // if in game -> remove game -> notify opponent
            // if in lobby -> send lobby state

            vec![Message(peer, ServerMessage::LogoutOk)]
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