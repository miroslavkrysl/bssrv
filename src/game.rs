use crate::types::{
    Hits, Layout, Orientation, Placement, Position, ShipKind, ShipsPlacements, Who,
};
use std::collections::HashMap;

/// An error indicating that player did something illegal with the game.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum GameError {
    AlreadyHasLayout,
    InvalidLayout,
    NotOnTurn,
}

/// A state of the one board cell.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum BoardCell {
    Empty,
    Miss,
    Hit,
    Ship(ShipKind),
}

/// Ship of particular kind and health.
#[derive(Debug)]
pub struct Ship {
    kind: ShipKind,
    health: u8,
}

/// The result of shooting.
#[derive(Debug, Eq, PartialEq)]
pub enum ShootResult {
    Missed,
    Hit,
    Sunk(ShipKind, Placement),
}

impl Ship {
    /// Create a new ship of the given kind.
    /// Sets the ships health to the correct value according to the kind.
    pub fn new(kind: ShipKind) -> Self {
        Ship {
            kind,
            health: kind.cells(),
        }
    }

    /// Decrease the ships health by one if not already zero.
    pub fn hit(&mut self) {
        if self.health > 0 {
            self.health -= 1;
        }
    }

    /// Get the kind of the ship
    pub fn kind(&self) -> ShipKind {
        self.kind
    }

    /// Check whether is the ship sunk (health == 0).
    pub fn is_sunk(&self) -> bool {
        self.health == 0
    }
}

/// A game of two players.
pub struct Game {
    first_player: usize,
    second_player: usize,
    first_layout: Option<Layout>,
    second_layout: Option<Layout>,
    first_board: [[BoardCell; 10]; 10],
    second_board: [[BoardCell; 10]; 10],
    first_ships: HashMap<ShipKind, Ship>,
    second_ships: HashMap<ShipKind, Ship>,
    on_turn: usize,
    winner: Option<usize>,
}

impl Game {
    /// Create a new game with the two players.
    pub fn new(first_player: usize, second_player: usize) -> Self {
        Game {
            first_player,
            second_player,
            first_layout: None,
            second_layout: None,
            first_board: [[BoardCell::Empty; 10]; 10],
            second_board: [[BoardCell::Empty; 10]; 10],
            first_ships: HashMap::new(),
            second_ships: HashMap::new(),
            on_turn: first_player,
            winner: None,
        }
    }

    /// Set the ships layout for the player.
    pub fn set_layout(&mut self, player: usize, layout: Layout) -> Result<bool, GameError> {
        let (l, s, b) = match player {
            id if id == self.first_player => (
                &mut self.first_layout,
                &mut self.first_ships,
                &mut self.first_board,
            ),
            id if id == self.second_player => (
                &mut self.second_layout,
                &mut self.second_ships,
                &mut self.second_board,
            ),
            _ => panic!("player {} is not in this game", player),
        };

        if l.is_some() {
            return Err(GameError::AlreadyHasLayout);
        }

        if !layout.is_valid() {
            return Err(GameError::InvalidLayout);
        }

        *l = Some(layout);

        // prepare fleet
        s.insert(
            ShipKind::AircraftCarrier,
            Ship::new(ShipKind::AircraftCarrier),
        );
        s.insert(ShipKind::Battleship, Ship::new(ShipKind::Battleship));
        s.insert(ShipKind::Cruiser, Ship::new(ShipKind::Cruiser));
        s.insert(ShipKind::Destroyer, Ship::new(ShipKind::Destroyer));
        s.insert(ShipKind::PatrolBoat, Ship::new(ShipKind::PatrolBoat));

        // mark ships on board
        for (kind, placement) in l.as_ref().unwrap().placements().placements() {
            let cells = kind.cells();
            let mut row: i32 = placement.position().row() as i32;
            let mut col: i32 = placement.position().col() as i32;

            let (inc_r, inc_c) = match placement.orientation() {
                Orientation::East => (0, 1),
                Orientation::North => (-1, 0),
                Orientation::West => (0, -1),
                Orientation::South => (1, 0),
            };

            // mark ships cells
            for _ in 0..cells {
                b[row as usize][col as usize] = BoardCell::Ship(*kind);

                row += inc_r;
                col += inc_c;
            }
        }

        Ok(self.playing())
    }

    /// Check if the both ship layouts are set and the game is in progress.
    pub fn playing(&self) -> bool {
        self.first_layout.is_some() && self.second_layout.is_some()
    }

    /// Get the game winner if the game has ended.
    pub fn winner(&self) -> Option<usize> {
        self.winner
    }

    /// Get the other player in the game.
    pub fn other_player(&self, player: &usize) -> usize {
        match player {
            id if *id == self.first_player => self.second_player,
            id if *id == self.second_player => self.first_player,
            _ => panic!("player {} is not in this game", player),
        }
    }

