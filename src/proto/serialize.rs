use crate::proto::codec::{escape, Payload, ESCAPE, MESSAGE_END, PAYLOAD_START};
use crate::proto::ServerMessage;
use crate::types::{
    Hits, Layout, Nickname, Orientation, Placement, Position, RestoreState, ShipKind,
    ShipsPlacements, Who,
};
use std::convert::TryInto;

// ---Stream serialize---

/// Message serializer which serializes ServerMessages
/// into a stream of bytes into the internal buffer which
/// can be read and be cleared.
pub struct Serializer {
    byte_buffer: Vec<u8>,
}

impl Serializer {
    /// Create a new Serializer.
    pub fn new() -> Self {
        Serializer {
            byte_buffer: Vec::new(),
        }
    }

    /// Serialize message into the stream of bytes.
    pub fn serialize(&mut self, message: &ServerMessage) {
        let mut message_string = message.serialize();

        // escape message end char
        message_string = escape(&message_string, &[MESSAGE_END], ESCAPE);
        message_string.push(MESSAGE_END);

        self.byte_buffer.extend(message_string.bytes())
    }

    /// Check if a serialized bytes are available in the internal bytes buffer.
    pub fn has_bytes(&self) -> bool {
        !self.byte_buffer.is_empty()
    }

    /// Get all available serialized bytes.
    pub fn bytes(&self) -> &[u8] {
        &self.byte_buffer
    }

    /// Discard first `count` bytes from the internal buffer.
    pub fn clear(&mut self, mut count: usize) {
        if count > self.byte_buffer.len() {
            count = self.byte_buffer.len()
        }

        self.byte_buffer.drain(..count);
    }
}

// ---Message serialize---

impl ServerMessage {
    /// Serialize the message into a string.
    pub fn serialize(&self) -> String {
        let mut serialized = String::new();
        let mut payload = Payload::empty();

        match self {
            ServerMessage::IllegalState => {
                serialized.push_str("illegal_state");
            }
            ServerMessage::AliveOk => {
                serialized.push_str("alive_ok");
            }
            ServerMessage::LoginOk => {
                serialized.push_str("login_ok");
            }
            ServerMessage::LoginRestored(restore_state) => {
                serialized.push_str("login_restored");
                restore_state.serialize(&mut payload);
            }
            ServerMessage::LoginFull => {
                serialized.push_str("login_full");
            }
            ServerMessage::LoginTaken => {
                serialized.push_str("login_taken");
            }
            ServerMessage::JoinGameWait => {
                serialized.push_str("join_game_wait");
            }
            ServerMessage::JoinGameOk(opponent) => {
                serialized.push_str("join_game_ok");
                opponent.serialize(&mut payload);
            }
            ServerMessage::LayoutOk => {
                serialized.push_str("layout_ok");
            }
            ServerMessage::LayoutFail => {
                serialized.push_str("layout_fail");
            }
            ServerMessage::ShootHit => {
                serialized.push_str("shoot_hit");
            }
            ServerMessage::ShootMissed => {
                serialized.push_str("shoot_missed");
            }
            ServerMessage::ShootSunk(kind, placement) => {
                serialized.push_str("shoot_sunk");
                kind.serialize(&mut payload);
                placement.serialize(&mut payload);
            }
            ServerMessage::LeaveGameOk => {
                serialized.push_str("leave_game_ok");
            }
            ServerMessage::LogoutOk => {
                serialized.push_str("logout_ok");
            }
            ServerMessage::Disconnect => {
                serialized.push_str("disconnect");
            }
            ServerMessage::OpponentJoined(opponent) => {
                serialized.push_str("opponent_joined");
                opponent.serialize(&mut payload);
            }
            ServerMessage::OpponentReady => {
                serialized.push_str("opponent_ready");
            }
            ServerMessage::OpponentOffline => {
                serialized.push_str("opponent_offline");
            }
            ServerMessage::OpponentLeft => {
                serialized.push_str("opponent_left");
            }
            ServerMessage::OpponentMissed(position) => {
                serialized.push_str("opponent_missed");
                position.serialize(&mut payload);
            }
            ServerMessage::OpponentHit(position) => {
                serialized.push_str("opponent_hit");
                position.serialize(&mut payload);
            }
            ServerMessage::GameOver(winner) => {
                serialized.push_str("game_over");
                winner.serialize(&mut payload);
            }
        }

        if let Some(ref serialized_payload) = payload.serialize() {
            serialized.push(PAYLOAD_START);
            serialized.push_str(serialized_payload);
        }

        serialized
    }
}

