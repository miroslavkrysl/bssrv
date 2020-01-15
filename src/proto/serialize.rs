use crate::proto::ServerMessage;
use crate::proto::codec::{PAYLOAD_ITEM_SEPARATOR, Payload};
use crate::types::{Nickname, SessionKey, ShipKind, Position, Orientation, Placement, RestoreState, Hits, Who, ShipsPlacements, Layout};
use std::convert::TryInto;

impl ServerMessage {
    /// Serialize the message into a string.
    pub fn serialize(&self) -> String {
        let mut serialized = String::new();
        let mut payload = Payload::empty();

        match self {
            ServerMessage::IllegalState => {
                serialized.push_str("illegal_state");
            },
            ServerMessage::AliveOk => {
                serialized.push_str("alive_ok");
            }
            ServerMessage::RestoreSessionOk(restore_state) => {
                serialized.push_str("restore_session");
                restore_state.serialize(&mut payload);
            }
            ServerMessage::RestoreSessionFail => {
                serialized.push_str("restore_session_fail");
            }
            ServerMessage::LoginOk(session_key) => {
                serialized.push_str("login_ok");
                session_key.serialize(&mut payload);
            }
            ServerMessage::LoginOkFail => {
                serialized.push_str("login_fail");
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
            ServerMessage::DisconnectOk => {
                serialized.push_str("disconnect_ok");
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
        }

        if let Some(ref serialized_payload) = payload.serialize() {
            serialized.push(PAYLOAD_ITEM_SEPARATOR);
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

impl SerializeIntoPayload for SessionKey {
    fn serialize(&self, payload: &mut Payload) {
        payload.put_string(self.get().clone())
    }
}

impl SerializeIntoPayload for ShipKind {
    fn serialize(&self, payload: &mut Payload) {
        match self {
            ShipKind::AircraftCarrier => payload.put_string(String::from("AC")),
            ShipKind::Battleship => payload.put_string(String::from("B")),
            ShipKind::Cruiser => payload.put_string(String::from("C")),
            ShipKind::Destroyer => payload.put_string(String::from("D")),
            ShipKind::PatrolBoat => payload.put_string(String::from("PB")),
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
            RestoreState::Lobby => payload.put_string(String::from("lobby")),
            RestoreState::Game {
                on_turn,
                player_board,
                layout,
                opponent_board,
                sunk_ships
            } => {
                payload.put_string(String::from("game"));
                on_turn.serialize(payload);
                player_board.serialize(payload);
                layout.serialize(payload);
                opponent_board.serialize(payload);
                sunk_ships.serialize(payload);
            },
        };
    }
}

impl SerializeIntoPayload for Layout {
    fn serialize(&self, payload: &mut Payload) {
        self.placements().serialize(payload);
    }
}