use std::time::Instant;
use crate::types::Nickname;
use std::fmt::{Display, Formatter, Error};

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

    pub fn nickname(&self) -> &Nickname {
        &self.nickname
    }
}
