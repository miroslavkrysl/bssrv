//! Battleships protocol message types,
//! And payload container.

use crate::types::{Nickname, Layout, Position, RestoreState, ShipKind, Who, Placement};
use std::fmt::{Formatter, Display};
use std::fmt;
use std::panic::resume_unwind;

/// A message received from a client.
#[derive(Debug, Clone)]
pub enum ClientMessage {
    Alive,
    Login(Nickname),
    JoinGame,
    Layout(Layout),
    Shoot(Position),
    LeaveGame,
    LogOut,
}

impl Display for ClientMessage {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), fmt::Error> {
        match self {
            ClientMessage::Alive =>
                write!(f, "[alive]"),
            ClientMessage::Login(nickname) =>
                write!(f, "[login: {}]", nickname),
            ClientMessage::JoinGame =>
                write!(f, "[join game]"),
            ClientMessage::Layout(layout) =>
                write!(f, "[layout: {}]", layout),
            ClientMessage::Shoot(position) =>
                write!(f, "[shoot: {}]", position),
            ClientMessage::LeaveGame =>
                write!(f, "[leave game]"),
            ClientMessage::LogOut =>
                write!(f, "[logout]"),
        }
    }
}

/// A message sending to a client.
pub enum ServerMessage {
    IllegalState,
    AliveOk,
    LoginOk,
    LoginRestored(RestoreState),
    LoginFull,
    LoginTaken,
    JoinGameWait,
    JoinGameOk(Nickname),
    LayoutOk,
    LayoutFail,
    ShootHit,
    ShootMissed,
    ShootSunk(ShipKind, Placement),
    LeaveGameOk,
    LogoutOk,
    Disconnect,
    OpponentJoined(Nickname),
    OpponentReady,
    OpponentOffline,
    OpponentLeft,
    OpponentMissed(Position),
    OpponentHit(Position),
    GameOver(Who),
}

impl Display for ServerMessage {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), fmt::Error> {
        match self {
            ServerMessage::IllegalState =>
                write!(f, "[illegal state]"),
            ServerMessage::AliveOk =>
                write!(f, "[alive ok]"),
            ServerMessage::LoginOk =>
                write!(f, "[login ok]"),
            ServerMessage::LoginRestored(restore_state) =>
                write!(f, "[login restored: {}]", restore_state),
            ServerMessage::LoginFull =>
                write!(f, "[login full]"),
            ServerMessage::LoginTaken =>
                write!(f, "[login taken]"),
            ServerMessage::JoinGameWait =>
                write!(f, "[join game wait]"),
            ServerMessage::JoinGameOk(opponent) =>
                write!(f, "[join game ok: {}]", opponent),
            ServerMessage::LayoutOk =>
                write!(f, "[layout ok]"),
            ServerMessage::LayoutFail =>
                write!(f, "[layout fail]"),
            ServerMessage::ShootHit =>
                write!(f, "[shoot hit]"),
            ServerMessage::ShootMissed =>
                write!(f, "[shoot missed]"),
            ServerMessage::ShootSunk(kind, placement) =>
                write!(f, "[shoot sunk: {}, {}]", kind, placement),
            ServerMessage::LeaveGameOk =>
                write!(f, "[leave game ok]"),
            ServerMessage::LogoutOk =>
                write!(f, "[logout ok]"),
            ServerMessage::Disconnect =>
                write!(f, "[disconnect]"),
            ServerMessage::OpponentJoined(opponent) =>
                write!(f, "[opponent joined: {}]", opponent),
            ServerMessage::OpponentReady =>
                write!(f, "[opponent ready]"),
            ServerMessage::OpponentOffline =>
                write!(f, "[opponent offline]"),
            ServerMessage::OpponentLeft =>
                write!(f, "[opponent left]"),
            ServerMessage::OpponentMissed(position) =>
                write!(f, "[opponent missed: {}]", position),
            ServerMessage::OpponentHit(position) =>
                write!(f, "[opponent hit: {}]", position),
            ServerMessage::GameOver(winner) =>
                write!(f, "[game over: {}]", winner),
        }
    }
}