/// A trait for items that can be serialized into a message [Payload](Payload).
trait SerializeIntoPayload {
    /// Serialize self into a message payload.
    fn serialize(&self, payload: &mut Payload);
}

impl SerializeIntoPayload for Nickname {
    fn serialize(&self, payload: &mut Payload) {
        payload.put_string(self.get().clone())
    }
}

impl SerializeIntoPayload for ShipKind {
    fn serialize(&self, payload: &mut Payload) {
        match self {
            ShipKind::AircraftCarrier => payload.put_string(String::from("A")),
            ShipKind::Battleship => payload.put_string(String::from("B")),
            ShipKind::Cruiser => payload.put_string(String::from("C")),
            ShipKind::Destroyer => payload.put_string(String::from("D")),
            ShipKind::PatrolBoat => payload.put_string(String::from("P")),
        }
    }
}

impl SerializeIntoPayload for Position {
    fn serialize(&self, payload: &mut Payload) {
        payload.put_int(self.row() as i32);
        payload.put_int(self.col() as i32);
    }
}

impl SerializeIntoPayload for Orientation {
    fn serialize(&self, payload: &mut Payload) {
        match self {
            Orientation::East => payload.put_string(String::from("east")),
            Orientation::North => payload.put_string(String::from("north")),
            Orientation::West => payload.put_string(String::from("west")),
            Orientation::South => payload.put_string(String::from("south")),
        }
    }
}

impl SerializeIntoPayload for Who {
    fn serialize(&self, payload: &mut Payload) {
        match self {
            Who::You => payload.put_string(String::from("you")),
            Who::Opponent => payload.put_string(String::from("opponent")),
        }
    }
}

impl SerializeIntoPayload for Placement {
    fn serialize(&self, payload: &mut Payload) {
        self.position().serialize(payload);
        self.orientation().serialize(payload);
    }
}

impl SerializeIntoPayload for Hits {
    fn serialize(&self, payload: &mut Payload) {
        let positions = self.positions();
        payload.put_int(positions.len().try_into().unwrap());

        for position in positions {
            position.serialize(payload);
        }
    }
}

impl SerializeIntoPayload for ShipsPlacements {
    fn serialize(&self, payload: &mut Payload) {
        let ships = self.placements();
        payload.put_int(ships.len().try_into().unwrap());

        for (kind, placement) in ships.iter() {
            kind.serialize(payload);
            placement.serialize(payload);
        }
    }
}

impl SerializeIntoPayload for RestoreState {
    fn serialize(&self, payload: &mut Payload) {
        match self {
            RestoreState::Lobby => {
                payload.put_string(String::from("lobby"));
            }
            RestoreState::Game {
                opponent,
                on_turn,
                player_board_hits,
                player_board_misses,
                layout,
                opponent_board_hits,
                opponent_board_misses,
                sunk_ships,
            } => {
                payload.put_string(String::from("game"));
                opponent.serialize(payload);
                on_turn.serialize(payload);
                player_board_hits.serialize(payload);
                player_board_misses.serialize(payload);
                layout.serialize(payload);
                opponent_board_hits.serialize(payload);
                opponent_board_misses.serialize(payload);
                sunk_ships.serialize(payload);
            }
        };
    }
}

impl SerializeIntoPayload for Layout {
    fn serialize(&self, payload: &mut Payload) {
        self.placements().serialize(payload);
    }
}
