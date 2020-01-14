//! Battleships protocol message types.

use crate::types::{SessionKey, Nickname, Layout, Position};
use std::fmt::{Formatter, Display};
use std::fmt;

/// A message received from a client.
#[derive(Debug, Clone)]
pub enum ClientMessage {
    Alive,
    RestoreSession(SessionKey),
    Login(Nickname),
    JoinGame,
    Layout(Layout),
    Shoot(Position),
    LeaveGame,
    Disconnect,
}

impl Display for ClientMessage {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), fmt::Error> {
        match self {
            ClientMessage::Alive => write!(f, "[alive]"),
            ClientMessage::RestoreSession(session_key) => write!(f, "[restore session: {}]", session_key),
            ClientMessage::Login(nickname) => write!(f, "[login: {}]", nickname),
            ClientMessage::JoinGame => write!(f, "[join game]"),
            ClientMessage::Layout(layout) => write!(f, "[layout: {}]", layout),
            ClientMessage::Shoot(position) => write!(f, "[shoot: {}]", position),
            ClientMessage::LeaveGame => write!(f, "[leave game]"),
            ClientMessage::Disconnect => write!(f, "[disconnect]"),
        }
    }
}


pub enum ServerMessage {
    IllegalState,

    AliveOk,

    RestoreSessionOk(RestoreState),
    RestoreSessionFail,

    LoginOk(SessionKey),
    LoginOkFail,

    JoinGameWait,
    JoinGameOk(Nickname),

    LayoutOk,
    LayoutFail,

    ShootHit,
    ShootMiss,
    ShootSunk(Position),

    LeaveGameOk,

    DisconnectOk,

    Disconnect,
    OpponentJoined(Nickname),
    OpponentReady,
    OpponentLeft,
    OpponentMissed(Position),
    OpponentHit(Position)
}

//impl ServerMessage {
//    pub fn deserialize(serialized: &str) -> Self {
//        // deserialize header
//        let payload_start = find(serialized, 0, PAYLOAD_START, ESCAPE);
//
//        let header;
//        let mut payload = None;
//
//        match payload_start {
//            None => {
//                // no payload
//                header = serialized;
//            }
//            Some(i) => {
//                // some payload
//                header = &serialized[..i];
//                payload = Some(&serialized[(i + 1)..]);
//            }
//        }
//
//        match header {
//            "illegal_state" => ServerMessage::IllegalState,
//            "alive_ok" => ServerMessage::,
//
//            RestoreSessionOk(RestoreState),
//            RestoreSessionFail,
//
//            LoginOk(SessionKey),
//            LoginOkFail,
//
//            JoinGameWait,
//            JoinGameOk(Nickname),
//
//            LayoutOk,
//            LayoutFail,
//
//            ShootHit,
//            ShootMiss,
//            ShootSunk(Position),
//
//            LeaveGameOk,
//
//            DisconnectOk,
//
//            Disconnect,
//            OpponentJoined(Nickname),
//            OpponentReady,
//            OpponentLeft,
//            OpponentMissed(Position),
//            OpponentHit(Position),
//            GameOver(Who),
//        }
//    }
//}