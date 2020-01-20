//! Client messages deserialization logic

use crate::types::{Nickname, Layout, Position, Placement, Orientation, ShipsPlacements, ShipKind};
use crate::proto::{ClientMessage};
use crate::proto::codec::{find, Payload, PAYLOAD_START, ESCAPE, MESSAGE_END, MAX_MESSAGE_LENGTH, unescape};
use std::fmt::{Display, Formatter};
use std::fmt;
use std::error::Error;
use std::num::ParseIntError;
use std::collections::{HashMap};


// ---Stream deserialize---

/// Message deserializer which deserializes ClientMessages
/// from the stream of bytes. Deserialized messages can be later
/// taken from the internal buffer all at once.
///
/// There must be only one Deserializer per stream, because
/// the deserializer remembers previously not yet deserialized parts
/// of the stream.
pub struct Deserializer {
    byte_buffer: Vec<u8>,
    string_buffer: String,
    message_buffer: Vec<ClientMessage>,
}

impl Deserializer {
    /// Create a new deserializer
    pub fn new() -> Self {
        Deserializer {
            byte_buffer: Vec::new(),
            string_buffer: String::new(),
            message_buffer: Vec::new(),
        }
    }

    /// Deserialize all available messages from the stream of bytes.
    /// If there is no message yet to be deserialized, the returned vector is empty.
    pub fn deserialize(&mut self, bytes: &[u8]) -> Result<(), DeserializationError> {

        // add new bytes to undecoded bytes from previous call
        self.byte_buffer.extend_from_slice(bytes);

        // decode bytes into utf8 string
        match std::str::from_utf8(&mut self.byte_buffer) {
            Ok(string) => {
                // all bytes decoded into utf8 string

                self.string_buffer.push_str(&string);
                self.byte_buffer.clear();
            },
            Err(error) => {
                // some characters are invalid or incomplete

                if let Some(_) = error.error_len() {
                    // invalid utf8 sequence

                    return Err(DeserializationErrorKind::InvalidUtf8.into());
                }

                // last character is incomplete

                // store complete characters into the string buffer
                unsafe {
                    self.string_buffer.push_str(std::str::from_utf8_unchecked(&self.byte_buffer[..error.valid_up_to()]))
                }

                // move incomplete characters to the beginning of the byte buffer
                self.byte_buffer.drain(..error.valid_up_to());
            },
        }

        // deserialize decoded string into messages

        // storage for deserialized messages
        let mut byte_offset = 0;

        loop {
            let separator_pos = find(&self.string_buffer[byte_offset..], MESSAGE_END, ESCAPE);

            match separator_pos {
                None => {
                    // message is incomplete

                    if self.string_buffer[byte_offset..].len() > MAX_MESSAGE_LENGTH {
                        // max message length exceeded
                        return Err(DeserializationErrorKind::MessageLengthExceeded.into());
                    }

                    break;
                },
                Some(separator_pos) => {
                    // a message end was found

                    let message_str = &self.string_buffer[byte_offset..separator_pos];
                    byte_offset = separator_pos + MESSAGE_END.len_utf8();

                    // unescape message end character
                    let message_string = unescape(message_str, &[MESSAGE_END], ESCAPE);

                    // build message
                    let message = ClientMessage::deserialize(&message_string)?;
                    self.message_buffer.push(message);
                },
            }
        }

        self.string_buffer.drain(..byte_offset);

        Ok(())
    }

    /// Check if a deserialized message is available in the internal message buffer.
    pub fn has_message(&self) -> bool {
        !self.message_buffer.is_empty()
    }

    /// Get all available deserialized messages.
    pub fn take_messages(&mut self) -> Vec<ClientMessage> {
        self.message_buffer.drain(..).collect()
    }
}

// ---Message deserialize---

impl ClientMessage {
    /// Deserialize message from a string.
    pub fn deserialize(serialized: &str) -> Result<Self, DeserializationError> {
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
            "logout" => Ok(ClientMessage::LogOut),
            _ => Err(DeserializationError::new(DeserializationErrorKind::UnknownHeader))
        }
    }
}

/// A trait for items that can be deserialized from a message [Payload](Payload).
trait DeserializeFromPayload: Sized {
    /// Deserialize self from message payload.
    fn deserialize(payload: &mut Payload) -> Result<Self, DeserializationError>;
}