    /// Shoot at position and get the result.
    pub fn shoot(&mut self, player: usize, position: Position) -> Result<ShootResult, GameError> {
        let (opponent, opponent_layout, opponent_board, opponent_fleet) = match player {
            id if id == self.second_player => (
                self.first_player,
                self.first_layout.as_ref().unwrap(),
                &mut self.first_board,
                &mut self.first_ships,
            ),
            id if id == self.first_player => (
                self.second_player,
                self.second_layout.as_ref().unwrap(),
                &mut self.second_board,
                &mut self.second_ships,
            ),
            _ => panic!("player {} is not in this game", player),
        };

        if let Some(_) = self.winner {
            panic!("game is over");
        }

        if player != self.on_turn {
            return Err(GameError::NotOnTurn);
        }

        // cell is already hit
        if let BoardCell::Hit = opponent_board[position.row() as usize][position.col() as usize] {
            return Ok(ShootResult::Hit);
        }

        let mut result = ShootResult::Missed;
        self.on_turn = opponent;

        // check if any ship is hit
        'outer: for r in 0..10 {
            for c in 0..10 {
                if let BoardCell::Ship(kind) = opponent_board[r as usize][c as usize] {
                    if position.row() == r && position.col() == c {
                        // ship is hit

                        self.on_turn = player;

                        let ship = opponent_fleet.get_mut(&kind).unwrap();
                        ship.hit();

                        if ship.is_sunk() {
                            result = ShootResult::Sunk(
                                kind,
                                opponent_layout
                                    .placements()
                                    .placements()
                                    .get(&kind)
                                    .unwrap()
                                    .clone(),
                            )
                        } else {
                            result = ShootResult::Hit;
                        }

                        break 'outer;
                    }
                }
            }
        }

        match result {
            ShootResult::Missed => {
                opponent_board[position.row() as usize][position.col() as usize] = BoardCell::Miss
            }
            ShootResult::Hit | ShootResult::Sunk(_, _) => {
                opponent_board[position.row() as usize][position.col() as usize] = BoardCell::Hit
            }
        }

        // check whether the all opponent ships are sunk
        self.winner = Some(player);
        for (_, ship) in opponent_fleet {
            if !ship.is_sunk() {
                self.winner = None;
            }
        }

        Ok(result)
    }

    /// Get the state of game for a concrete player.
    pub fn state(&self, player: usize) -> (Who, Hits, Hits, Layout, Hits, Hits, ShipsPlacements) {
        let (board, layout, opponent_board, opponent_layout, opponent_ships) = match player {
            id if id == self.second_player => (
                &self.second_board,
                self.second_layout.as_ref().unwrap(),
                &self.first_board,
                self.first_layout.as_ref().unwrap(),
                &self.first_ships,
            ),
            id if id == self.first_player => (
                &self.first_board,
                self.first_layout.as_ref().unwrap(),
                &self.second_board,
                self.second_layout.as_ref().unwrap(),
                &self.second_ships,
            ),
            _ => panic!("player {} is not in this game", player),
        };

        let on_turn = if player == self.on_turn {
            Who::You
        } else {
            Who::Opponent
        };
        let player_hits = Self::serialize_hits(board);
        let player_misses = Self::serialize_misses(board);
        let layout = layout.clone();
        let opponent_hits = Self::serialize_hits(opponent_board);
        let opponent_misses = Self::serialize_misses(opponent_board);
        let opponent_sunk_ships = Self::serialize_sunk(opponent_layout, opponent_ships);

        (
            on_turn,
            player_hits,
            player_misses,
            layout,
            opponent_hits,
            opponent_misses,
            opponent_sunk_ships,
        )
    }

    /// Serialize all board cells which are hit into the Hits structure.
    pub fn serialize_hits(board: &[[BoardCell; 10]; 10]) -> Hits {
        let mut hits = Vec::new();

        for r in 0..10 {
            for c in 0..10 {
                if let BoardCell::Hit = board[r as usize][c as usize] {
                    hits.push(Position::new(r, c).unwrap());
                }
            }
        }

        Hits::new(hits)
    }

    /// Serialize all board cells which are missed into the Hits structure.
    pub fn serialize_misses(board: &[[BoardCell; 10]; 10]) -> Hits {
        let mut hits = Vec::new();

        for r in 0..10 {
            for c in 0..10 {
                if let BoardCell::Miss = board[r as usize][c as usize] {
                    hits.push(Position::new(r, c).unwrap());
                }
            }
        }

        Hits::new(hits)
    }

    /// Serialize all ships which are sunk into a ShipsPlacements structure.
    pub fn serialize_sunk(layout: &Layout, ships: &HashMap<ShipKind, Ship>) -> ShipsPlacements {
        let mut placements = HashMap::new();

        for (kind, ship) in ships {
            if ship.is_sunk() {
                placements.insert(
                    *kind,
                    layout.placements().placements().get(&kind).unwrap().clone(),
                );
            }
        }

        ShipsPlacements::new(placements)
    }
}
