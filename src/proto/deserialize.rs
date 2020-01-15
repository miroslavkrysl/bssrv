//! Client messages deserialization logic

use crate::types::{SessionKey, Nickname, Layout, Position, Placement, Orientation, DomainErrorKind, ShipsPlacements, ShipKind};
use crate::proto::{ClientMessage, ServerMessage};
use crate::proto::codec::{find, Payload, PAYLOAD_START, ESCAPE, MESSAGE_END, MAX_MESSAGE_LENGTH, unescape};
use std::fmt::{Display, Formatter};
use std::fmt;
use std::error::Error;
use std::num::ParseIntError;
use std::collections::{HashMap, LinkedList};
use log::{trace, error, info};

// ---ERRORS---

/// Describes the kind of the deserialization error.
#[derive(Debug, Eq, PartialEq)]
pub enum DeserializeErrorKind {
    UnknownHeader,
    NoMorePayloadItems,
    InvalidEnumValue,
    MessageLengthExceeded,
    InvalidUtf8,
    IntError(ParseIntError),
    StructError(StructDeserializeError),
}

impl Display for DeserializeErrorKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), fmt::Error> {
        match self {
            DeserializeErrorKind::UnknownHeader => write!(f, "Unknown header."),
            DeserializeErrorKind::NoMorePayloadItems => write!(f, "Further payload item was expected, but not present."),
            DeserializeErrorKind::InvalidEnumValue => write!(f, "Invalid enum value."),
            DeserializeErrorKind::MessageLengthExceeded => write!(f, "String segment is too long to be a valid message."),
            DeserializeErrorKind::InvalidUtf8 => write!(f, "Invalid UTF-8 byte sequence."),
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

impl From<DeserializeErrorKind> for DeserializeError {
    fn from(kind: DeserializeErrorKind) -> Self {
        DeserializeError::new(kind)
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
    SessionKey,
    Nickname,
    ShipKind,
    Position,
    Orientation,
    Placement,
    Layout,
    ShipsPlacements,
}

impl Display for StructDeserializeErrorKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), fmt::Error> {
        match self {
            StructDeserializeErrorKind::SessionKey =>
                write!(f, "SessionKey can't be properly deserialized"),
            StructDeserializeErrorKind::Nickname =>
                write!(f, "Nickname can't be properly deserialized"),
            StructDeserializeErrorKind::ShipKind =>
                write!(f, "ShipId can't be properly deserialized"),
            StructDeserializeErrorKind::Position =>
                write!(f, "Position can't be properly deserialized"),
            StructDeserializeErrorKind::Orientation =>
                write!(f, "Orientation can't be properly deserialized"),
            StructDeserializeErrorKind::Placement =>
                write!(f, "Placement can't be properly deserialized"),
            StructDeserializeErrorKind::ShipsPlacements =>
                write!(f, "ShipsPlacements can't be properly deserialized"),
            StructDeserializeErrorKind::Layout =>
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

// ---Stream deserialize---

pub struct Deserializer {
    to_decode: Vec<u8>,
    byte_buffer: Vec<u8>,
    string_buffer: String,
}

impl Deserializer {
    pub fn new() -> Self {
        Deserializer {
            to_decode: Vec::new(),
            byte_buffer: Vec::new(),
            string_buffer: String::new(),
        }
    }

    pub fn deserialize(&mut self, bytes: &[u8]) -> Result<Vec<ClientMessage>, DeserializeError> {
        info!("deserializing {} bytes", bytes.len());
        let mut messages = Vec::new();

        // decode bytes into utf8 string
        self.to_decode.clear();

        if !self.byte_buffer.is_empty() {
            self.to_decode.extend(self.byte_buffer.drain(..));
        }

        self.to_decode.extend_from_slice(bytes);

        match std::str::from_utf8(&mut self.to_decode) {
            Ok(string) => {
                trace!("all bytes decoded into string");

                // all bytes decoded into utf8 string
                self.string_buffer.push_str(&string);
            },
            Err(error) => {
                // some bytes are invalid

                if let Some(_) = error.error_len() {
                    error!("invalid utf-8 sequence");
                    // invalid utf8 sequence
                    return Err(DeserializeErrorKind::InvalidUtf8.into());
                }

                // last utf8 character is not complete
                trace!("last utf-8 char incomplete");

                let (complete, incomplete) = self.to_decode.split_at(error.valid_up_to());

                unsafe {
                    // store complete sequence into string buffer
                    self.string_buffer.push_str(std::str::from_utf8_unchecked(complete))
                }

                // store incomplete character into byte buffer
                self.byte_buffer.extend_from_slice(incomplete);
            },
        }

        // deserialize string into messages
        let mut byte_offset = 0;

        loop {
            let separator_pos = find(&self.string_buffer[byte_offset..], MESSAGE_END, ESCAPE);

            match separator_pos {
                None => {
                    // message is not complete yet
                    trace!("message is not complete");

                    if self.string_buffer[byte_offset..].len() > MAX_MESSAGE_LENGTH {
                        error!("allowed message length exceeded");
                        return Err(DeserializeErrorKind::MessageLengthExceeded.into());
                    }

                    break;
                },
                Some(separator_pos) => {
                    trace!("a message end was found - deserializing");

                    let message_str = &self.string_buffer[byte_offset..separator_pos];

                    byte_offset = separator_pos + MESSAGE_END.len_utf8();

                    // unescape message end char
                    let message_string = unescape(message_str, &[MESSAGE_END], ESCAPE);

                    // build message
                    let message = ClientMessage::deserialize(&message_string)?;
                    messages.push(message);
                },
            }
        }

        if byte_offset > 0 {
            // move undeserialized string to the string beginning
            self.string_buffer.drain(..byte_offset);
        }

        info!("{} messages was found", messages.len());

        Ok(messages)
    }
}

// ---Message deserialize---

impl ClientMessage {
    /// Deserialize message from a string.
    pub fn deserialize(serialized: &str) -> Result<Self, DeserializeError> {
        // deserialize header
        let payload_start = find(serialized, PAYLOAD_START, ESCAPE);

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
        let key = payload.take_string();

        if let Err(error) = key {
            return Err(
                StructDeserializeError::new(
                    StructDeserializeErrorKind::SessionKey, error.into()).into())
        }

        match SessionKey::new(key.unwrap()) {
            Ok(session_key) => Ok(session_key),
            Err(error) => Err(
                StructDeserializeError::new(
                    StructDeserializeErrorKind::SessionKey, error.into()).into()),
        }
    }
}

impl DeserializeFromPayload for Nickname {
    fn deserialize(payload: &mut Payload) -> Result<Self, DeserializeError> {
        let nickname = payload.take_string();

        if let Err(error) = nickname {
            return Err(
                StructDeserializeError::new(
                    StructDeserializeErrorKind::Nickname, error.into()).into())
        }

        match Nickname::new(nickname.unwrap()) {
            Ok(nickname) => Ok(nickname),
            Err(error) => Err(
                StructDeserializeError::new(
                    StructDeserializeErrorKind::Nickname, error.into()).into()),
        }
    }
}

impl DeserializeFromPayload for Position {
    fn deserialize(payload: &mut Payload) -> Result<Self, DeserializeError> {
        let row = payload.take_u8();
        let col = payload.take_u8();

        if let Err(error) = row {
            return Err(
                StructDeserializeError::new(
                    StructDeserializeErrorKind::Position, error.into()).into())
        }

        if let Err(error) = col {
            return Err(
                StructDeserializeError::new(
                    StructDeserializeErrorKind::Position, error.into()).into())
        }

        match Position::new(row.unwrap(), col.unwrap()) {
            Ok(position) => Ok(position),
            Err(error) => Err(
                StructDeserializeError::new(
                    StructDeserializeErrorKind::Position, error.into()).into()),
        }
    }
}

impl DeserializeFromPayload for Orientation {
    fn deserialize(payload: &mut Payload) -> Result<Self, DeserializeError> {
        let string = payload.take_string();

        if let Err(error) = string {
            return Err(
                StructDeserializeError::new(
                    StructDeserializeErrorKind::Orientation, error.into()).into())
        }

        match string.unwrap().as_str() {
            "east" => Ok(Orientation::East),
            "north" => Ok(Orientation::North),
            "west" => Ok(Orientation::West),
            "south" => Ok(Orientation::South),
            _ => Err(
                StructDeserializeError::new(
                    StructDeserializeErrorKind::Orientation,
                    Box::new(DeserializeError::new(DeserializeErrorKind::InvalidEnumValue))).into())
        }
    }
}

impl DeserializeFromPayload for Placement {
    fn deserialize(payload: &mut Payload) -> Result<Self, DeserializeError> {
        let position = Position::deserialize(payload);
        let orientation = Orientation::deserialize(payload);

        if let Err(error) = position {
            return Err(
                StructDeserializeError::new(
                    StructDeserializeErrorKind::Placement, error.into()).into())
        }

        if let Err(error) = orientation {
            return Err(
                StructDeserializeError::new(
                    StructDeserializeErrorKind::Placement, error.into()).into())
        }

        Ok(Placement::new(position.unwrap(), orientation.unwrap()))
    }
}


impl DeserializeFromPayload for Layout {
    fn deserialize(payload: &mut Payload) -> Result<Self, DeserializeError> {
        let placements = ShipsPlacements::deserialize(payload);

        if let Err(error) = placements {
            return Err(
                StructDeserializeError::new(
                    StructDeserializeErrorKind::Layout, error.into()).into())
        }

        match Layout::new(placements.unwrap()) {
            Ok(layout) => Ok(layout),
            Err(error) => Err(
                StructDeserializeError::new(
                    StructDeserializeErrorKind::Layout, error.into()).into()),
        }
    }
}

impl DeserializeFromPayload for ShipsPlacements {
    fn deserialize(payload: &mut Payload) -> Result<Self, DeserializeError> {
        let size = payload.take_u8();

        if let Err(error) = size {
            return Err(
                StructDeserializeError::new(
                    StructDeserializeErrorKind::ShipsPlacements, error.into()).into())
        }

        let mut placements = HashMap::with_capacity(5);

        for _ in 0..(size.unwrap()) {
            let kind = ShipKind::deserialize(payload);
            let placement = Placement::deserialize(payload);

            if let Err(error) = kind {
                return Err(
                    StructDeserializeError::new(
                        StructDeserializeErrorKind::ShipsPlacements, error.into()).into())
            }

            if let Err(error) = placement {
                return Err(
                    StructDeserializeError::new(
                        StructDeserializeErrorKind::ShipsPlacements, error.into()).into())
            }

            placements.insert(kind.unwrap(), placement.unwrap());
        }

        Ok(ShipsPlacements::new(placements))
    }
}

impl DeserializeFromPayload for ShipKind {
    fn deserialize(payload: &mut Payload) -> Result<Self, DeserializeError> {
        let string = payload.take_string();

        if let Err(error) = string {
            return Err(
                StructDeserializeError::new(
                    StructDeserializeErrorKind::ShipKind, error.into()).into())
        }

        match string.unwrap().as_str() {
            "AC" => Ok(ShipKind::AircraftCarrier),
            "B" => Ok(ShipKind::Battleship),
            "C" => Ok(ShipKind::Cruiser),
            "D" => Ok(ShipKind::Destroyer),
            "PB" => Ok(ShipKind::PatrolBoat),
            _ => Err(
                StructDeserializeError::new(
                    StructDeserializeErrorKind::ShipKind,
                    Box::new(DeserializeError::new(DeserializeErrorKind::InvalidEnumValue))).into())
        }
    }
}