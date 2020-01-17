use std::time::Instant;
use crate::types::Nickname;

pub struct Session {
    nickname: Nickname,
    last_active: Instant
}

impl Session {
    pub fn new(nickname: Nickname) -> Self {
        Session {
            nickname,
            last_active: Instant::now()
        }
    }

    pub fn update_last_active(&mut self) {
        self.last_active = Instant::now()
    }

    pub fn last_active(&self) -> &Instant {
        &self.last_active
    }

    pub fn nickname(&self) -> &Nickname {
        &self.nickname
    }
}
