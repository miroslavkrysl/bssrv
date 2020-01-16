use std::collections::HashMap;
use crate::types::{SessionKey, Nickname, RestoreState};
use crate::session::Session;
use crate::game::Game;
use crate::proto::{ClientMessage, ServerMessage};
use crate::Command;
use crate::Command::Message;
use rand::Rng;

pub struct App {
    /// Sessions map indexed by session key
    sessions: HashMap<u64, Session>,
    /// Games map indexed by game id
    games: HashMap<usize, Game>,
    /// Sessions map indexed by peers id.
    peers_sessions: HashMap<usize, u64>,
    /// A pending game with only one player
    pending_game: Option<usize>
}

impl App {
    pub fn new() -> Self {
        App {
            sessions: HashMap::new(),
            games: HashMap::new(),
            peers_sessions: HashMap::new(),
            pending_game: None
        }
    }

    pub fn handle_message(&mut self, id: usize, message: ClientMessage) -> Vec<Command> {
        match message {
            ClientMessage::Alive => return self.handle_alive(id),
            ClientMessage::RestoreSession(session_key) => return self.handle_restore_session(id, session_key),
            ClientMessage::Login(nickname) => return self.handle_login(id, nickname),
//            ClientMessage::JoinGame => return self.handle_join_game(id),
//            ClientMessage::Layout(_) => {},
//            ClientMessage::Shoot(_) => {},
//            ClientMessage::LeaveGame => {},
            ClientMessage::LogOut => return self.handle_logout(id),
            _ => return vec![]
        }
    }

    /// Handle the alive command from the client.
    fn handle_alive(&mut self, id: usize) -> Vec<Command> {
        if let Some(session_key) = self.peers_sessions.get_mut(&id) {
            let session = self.sessions.get_mut(session_key).unwrap();
            session.update_last_active();
        }

        vec![Message(id, ServerMessage::AliveOk)]
    }

    /// Handle the restore session command from the client.
    fn handle_restore_session(&mut self, id: usize, key: SessionKey) -> Vec<Command> {
        if self.peers_sessions.contains_key(&id) {
            // already logged
            return vec![Message(id, ServerMessage::IllegalState)];
        }

        if let Some(session) = self.sessions.get_mut(&key.get()) {
            // a session found
            session.update_last_active();
            session.set_peer(Some(id));

            // TODO: game restore state + notify opponent

            vec![Message(id, ServerMessage::RestoreSessionOk(RestoreState::Lobby))]
        } else {
            // no session of given key found
            vec![Message(id, ServerMessage::RestoreSessionFail)]
        }
    }

    fn handle_login(&mut self, id:usize, nickname: Nickname) -> Vec<Command> {
        if self.peers_sessions.contains_key(&id) {
            return vec![Message(id, ServerMessage::IllegalState)];
        }

        let key = self.unique_session_key();
        self.sessions.insert(key, Session::new(key, nickname));
        self.peers_sessions.insert(id, key);

        vec![Message(id, ServerMessage::LoginOk(SessionKey::new(key)))]
    }

    fn handle_logout(&mut self, id:usize) -> Vec<Command> {
        if let Some(key) = self.peers_sessions.remove(&id) {
            self.sessions.remove(&key);
            vec![Message(id, ServerMessage::LogoutOk)]
        } else {
            vec![Message(id, ServerMessage::IllegalState)]
        }
    }

    fn unique_session_key(&self) -> u64 {
        loop {
            let key = rand::thread_rng().gen();
            if !self.sessions.contains_key(&key) {
                break key
            }
        }
    }
}