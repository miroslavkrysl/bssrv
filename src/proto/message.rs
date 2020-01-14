//! Battleships protocol message types,
//! And payload container.

use crate::types::{SessionKey, Nickname, Layout, Position, RestoreState, ShipKind};
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
    ShootSunk(ShipKind, Position),
    LeaveGameOk,
    DisconnectOk,
    Disconnect,
    OpponentJoined(Nickname),
    OpponentReady,
    OpponentLeft,
    OpponentMissed(Position),
    OpponentHit(Position)
}