impl DeserializeFromPayload for Nickname {
    fn deserialize(payload: &mut Payload) -> Result<Self, DeserializationError> {
        let nickname = payload.take_string();

        if let Err(error) = nickname {
            return Err(
                StructDeserializationError::new(
                    StructDeserializeErrorKind::Nickname, error.into()).into())
        }

        match Nickname::new(nickname.unwrap()) {
            Ok(nickname) => Ok(nickname),
            Err(error) => Err(
                StructDeserializationError::new(
                    StructDeserializeErrorKind::Nickname, error.into()).into()),
        }
    }
}

impl DeserializeFromPayload for Position {
    fn deserialize(payload: &mut Payload) -> Result<Self, DeserializationError> {
        let row = payload.take_u8();
        let col = payload.take_u8();

        if let Err(error) = row {
            return Err(
                StructDeserializationError::new(
                    StructDeserializeErrorKind::Position, error.into()).into())
        }

        if let Err(error) = col {
            return Err(
                StructDeserializationError::new(
                    StructDeserializeErrorKind::Position, error.into()).into())
        }

        match Position::new(row.unwrap(), col.unwrap()) {
            Ok(position) => Ok(position),
            Err(error) => Err(
                StructDeserializationError::new(
                    StructDeserializeErrorKind::Position, error.into()).into()),
        }
    }
}

impl DeserializeFromPayload for Orientation {
    fn deserialize(payload: &mut Payload) -> Result<Self, DeserializationError> {
        let string = payload.take_string();

        if let Err(error) = string {
            return Err(
                StructDeserializationError::new(
                    StructDeserializeErrorKind::Orientation, error.into()).into())
        }

        match string.unwrap().as_str() {
            "east" => Ok(Orientation::East),
            "north" => Ok(Orientation::North),
            "west" => Ok(Orientation::West),
            "south" => Ok(Orientation::South),
            _ => Err(
                StructDeserializationError::new(
                    StructDeserializeErrorKind::Orientation,
                    Box::new(DeserializationError::new(DeserializationErrorKind::InvalidEnumValue))).into())
        }
    }
}

impl DeserializeFromPayload for Placement {
    fn deserialize(payload: &mut Payload) -> Result<Self, DeserializationError> {
        let position = Position::deserialize(payload);
        let orientation = Orientation::deserialize(payload);

        if let Err(error) = position {
            return Err(
                StructDeserializationError::new(
                    StructDeserializeErrorKind::Placement, error.into()).into())
        }

        if let Err(error) = orientation {
            return Err(
                StructDeserializationError::new(
                    StructDeserializeErrorKind::Placement, error.into()).into())
        }

        Ok(Placement::new(position.unwrap(), orientation.unwrap()))
    }
}


impl DeserializeFromPayload for Layout {
    fn deserialize(payload: &mut Payload) -> Result<Self, DeserializationError> {
        let placements = ShipsPlacements::deserialize(payload);

        if let Err(error) = placements {
            return Err(
                StructDeserializationError::new(
                    StructDeserializeErrorKind::Layout, error.into()).into())
        }

        match Layout::new(placements.unwrap()) {
            Ok(layout) => Ok(layout),
            Err(error) => Err(
                StructDeserializationError::new(
                    StructDeserializeErrorKind::Layout, error.into()).into()),
        }
    }
}

impl DeserializeFromPayload for ShipsPlacements {
    fn deserialize(payload: &mut Payload) -> Result<Self, DeserializationError> {
        let size = payload.take_u8();

        if let Err(error) = size {
            return Err(
                StructDeserializationError::new(
                    StructDeserializeErrorKind::ShipsPlacements, error.into()).into())
        }

        let mut placements = HashMap::with_capacity(5);

        for _ in 0..(size.unwrap()) {
            let kind = ShipKind::deserialize(payload);
            let placement = Placement::deserialize(payload);

            if let Err(error) = kind {
                return Err(
                    StructDeserializationError::new(
                        StructDeserializeErrorKind::ShipsPlacements, error.into()).into())
            }

            if let Err(error) = placement {
                return Err(
                    StructDeserializationError::new(
                        StructDeserializeErrorKind::ShipsPlacements, error.into()).into())
            }

            placements.insert(kind.unwrap(), placement.unwrap());
        }

        Ok(ShipsPlacements::new(placements))
    }
}

