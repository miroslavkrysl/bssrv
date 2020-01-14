//! Client messages deserialization logic

use crate::types::{SessionKey, Nickname, Layout, Position, Placement, Orientation, DomainErrorKind};
use crate::proto::ClientMessage;
use std::fmt::{Display, Formatter};
use std::fmt;
use std::error::Error;
use std::num::ParseIntError;
use crate::proto::codec::{find, Payload, PAYLOAD_START, ESCAPE};

// ---ERRORS---

/// Describes the kind of the deserialization error.
#[derive(Debug, Eq, PartialEq)]
pub enum DeserializeErrorKind {
    UnknownHeader,
    NoMorePayloadItems,
    InvalidEnumValue,
    IntError(ParseIntError),
    StructError(StructDeserializeError),
}

impl Display for DeserializeErrorKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), fmt::Error> {
        match self {
            DeserializeErrorKind::UnknownHeader => write!(f, "Unknown header."),
            DeserializeErrorKind::NoMorePayloadItems => write!(f, "Further payload item was expected, but not present."),
            DeserializeErrorKind::InvalidEnumValue => write!(f, "Invalid enum value."),
            DeserializeErrorKind::IntError(ref error) => write!(f, "Integer can't be properly deserialized: {}", error),
            DeserializeErrorKind::StructError(ref error) => write!(f, "{}", error),
        }
    }
}

impl DeserializeError {
    /// Create new deserialization error of given kind.
    pub fn new(kind: DeserializeErrorKind) -> Self {
        DeserializeError {
            kind
        }
    }
}

/// An error indicating that a value is out of its domain.
#[derive(Debug, Eq, PartialEq)]
pub struct DeserializeError {
    /// Kind of deserialization error.
    kind: DeserializeErrorKind
}

impl Display for DeserializeError {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), fmt::Error> {
        write!(f, "Deserialization error: {}", self.kind)
    }
}

impl From<ParseIntError> for DeserializeError {
    fn from(error: ParseIntError) -> Self {
        DeserializeError::new(DeserializeErrorKind::IntError(error))
    }
}

impl From<StructDeserializeError> for DeserializeError {
    fn from(error: StructDeserializeError) -> Self {
        DeserializeError::new(DeserializeErrorKind::StructError(error))
    }
}

impl Error for DeserializeError {}

/// Describes the kind of the struct deserialization error.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum StructDeserializeErrorKind {
    SessionKeyError,
    NicknameError,
    ShipIdError,
    PositionError,
    OrientationError,
    PlacementError,
    LayoutError,
}

impl Display for StructDeserializeErrorKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), fmt::Error> {
        match self {
            StructDeserializeErrorKind::SessionKeyError =>
                write!(f, "SessionKey can't be properly deserialized"),
            StructDeserializeErrorKind::NicknameError =>
                write!(f, "Nickname can't be properly deserialized"),
            StructDeserializeErrorKind::ShipIdError =>
                write!(f, "ShipId can't be properly deserialized"),
            StructDeserializeErrorKind::PositionError =>
                write!(f, "Position can't be properly deserialized"),
            StructDeserializeErrorKind::OrientationError =>
                write!(f, "Orientation can't be properly deserialized"),
            StructDeserializeErrorKind::PlacementError =>
                write!(f, "Placement can't be properly deserialized"),
            StructDeserializeErrorKind::LayoutError =>
                write!(f, "Layout can't be properly deserialized"),
        }
    }
}

/// An error indicating that a value is out of its domain.
#[derive(Debug)]
pub struct StructDeserializeError {
    /// Kind of deserialization error.
    kind: StructDeserializeErrorKind,

    /// Cause of the error.
    error: Box<dyn Error>
}

impl StructDeserializeError {
    /// Create new struct deserialization error of given kind and cause.
    fn new(kind: StructDeserializeErrorKind, cause: Box<dyn Error>) -> Self {
        StructDeserializeError {
            kind,
            error: cause
        }
    }
}

impl PartialEq for StructDeserializeError {
    fn eq(&self, other: &Self) -> bool {
        self.kind == other.kind
    }
}

impl Eq for StructDeserializeError {}

impl Display for StructDeserializeError {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), fmt::Error> {
        write!(f, "{}: {}", self.kind, self.error)
    }
}

