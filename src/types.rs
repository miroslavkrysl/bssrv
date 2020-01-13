use std::error::Error;
use std::fmt::{Display, Formatter};
use std::fmt;
use std::collections::HashMap;

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum DomainErrorKind {
    InvalidLength,
    InvalidCharacters,
    OutOfRange,
}

/// An error indicating that a value is out of its domain.
#[derive(Debug, Eq, PartialEq)]
pub struct DomainError {
    /// Kind of domain error.
    kind: DomainErrorKind,
    /// Cause describing why is the value out of domain.
    because: String,
}

impl DomainError {
    /// Create new domain error of given kind and a message which describes the cause.
    fn new(kind: DomainErrorKind, because: String) -> Self {
        DomainError {
            kind,
            because,
        }
    }
}

impl Display for DomainError {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), fmt::Error> {
        match self.kind {
            DomainErrorKind::InvalidLength => write!(f, "Invalid length: {}", self.because),
            DomainErrorKind::InvalidCharacters => write!(f, "Invalid characters: {}", self.because),
            DomainErrorKind::OutOfRange => write!(f, "Out of range: {}", self.because),
        }
    }
}

impl Error for DomainError {}


// ---Nickname---

/// A string wrapper type for Nickname.
/// Forces string to has 3 - 32 alphanumeric characters.
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Nickname {
    nickname: String,
}

impl Nickname {
    pub fn new(nickname: String) -> Result<Self, DomainError> {
        if nickname.len() < 3 || nickname.len() > 32 {
            return Err(
                DomainError::new(
                    DomainErrorKind::InvalidLength,
                    format!("Nickname must have 3 - 32 characters, but has {}.", nickname.len())));
        }

        if !nickname.chars().all(|c| c.is_alphanumeric()) {
            return Err(
                DomainError::new(
                    DomainErrorKind::InvalidCharacters,
                    String::from("Nickname must contain only alphanumeric characters.")));
        }

        Ok(Nickname { nickname })
    }

    pub fn get(&self) -> &String {
        &self.nickname
    }
}


// ---SessionKey---

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct SessionKey {
    session_key: String,
}

impl SessionKey {
    pub fn new(session_key: String) -> Result<Self, DomainError> {
        if session_key.len() != 32 {
            return Err(
                DomainError::new(
                    DomainErrorKind::InvalidLength,
                    format!("SessionKey must have 3 - 32 characters, but has {}.", session_key.len())));
        }

        if !session_key.chars().all(|c| c.is_alphanumeric()) {
            return Err(
                DomainError::new(
                    DomainErrorKind::InvalidCharacters,
                    String::from("SessionKey must contain only alphanumeric characters.")));
        }

        Ok(SessionKey { session_key })
    }

    pub fn get(&self) -> &String {
        &self.session_key
    }
}


// ---ShipId---

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct ShipId {
    id: u8
}

impl ShipId {
    pub fn new(id: u8) -> Result<Self, DomainError> {
        if id >= 5 {
            return Err(
                DomainError::new(
                    DomainErrorKind::OutOfRange,
                    format!("ShipId must be between 0 - 4. {} given.", id)));
        }

        Ok(ShipId { id })
    }

    pub fn get(&self) -> u8 {
        self.id
    }
}


// ---Position---

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct Position {
    row: u8,
    col: u8,
}

impl Position {
    pub fn new(row: u8, col: u8) -> Result<Self, DomainError> {
        if row >= 10 {
            return Err(
                DomainError::new(
                    DomainErrorKind::OutOfRange,
                    format!("Position row must be between 0 - 9. {} given.", row)));
        }

        if row >= 10 {
            return Err(
                DomainError::new(
                    DomainErrorKind::OutOfRange,
                    format!("Position col must be between 0 - 9. {} given.", col)));
        }

        Ok(Position {
            row,
            col,
        })
    }

    pub fn row(&self) -> u8 {
        self.row
    }

    pub fn col(&self) -> u8 {
        self.col
    }
}


// ---Orientation---

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Orientation {
    East,
    North,
    West,
    South,
}


// ---Who---

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Who {
    YOU,
    OPPONENT,
}


// ---Placement---

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct Placement {
    position: Position,
    orientation: Orientation,
}

impl Placement {
    pub fn new(position: Position, orientation: Orientation) -> Self {
        Placement { position, orientation }
    }

    pub fn position(&self) -> Position {
        self.position
    }

    pub fn orientation(&self) -> Orientation {
        self.orientation
    }
}


// ---Layout---

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Layout {
    placements: Vec<Placement>
}

impl Layout {
    pub fn new(placements: Vec<Placement>) -> Result<Self, DomainError> {
        if placements.len() != 5 {
            return Err(
                DomainError::new(
                    DomainErrorKind::InvalidLength,
                    format!("Layout must have exactly 5 placements, but has {}.", placements.len())));
        }

        Ok(Layout { placements })
    }

    pub fn placements(&self) -> &Vec<Placement> {
        &self.placements
    }
}


// ---SunkShips---

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct SunkShips {
    ships: HashMap<ShipId, Placement>
}

impl SunkShips {
    pub fn new(ships: HashMap<ShipId, Placement>) -> Self {
        SunkShips { ships }
    }

    pub fn ships(&self) -> &HashMap<ShipId, Placement> {
        &self.ships
    }
}


// ---Layout---

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Hits {
    positions: Vec<Position>
}

impl Hits {
    pub fn new(positions: Vec<Position>) -> Self {
        Hits {
            positions
        }
    }

    pub fn positions(&self) -> &Vec<Position> {
        &self.positions
    }
}


// ---RestoreState---

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum RestoreState {
    Lobby,
    Game {
        on_turn: Who,
        player_board: Hits,
        opponent_board: Hits,
        sunk_ships: SunkShips
    },
    GameOver {
        winner: Who
    },
}
