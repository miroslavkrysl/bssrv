pub struct Game {
    first_player: u64,
    second_player: Option<u64>
}

impl Game {
    pub fn new(first_player: u64) -> Self {
        Game {
            first_player,
            second_player: None
        }
    }
    
    pub fn add_second_player(&mut self, second_player: u64) {
        match self.second_player {
            None => {
                self.second_player = Some(second_player)
            },
            Some(_) => {
                panic!("second player is already in game")
            },
        }
    }

    pub fn first_player(&mut self) -> u64 {
        self.first_player
    }
}