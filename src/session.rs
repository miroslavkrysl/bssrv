use std::time::Instant;
use crate::types::Nickname;

pub struct Session {
    key: u64,
    nickname: Nickname,
    peer: Option<usize>,
    game: Option<usize>,
    last_active: Instant
}

impl Session {
    pub fn new(key: u64, nickname: Nickname, peer: usize) -> Self {
        Session {
            key,
            nickname,
            peer: Some(peer),
            game: None,
            last_active: Instant::now()
        }
    }

    pub fn update_last_active(&mut self) {
        self.last_active = Instant::now()
    }

    pub fn nickname(&self) -> &Nickname {
        &self.nickname
    }

    pub fn peer(&self) -> Option<usize> {
        self.peer
    }

    pub fn set_peer(&mut self, id: Option<usize>) {
        self.peer = id
    }

    pub fn game(&self) -> Option<usize> {
        self.game
    }

    pub fn set_game(&mut self, id: Option<usize>) {
        self.game = id
    }
}