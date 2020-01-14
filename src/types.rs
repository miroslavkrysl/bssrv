use std::error::Error;
use std::fmt::{Display, Formatter};
use std::fmt;
use std::collections::HashMap;

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum DomainErrorKind {
    InvalidLength,
    InvalidCharacters,
    OutOfRange
}

/// An error indicating that a value is out of its domain.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct DomainError {
    /// Kind of domain error.
    kind: DomainErrorKind,
    /// Cause describing why is the value out of domain.
    because: String,
}

impl DomainError {
    /// Create new domain error of given kind and a message which describes the cause.
    pub fn new(kind: DomainErrorKind, because: String) -> Self {
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
            DomainErrorKind::OutOfRange => write!(f, "Out of range: {}", self.because)
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
                    format!("Nickname must have 3 - 16 characters, but has {}.", nickname.len())));
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

impl Display for Nickname {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), fmt::Error> {
        write!(f, "{}", self.nickname)
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

impl Display for SessionKey {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), fmt::Error> {
        write!(f, "{}", self.session_key)
    }
}

// ---ShipKind---

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum ShipKind {
    AircraftCarrier,
    Battleship,
    Cruiser,
    Destroyer,
    PatrolBoat
}

impl Display for ShipKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), fmt::Error> {
        match self {
            ShipKind::AircraftCarrier => write!(f, "AircraftCarrier 5"),
            ShipKind::Battleship => write!(f, "Battleship 4"),
            ShipKind::Cruiser => write!(f, "Cruiser 3"),
            ShipKind::Destroyer => write!(f, "Destroyer 2"),
            ShipKind::PatrolBoat => write!(f, "PatrolBoat 1"),
        }
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


impl Display for Position {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), fmt::Error> {
        write!(f, "({}, {})", self.row, self.col)
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

impl Display for Orientation {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), fmt::Error> {
        match self {
            Orientation::East => write!(f, "east"),
            Orientation::North => write!(f, "north"),
            Orientation::West => write!(f, "west"),
            Orientation::South => write!(f, "south"),
        }
    }
}


// ---Who---

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Who {
    You,
    Opponent,
}

impl Display for Who {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), fmt::Error> {
        match self {
            Who::You => write!(f, "you"),
            Who::Opponent => write!(f, "opponent")
        }
    }
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

impl Display for Placement {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), fmt::Error> {
        write!(f, "({}, {})", self.position, self.orientation)
    }
}


// ---Layout---

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Layout {
    placements: ShipsPlacements
}

impl Layout {
    pub fn new(placements: ShipsPlacements) -> Result<Self, DomainError> {
        if placements.len() != 5 {
            return Err(
                DomainError::new(
                    DomainErrorKind::InvalidLength,
                    format!("Layout must have exactly 5 placements, but has {}.", placements.len())));
        }

        Ok(Layout { placements })
    }

    pub fn placements(&self) -> &ShipsPlacements {
        &self.placements
    }
}

impl Display for Layout {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), fmt::Error> {
        write!(f, "{}", self.placements)
    }
}


// ---SunkShips---

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ShipsPlacements {
    placements: HashMap<ShipKind, Placement>
}

impl ShipsPlacements {
    pub fn new(ships: HashMap<ShipKind, Placement>) -> Self {
        ShipsPlacements { placements: ships }
    }

    pub fn placements(&self) -> &HashMap<ShipKind, Placement> {
        &self.placements
    }
    
    pub fn len(&self) -> usize {
        self.placements.len()
    }
}

impl Display for ShipsPlacements {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), fmt::Error> {
        let mut string = String::from("{");

        string.push_str(
            &self.placements.iter()
                .map(|(k, p)| format!("{} {}", k, p))
                .collect::<Vec<_>>().join(", "));

        string.push_str("}");

        write!(f, "{}", string)
    }
}


// ---Hits---

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

impl Display for Hits {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), fmt::Error> {
        let mut string = String::from("{");

        string.push_str(
            &self.positions.iter()
                .map(|p| format!("{}", p))
                .collect::<Vec<_>>().join(", "));

        string.push_str("}");

        write!(f, "{}", string)
    }
}


// ---RestoreState---

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum RestoreState {
    Lobby,
    Game {
        on_turn: Who,
        player_board: Hits,
        layout: Layout,
        opponent_board: Hits,
        sunk_ships: ShipsPlacements
    }
}

impl Display for RestoreState {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), fmt::Error> {
        match self {
            RestoreState::Lobby => write!(f, "lobby"),
            RestoreState::Game {
                on_turn,
                player_board,
                layout,
                opponent_board,
                sunk_ships
            } => write!(f, "game ({}, {}, {}, {}, {})", on_turn, player_board, layout, opponent_board, sunk_ships)
        }
    }
}