impl DeserializeFromPayload for ShipKind {
    fn deserialize(payload: &mut Payload) -> Result<Self, DeserializationError> {
        let string = payload.take_string();

        if let Err(error) = string {
            return Err(
                StructDeserializationError::new(
                    StructDeserializeErrorKind::ShipKind, error.into()).into())
        }

        match string.unwrap().as_str() {
            "A" => Ok(ShipKind::AircraftCarrier),
            "B" => Ok(ShipKind::Battleship),
            "C" => Ok(ShipKind::Cruiser),
            "D" => Ok(ShipKind::Destroyer),
            "P" => Ok(ShipKind::PatrolBoat),
            _ => Err(
                StructDeserializationError::new(
                    StructDeserializeErrorKind::ShipKind,
                    Box::new(DeserializationError::new(DeserializationErrorKind::InvalidEnumValue))).into())
        }
    }
}




// ---ERRORS---

/// Describes the kind of the deserialization error.
#[derive(Debug, Eq, PartialEq)]
pub enum DeserializationErrorKind {
    UnknownHeader,
    NoMorePayloadItems,
    InvalidEnumValue,
    MessageLengthExceeded,
    InvalidUtf8,
    ParseInt(ParseIntError),
    StructDeserialization(StructDeserializationError),
}

impl Display for DeserializationErrorKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), fmt::Error> {
        match self {
            DeserializationErrorKind::UnknownHeader => write!(f, "Unknown header."),
            DeserializationErrorKind::NoMorePayloadItems => write!(f, "Further payload item was expected, but not present."),
            DeserializationErrorKind::InvalidEnumValue => write!(f, "Invalid enum value."),
            DeserializationErrorKind::MessageLengthExceeded => write!(f, "String segment is too long to be a valid message."),
            DeserializationErrorKind::InvalidUtf8 => write!(f, "Invalid UTF-8 byte sequence."),
            DeserializationErrorKind::ParseInt(ref error) => write!(f, "Integer can't be properly deserialized: {}", error),
            DeserializationErrorKind::StructDeserialization(ref error) => write!(f, "{}", error),
        }
    }
}

/// An error indicating that a value is out of its domain.
#[derive(Debug, Eq, PartialEq)]
pub struct DeserializationError {
    /// Kind of deserialization error.
    kind: DeserializationErrorKind
}

impl DeserializationError {
    /// Create new deserialization error of given kind.
    pub fn new(kind: DeserializationErrorKind) -> Self {
        DeserializationError {
            kind
        }
    }
}

impl Display for DeserializationError {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), fmt::Error> {
        write!(f, "Deserialization error: {}", self.kind)
    }
}

impl From<DeserializationErrorKind> for DeserializationError {
    fn from(kind: DeserializationErrorKind) -> Self {
        DeserializationError::new(kind)
    }
}

impl From<ParseIntError> for DeserializationError {
    fn from(error: ParseIntError) -> Self {
        DeserializationError::new(DeserializationErrorKind::ParseInt(error))
    }
}

impl From<StructDeserializationError> for DeserializationError {
    fn from(error: StructDeserializationError) -> Self {
        DeserializationError::new(DeserializationErrorKind::StructDeserialization(error))
    }
}

impl Error for DeserializationError {}

/// Describes the kind of the struct deserialization error.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum StructDeserializeErrorKind {
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
pub struct StructDeserializationError {
    /// Kind of deserialization error.
    kind: StructDeserializeErrorKind,

    /// Cause of the error.
    error: Box<dyn Error>
}

impl StructDeserializationError {
    /// Create new struct deserialization error of given kind and cause.
    fn new(kind: StructDeserializeErrorKind, cause: Box<dyn Error>) -> Self {
        StructDeserializationError {
            kind,
            error: cause
        }
    }
}

impl PartialEq for StructDeserializationError {
    fn eq(&self, other: &Self) -> bool {
        self.kind == other.kind
    }
}

impl Eq for StructDeserializationError {}

impl Display for StructDeserializationError {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), fmt::Error> {
        write!(f, "{}: {}", self.kind, self.error)
    }
}

impl Error for StructDeserializationError {}
