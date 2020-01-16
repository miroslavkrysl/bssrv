use crate::types::{Layout, Placement, ShipsPlacements, Who, Hits, Position, ShipKind};
use std::collections::HashMap;

pub enum Game {
    Pending(GamePending),
    Layouting(GameLayouting),
    Playing(GamePlaying)
}


pub struct GamePending {
    first_player: u64
}

impl GamePending {
    pub fn new(first_player: u64) -> Self {
        GamePending {
            first_player
        }
    }

    pub fn add_second_player(self, second_player: u64) -> GameLayouting {
        GameLayouting::new(self.first_player, second_player)
    }
}


pub struct GameLayouting {
    first_player: u64,
    second_player: u64,
    first_layout: Option<Layout>,
    second_layout: Option<Layout>,
}

impl GameLayouting {
    fn new(first_player: u64, second_player: u64) -> Self {
        GameLayouting {
            first_player,
            second_player,
            first_layout: None,
            second_layout: None,
        }
    }

    pub fn set_layout(&mut self, player: u64, layout: Layout) -> Result<bool, GameError> {
        let l = match player {
            id if id == self.first_player => {
                &mut self.first_layout
            }
            id if id == self.second_player => {
                &mut self.second_layout
            }
            _ => {
                panic!("player {} is not in this game", player)
            }
        };

        if l.is_some() {
            return Err(GameError::AlreadyHasLayout);
        }

        if !Self::is_valid_layout(&layout) {
            return Err(GameError::InvalidLayout);
        }

        *l = Some(layout);

        Ok(self.first_layout.is_some() && self.second_layout.is_some())
    }

    fn is_valid_layout(layout: &Layout) -> bool {
        // TODO: validate layout
        return true;
    }

    pub fn start(self) -> GamePlaying {
        GamePlaying::new(
            self.first_player,
            self.second_player,
            self.first_layout.unwrap(),
            self.second_layout.unwrap())
    }
}


pub struct GamePlaying {
    first_player: u64,
    second_player: u64,
    first_layout: Layout,
    second_layout: Layout,
    first_board: [[bool; 10]; 10],
    second_board: [[bool; 10]; 10],
    first_fleet: HashMap<ShipKind, u8>,
    second_fleet: HashMap<ShipKind, u8>,
    on_turn: u64,
}

impl GamePlaying {
    fn new(first_player: u64, second_player: u64, first_layout: Layout, second_layout: Layout) -> Self {
        let mut fleet = HashMap::new();
        fleet.insert(ShipKind::AircraftCarrier, ShipKind::AircraftCarrier.cells());
        fleet.insert(ShipKind::Battleship, ShipKind::Battleship.cells());
        fleet.insert(ShipKind::Cruiser, ShipKind::Cruiser.cells());
        fleet.insert(ShipKind::Destroyer, ShipKind::Destroyer.cells());
        fleet.insert(ShipKind::PatrolBoat, ShipKind::PatrolBoat.cells());

        GamePlaying {
            first_player,
            second_player,
            first_layout,
            second_layout,
            first_board: [[false; 10]; 10],
            second_board: [[false; 10]; 10],
            first_fleet: fleet.clone(),
            second_fleet: fleet,
            on_turn: first_player,
        }
    }

    pub fn other_player(&self, player: u64) -> u64 {
        match player {
            id if id == self.first_player => {
                self.second_player
            }
            id if id == self.second_player => {
                self.first_player
            }
            _ => {
                panic!("player {} is not in this game", player)
            }
        }
    }

    pub fn shoot(&mut self, player: u64, position: Position) -> Result<ShootResult, GameError> {
        let (opponent, opponent_layout, opponent_board, opponent_fleet) = match player {
            id if id == self.second_player => {
                (&mut self.first_player, &mut self.first_layout, &mut self.first_board, &mut self.first_fleet)
            }
            id if id == self.first_player => {
                (&mut self.second_player, &mut self.second_layout, &mut self.second_board, &mut self.second_fleet)
            }
            _ => {
                panic!("player {} is not in this game", player)
            }
        };

        if player != self.on_turn {
            return Err(GameError::NotOnTurn)
        }

        if opponent_board[position.row() as usize][position.col() as usize] {
            return Ok(ShootResult::Hit);
        }

        let hit = false;

        // TODO: hit ship

        for (kind, health) in opponent_fleet {
            if *health == 0 {
                return Ok(ShootResult::GameOver(player))
            }
        }

        Ok(if hit {ShootResult::Hit} else {ShootResult::Missed})
    }

    pub fn get_state(&self, player: u64) -> (Who, Hits, Layout, Hits, ShipsPlacements) {
        // TODO: implement get game state
    }
}

pub enum ShootResult {
    Missed,
    Hit,
    Sunk(Placement),
    GameOver(u64),
}

pub enum GameError {
    AlreadyHasLayout,
    InvalidLayout,
    NotOnTurn,
}