impl Error for StructDeserializeError {}


// ---DESERIALIZATION---

impl ClientMessage {
    pub fn deserialize(serialized: &str) -> Result<Self, DeserializeError> {
        // deserialize header
        let payload_start = find(serialized, 0, PAYLOAD_START, ESCAPE);

        let header;
        let mut payload;

        match payload_start {
            None => {
                // no payload
                header = serialized;
                payload = Payload::empty();
            }
            Some(i) => {
                // some payload
                header = &serialized[..i];
                payload = Payload::deserialize(&serialized[(i + 1)..]);
            }
        }

        match header {
            "alive" => Ok(ClientMessage::Alive),
            "restore_session" => {
                let session_key = SessionKey::deserialize(&mut payload)?;
                Ok(ClientMessage::RestoreSession(session_key))
            },
            "login" => {
                let nickname = Nickname::deserialize(&mut payload)?;
                Ok(ClientMessage::Login(nickname))
            },
            "join_game" => Ok(ClientMessage::JoinGame),
            "layout" => {
                let layout = Layout::deserialize(&mut payload)?;
                Ok(ClientMessage::Layout(layout))
            },
            "shoot" => {
                let position = Position::deserialize(&mut payload)?;
                Ok(ClientMessage::Shoot(position))
            },
            "leave_game" => Ok(ClientMessage::LeaveGame),
            "disconnect" => Ok(ClientMessage::Disconnect),
            _ => Err(DeserializeError::new(DeserializeErrorKind::UnknownHeader))
        }
    }
}

/// A trait for items that can be deserialized from a message [Payload](Payload).
trait DeserializeFromPayload: Sized {
    /// Deserialize self from message payload.
    fn deserialize(payload: &mut Payload) -> Result<Self, DeserializeError>;
}

impl DeserializeFromPayload for SessionKey {
    fn deserialize(payload: &mut Payload) -> Result<Self, DeserializeError> {
        let key = payload.take_string()?;
        match SessionKey::new(key) {
            Ok(session_key) => Ok(session_key),
            Err(error) => Err(
                StructDeserializeError::new(
                    StructDeserializeErrorKind::SessionKeyError, error.into()).into()),
        }
    }
}

impl DeserializeFromPayload for Nickname {
    fn deserialize(payload: &mut Payload) -> Result<Self, DeserializeError> {
        let nickname = payload.take_string()?;
        match Nickname::new(nickname) {
            Ok(nickname) => Ok(nickname),
            Err(error) => Err(
                StructDeserializeError::new(
                    StructDeserializeErrorKind::NicknameError, error.into()).into()),
        }
    }
}

impl DeserializeFromPayload for Position {
    fn deserialize(payload: &mut Payload) -> Result<Self, DeserializeError> {
        let row = payload.take_u8()?;
        let col = payload.take_u8()?;

        match Position::new(row, col) {
            Ok(position) => Ok(position),
            Err(error) => Err(
                StructDeserializeError::new(
                    StructDeserializeErrorKind::PositionError, error.into()).into()),
        }
    }
}

impl DeserializeFromPayload for Orientation {
    fn deserialize(payload: &mut Payload) -> Result<Self, DeserializeError> {
        match payload.take_string()?.as_str() {
            "east" => Ok(Orientation::East),
            "north" => Ok(Orientation::North),
            "west" => Ok(Orientation::West),
            "south" => Ok(Orientation::South),
            _ => Err(
                StructDeserializeError::new(
                    StructDeserializeErrorKind::OrientationError,
                    Box::new(DeserializeError::new(DeserializeErrorKind::InvalidEnumValue))).into())
        }
    }
}

impl DeserializeFromPayload for Placement {
    fn deserialize(payload: &mut Payload) -> Result<Self, DeserializeError> {
        let position = Position::deserialize(payload)?;
        let orientation = Orientation::deserialize(payload)?;

        Ok(Placement::new(position, orientation))
    }
}


impl DeserializeFromPayload for Layout {
    fn deserialize(payload: &mut Payload) -> Result<Self, DeserializeError> {
        let mut placements = Vec::with_capacity(5);

        for i in 0..5 {
            placements.push(Placement::deserialize(payload)?)
        }

        Ok(Layout::new(placements).unwrap())
    }
}