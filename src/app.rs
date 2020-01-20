use std::collections::{HashMap};
use crate::types::{Nickname, RestoreState, Layout, Position, Who};
use crate::proto::{ClientMessage, ServerMessage};
use crate::Command;
use crate::Command::{Message};
use log::{trace, debug, warn, info};
use crate::game::{Game, GameError, ShootResult};
use rand::Rng;
use std::time::{Instant, Duration};

pub struct App {
    /// Limit of maximum players.
    max_players: usize,
    /// Limit of maximum players.
    session_timeout: Duration,
    /// A player waiting for opponent.
    pending_player: Option<usize>,
    /// Player-id-to-nickname indexed by player ids.
    sessions_nicknames: HashMap<usize, String>,
    /// Player-id-to-last-active indexed by player ids.
    last_active: HashMap<usize, Instant>,
    /// Nickname-to-player-id indexed by nickname.
    nicknames_sessions: HashMap<String, usize>,
    /// Games storage indexed by games ids.
    games: HashMap<usize, Game>,
    /// Player-id-to-game map indexed by session ids.
    sessions_games: HashMap<usize, usize>,
    /// Peer-to-player-id map indexed by peer ids.
    peers_sessions: HashMap<usize, usize>,
    /// Player-id-to-peer map indexed by player ids.
    sessions_peers: HashMap<usize, usize>,
}

impl App {
    /// Create a new app.
    pub fn new(max_players: usize, session_timeout: Duration) -> Self {
        App {
            max_players,
            session_timeout,
            pending_player: None,
            sessions_nicknames: Default::default(),
            last_active: Default::default(),
            nicknames_sessions: Default::default(),
            games: Default::default(),
            sessions_games: Default::default(),
            peers_sessions: Default::default(),
            sessions_peers: Default::default(),
        }
    }

    /// Pass the message to the sub-handler based on the message type.
    pub fn handle_message(&mut self, peer_id: &usize, message: ClientMessage) -> Vec<Command> {
        match message {
            ClientMessage::Alive => self.handle_alive(&peer_id),
            ClientMessage::Login(nickname) => self.handle_login(&peer_id, nickname),
            ClientMessage::JoinGame => self.handle_join_game(&peer_id),
            ClientMessage::Layout(layout) => self.handle_layout(&peer_id, layout),
            ClientMessage::Shoot(position) => self.handle_shoot(&peer_id, position),
            ClientMessage::LeaveGame => self.handle_leave_game(&peer_id),
            ClientMessage::LogOut => self.handle_logout(&peer_id),
        }
    }

    /// Handle the alive command from the client.
    fn handle_alive(&mut self, peer_id: &usize) -> Vec<Command> {
        debug!("peer {:0>16X} is alive", peer_id);

        match self.peers_sessions.get(peer_id) {
            None => {
                trace!("no session")
            }
            Some(player_id) => {
                let nickname = self.sessions_nicknames.get(player_id).unwrap();
                trace!("logged as {}", nickname);
                {
                    let last_active = self.last_active.get_mut(player_id).unwrap();
                    *last_active = Instant::now();
                }
            }
        }

        vec![Message(*peer_id, ServerMessage::AliveOk)]
    }

    /// Handle login command from the client.
    fn handle_login(&mut self, peer_id: &usize, nickname: Nickname) -> Vec<Command> {
        debug!("peer {:0>16X} wants to login as {}", peer_id, nickname);
        let mut commands = Vec::new();

        match self.peers_sessions.get(peer_id) {
            None => {
                match self.nicknames_sessions.get(nickname.get()){
                    None => {
                        trace!("not registered yet - registering");

                        if self.nicknames_sessions.len() >= self.max_players {
                            warn!("registration of player {} refused because the maximum number of players is reached: {}",
                                  nickname.get(),
                                  self.max_players);

                            commands.push(Message(*peer_id, ServerMessage::LoginFull));
                        } else {
                            info!("peer {:0>16X} registered as {}", peer_id, nickname.get());

                            let player_id = self.unique_session_key();
                            self.nicknames_sessions.insert(nickname.get().clone(), player_id);
                            self.sessions_nicknames.insert(player_id, nickname.get().clone());
                            self.peers_sessions.insert(*peer_id, player_id);
                            self.sessions_peers.insert(player_id, *peer_id);
                            self.last_active.insert(player_id, Instant::now());

                            commands.push(Message(*peer_id, ServerMessage::LoginOk))
                        }
                    },
                    Some(player_id) => {
                        if let Some(id) = self.sessions_peers.get(player_id) {
                            warn!("{} is already registered and online with peer {:0>16X}", nickname.get(), id);
                            commands.push(Message(*peer_id, ServerMessage::LoginTaken));
                        } else {
                            info!("{} is already registered but offline - restoring the session with peer {}", nickname.get(), peer_id);

                            {
                                let last_active = self.last_active.get_mut(&player_id).unwrap();
                                *last_active = Instant::now();
                            }

                            self.sessions_peers.insert(*player_id, *peer_id);
                            self.peers_sessions.insert(*peer_id, *player_id);

                            match self.sessions_games.get(player_id) {
                                None => {
                                    trace!("not in any game");
                                    commands.push(Message(*peer_id, ServerMessage::LoginRestored(RestoreState::Lobby)));
                                }
                                Some(game_id) => {
                                    let game = self.games.get(game_id).unwrap();
                                    let opponent_id = &game.other_player(&player_id);
                                    let opponent_nickname = self.sessions_nicknames.get(opponent_id).unwrap();

                                    trace!("in game {:0>16X} - notifying opponent {}", game_id, opponent_nickname);

                                    if let Some(opponent_peer_id) = self.sessions_peers.get(opponent_id) {
                                        commands.push(Message(*opponent_peer_id, ServerMessage::OpponentReady))
                                    }

                                    let (
                                        on_turn,
                                        player_board_hits,
                                        player_board_misses,
                                        layout,
                                        opponent_board_hits,
                                        opponent_board_misses,
                                        sunk_ships
                                    ) = game.state(*player_id);

                                    commands.push(Message(*peer_id, ServerMessage::LoginRestored(RestoreState::Game {
                                        opponent: Nickname::new(opponent_nickname.clone()).unwrap(),
                                        on_turn,
                                        player_board_hits,
                                        player_board_misses,
                                        layout,
                                        opponent_board_hits,
                                        opponent_board_misses,
                                        sunk_ships,
                                    })));
                                }
                            }
                        }
                    },
                }
            }
            Some(_) => {
                warn!("peer {:0>16X} is already logged in as {}", peer_id, nickname.get());
                commands.push(Message(*peer_id, ServerMessage::IllegalState));
            }
        }

        commands
    }

    /// Handle join game command from the client.
    fn handle_join_game(&mut self, peer_id: &usize) -> Vec<Command> {
        let mut commands = Vec::new();

        match self.peers_sessions.get(peer_id).cloned() {
            Some(player_id) => {
                debug!("player {} wants to join a game", self.sessions_nicknames.get(&player_id).unwrap());

                {
                    let last_active = self.last_active.get_mut(&player_id).unwrap();
                    *last_active = Instant::now();
                }

                match self.sessions_games.get(&player_id) {
                    None => {
                        trace!("not in any game");

                        match self.pending_player {
                            None => {
                                info!("no pending player - {} is set as pending player", self.sessions_nicknames.get(&player_id).unwrap());

                                self.pending_player = Some(player_id);

                                commands.push(Message(*peer_id, ServerMessage::JoinGameWait))
                            }
                            Some(opponent_id) => {
                                if opponent_id == player_id {
                                    warn!("{} is already waiting for a game", self.sessions_nicknames.get(&player_id).unwrap());

                                    commands.push(Message(*peer_id, ServerMessage::IllegalState));
                                } else {
                                    let game = Game::new(opponent_id, player_id);
                                    let game_id = self.unique_game_id();
                                    self.games.insert(game_id, game);

                                    self.sessions_games.insert(player_id, game_id);
                                    self.sessions_games.insert(opponent_id, game_id);

                                    let nickname = self.sessions_nicknames.get(&player_id).unwrap();
                                    let opponent_nickname = self.sessions_nicknames.get(&opponent_id).unwrap();

                                    info!("a pending player {} is present - creating a game with {}", opponent_nickname, nickname);
                                    trace!("adding the game {:0>16X}", game_id);
                                    self.pending_player = None;

                                    let opponent_peer_id = self.sessions_peers.get(&opponent_id).unwrap();

                                    commands.push(Message(*opponent_peer_id, ServerMessage::OpponentJoined(Nickname::new(nickname.clone()).unwrap())));
                                    commands.push(Message(*peer_id, ServerMessage::JoinGameOk(Nickname::new(opponent_nickname.clone()).unwrap())));
                                }
                            }
                        }
                    }
                    Some(game_id) => {
                        warn!("{} is already in a game", self.sessions_nicknames.get(&player_id).unwrap());
                        commands.push(Message(*peer_id, ServerMessage::IllegalState));
                    }
                }
            }
            None => {
                warn!("peer {:0>16X} is not logged - can't join a game", peer_id);
                commands.push(Message(*peer_id, ServerMessage::IllegalState))
            }
        }

        return commands;
    }

    /// Handle the layout command from client
    fn handle_layout(&mut self, peer_id: &usize, layout: Layout) -> Vec<Command> {
        let mut commands = Vec::new();

        match self.peers_sessions.get(peer_id).cloned() {
            Some(player_id) => {
                debug!("player {} wants to choose a game layout", self.sessions_nicknames.get(&player_id).unwrap());
                trace!("layout: {}", layout);
                {
                    let last_active = self.last_active.get_mut(&player_id).unwrap();
                    *last_active = Instant::now();
                }

                match self.sessions_games.get(&player_id) {
                    None => {
                        trace!("not in game");
                        commands.push(Message(*peer_id, ServerMessage::IllegalState))
                    }
                    Some(game_id) => {
                        trace!("in game {}", game_id);

                        let game = self.games.get_mut(game_id).unwrap();

                        if game.playing() {
                            warn!("player {} is already playing - can't choose layout", self.sessions_nicknames.get(&player_id).unwrap());

                            commands.push(Message(*peer_id, ServerMessage::IllegalState))
                        } else {
                            match game.set_layout(player_id, layout) {
                                Ok(_) => {
                                    debug!("layout confirmed for the player {}", self.sessions_nicknames.get(&player_id).unwrap());

                                    let opponent_id = game.other_player(&player_id);
                                    let opponent_peer_id = self.sessions_peers.get(&opponent_id).unwrap();

                                    commands.push(Message(*peer_id, ServerMessage::LayoutOk));
                                    commands.push(Message(*opponent_peer_id, ServerMessage::OpponentReady));
                                }
                                Err(error) => {
                                    match error {
                                        GameError::AlreadyHasLayout => {
                                            warn!("player {} has already a layout", self.sessions_nicknames.get(&player_id).unwrap());
                                            commands.push(Message(*peer_id, ServerMessage::IllegalState))
                                        }
                                        GameError::InvalidLayout => {
                                            warn!("player {} has invalid layout", self.sessions_nicknames.get(&player_id).unwrap());
                                            commands.push(Message(*peer_id, ServerMessage::LayoutFail))
                                        }
                                        _ => {}
                                    }
                                }
                            }
                        }
                    }
                }
            }
            None => {
                warn!("peer {:0>16X} is not logged - can't choose a layout", peer_id);
                commands.push(Message(*peer_id, ServerMessage::IllegalState))
            }
        }

        return commands;
    }

    /// Handle the shoot command from client
    fn handle_shoot(&mut self, peer_id: &usize, position: Position) -> Vec<Command> {
        let mut commands = Vec::new();

        match self.peers_sessions.get(peer_id).cloned() {
            Some(player_id) => {
                debug!("player {} wants to shoot", self.sessions_nicknames.get(&player_id).unwrap());
                trace!("position: {}", position);

                {
                    let last_active = self.last_active.get_mut(&player_id).unwrap();
                    *last_active = Instant::now();
                }

                match self.sessions_games.get(&player_id) {
                    None => {
                        warn!("player {} is not in a game - can't shoot", self.sessions_nicknames.get(&player_id).unwrap());
                        commands.push(Message(*peer_id, ServerMessage::IllegalState))
                    }
                    Some(game_id) => {
                        trace!("in game {}", game_id);

                        let game = self.games.get_mut(game_id).unwrap();

                        if !game.playing() {
                            warn!("player {} can't shoot while layouting", self.sessions_nicknames.get(&player_id).unwrap());
                            commands.push(Message(*peer_id, ServerMessage::IllegalState))
                        } else {
                            match game.shoot(player_id, position) {
                                Ok(result) => {
                                    let opponent_id = game.other_player(&player_id);

                                    match result {
                                        ShootResult::Missed => {
                                            debug!("missed");

                                            commands.push(Message(*peer_id, ServerMessage::ShootMissed));
                                            if let Some(opponent_peer_id) = self.sessions_peers.get(&opponent_id) {
                                                commands.push(Message(*opponent_peer_id, ServerMessage::OpponentMissed(position)));
                                            }
                                        }
                                        ShootResult::Hit => {
                                            debug!("hit");

                                            commands.push(Message(*peer_id, ServerMessage::ShootHit));
                                            if let Some(opponent_peer_id) = self.sessions_peers.get(&opponent_id) {
                                                commands.push(Message(*opponent_peer_id, ServerMessage::OpponentHit(position)));
                                            }
                                        }
                                        ShootResult::Sunk(ship_kind, placement) => {
                                            debug!("sunk a ship {} at {}", ship_kind, placement);

                                            commands.push(Message(*peer_id, ServerMessage::ShootSunk(ship_kind, placement)));
                                            if let Some(opponent_peer_id) = self.sessions_peers.get(&opponent_id) {
                                                commands.push(Message(*opponent_peer_id, ServerMessage::OpponentHit(position)));
                                            }
                                        }
                                    }

                                    if let Some(winner) = game.winner() {
                                        info!("{} vs {} - game over, winner: {}",
                                              self.sessions_nicknames.get(&player_id).unwrap(),
                                              self.sessions_nicknames.get(&opponent_id).unwrap(),
                                              self.sessions_nicknames.get(&winner).unwrap());

                                        commands.push(Message(
                                            *peer_id,
                                            ServerMessage::GameOver(
                                                if winner == player_id { Who::You } else { Who::Opponent }
                                            )));

                                        if let Some(opponent_peer_id) = self.sessions_peers.get(&opponent_id) {
                                            commands.push(Message(
                                                *opponent_peer_id,
                                                ServerMessage::GameOver(
                                                    if winner == opponent_id { Who::You } else { Who::Opponent }
                                                )));
                                        }

                                        trace!("removing the game {:0>16X}", game_id);

                                        self.games.remove(game_id);
                                        self.sessions_games.remove(&player_id);
                                        self.sessions_games.remove(&opponent_id);
                                    }
                                }
                                Err(_) => {
                                    warn!("player {} is not on turn", self.sessions_nicknames.get(&player_id).unwrap());
                                    commands.push(Message(*peer_id, ServerMessage::IllegalState))
                                }
                            }
                        }
                    }
                }
            }
            None => {
                warn!("peer {:0>16X} is not logged - can't shoot", peer_id);
                commands.push(Message(*peer_id, ServerMessage::IllegalState))
            }
        }

        return commands;
    }

    /// Handle the leave game command from client
    fn handle_leave_game(&mut self, peer_id: &usize) -> Vec<Command> {
        let mut commands = Vec::new();

        match self.peers_sessions.get(peer_id).cloned() {
            Some(player_id) => {
                debug!("player {} wants to leave the game", self.sessions_nicknames.get(&player_id).unwrap());
                {
                    let last_active = self.last_active.get_mut(&player_id).unwrap();
                    *last_active = Instant::now();
                }

                match self.sessions_games.get(&player_id) {
                    None => {
                        match self.pending_player {
                            None => {
                                warn!("player {} is not in a game - can't leave any", self.sessions_nicknames.get(&player_id).unwrap());

                                commands.push(Message(*peer_id, ServerMessage::IllegalState))
                            }
                            Some(pending_player_id) => {
                                if pending_player_id == player_id {
                                    info!("removing player {} from game pending queue", self.sessions_nicknames.get(&player_id).unwrap());

                                    self.pending_player = None;

                                    commands.push(Message(*peer_id, ServerMessage::LeaveGameOk));
                                }
                            }
                        }
                    }
                    Some(game_id) => {
                        let game = self.games.remove(game_id).unwrap();
                        let opponent_id = &game.other_player(&player_id);

                        info!("removing player {} from game with {}",
                              self.sessions_nicknames.get(&player_id).unwrap(),
                              self.sessions_nicknames.get(opponent_id).unwrap());
                        trace!("notifying opponent");

                        self.sessions_games.remove(&player_id);
                        self.sessions_games.remove(opponent_id);

                        if let Some(opponent_peer_id) = self.sessions_peers.get(&opponent_id) {
                            commands.push(Message(*opponent_peer_id, ServerMessage::OpponentLeft))
                        }

                        commands.push(Message(*peer_id, ServerMessage::LeaveGameOk));
                    }
                }
            }
            None => {
                warn!("peer {:0>16X} is not logged - can't join a game", peer_id);
                commands.push(Message(*peer_id, ServerMessage::IllegalState))
            }
        }

        return commands;
    }

    /// Handle logout command from the client.
    fn handle_logout(&mut self, peer_id: &usize) -> Vec<Command> {
        let mut commands = Vec::new();

        match self.peers_sessions.get(peer_id).cloned() {
            None => {
                warn!("peer {:0>16X} is not logged - can't logout", peer_id);
                commands.push(Message(*peer_id, ServerMessage::IllegalState))
            }
            Some(player_id) => {
                info!("logging out the player {}", self.sessions_nicknames.get(&player_id).unwrap());

                // handle if the session is in any game
                match self.sessions_games.get(&player_id) {
                    None => {
                        if let Some(pending_player_id) = self.pending_player {
                            if pending_player_id == player_id {
                                info!("removing player {} from game pending queue", self.sessions_nicknames.get(&player_id).unwrap());
                                self.pending_player = None;
                            }
                        } else {
                            trace!("not in any game");
                        }
                    }
                    Some(game_id) => {
                        let game = self.games.remove(&game_id).unwrap();
                        let opponent_id = game.other_player(&player_id);

                        info!("removing player {} from game with {}",
                              self.sessions_nicknames.get(&player_id).unwrap(),
                              self.sessions_nicknames.get(&opponent_id).unwrap());
                        trace!("notifying opponent");

                        self.sessions_games.remove(&player_id);
                        self.sessions_games.remove(&opponent_id);

                        if let Some(opponent_peer_id) = self.sessions_peers.get(&opponent_id) {
                            commands.push(Message(*opponent_peer_id, ServerMessage::OpponentLeft))
                        }
                    }
                }

                self.nicknames_sessions.remove(self.sessions_nicknames.get(&player_id).unwrap());
                self.sessions_nicknames.remove(&player_id);
                self.sessions_peers.remove(&player_id);
                self.peers_sessions.remove(&peer_id);
                self.last_active.remove(&player_id);

                commands.push(Message(*peer_id, ServerMessage::LogoutOk));
            }
        }

        commands
    }

    /// Handle the peer socket disconnection.
    pub fn handle_offline(&mut self, peer_id: &usize) -> Vec<Command> {
        let mut commands = Vec::new();

        match self.peers_sessions.get(&peer_id).cloned() {
            None => {
                //
            }
            Some(player_id) => {
                info!("player {} is offline", self.sessions_nicknames.get(&player_id).unwrap());

                // handle if the session is in any game
                match self.sessions_games.get(&player_id).cloned() {
                    None => {

                        if let Some(pending_player_id) = self.pending_player {
                            if pending_player_id == player_id {
                                info!("removing player {} from game pending queue", self.sessions_nicknames.get(&player_id).unwrap());
                                self.pending_player = None;
                            }
                        } else {
                            trace!("not in any game");
                        }
                    }
                    Some(game_id) => {
                        let game = self.games.get(&game_id).unwrap();
                        let opponent_id = game.other_player(&player_id);

                        if !game.playing() {
                            info!("removing player {} from the non-started game with {}",
                                  self.sessions_nicknames.get(&player_id).unwrap(),
                                  self.sessions_nicknames.get(&opponent_id).unwrap());
                            trace!("notifying opponent");

                            self.sessions_games.remove(&player_id);
                            self.sessions_games.remove(&opponent_id);
                            self.games.remove(&game_id);

                            if let Some(opponent_peer_id) = self.sessions_peers.get(&opponent_id) {
                                commands.push(Message(*opponent_peer_id, ServerMessage::OpponentLeft))
                            }
                        } else {
                            trace!("in the game with {} - notifying", self.sessions_nicknames.get(&opponent_id).unwrap());

                            if let Some(opponent_peer_id) = self.sessions_peers.get(&opponent_id) {
                                commands.push(Message(*opponent_peer_id, ServerMessage::OpponentOffline))
                            }
                        }
                    }
                }

                self.sessions_peers.remove(&player_id);
                self.peers_sessions.remove(&peer_id);
            }
        }

        commands
    }

    /// Do clean up of inactive sessions.
    pub fn handle_cleanup(&mut self) -> Vec<Command> {
        let mut commands = Vec::new();

        let now = Instant::now();

        let to_remove = self.last_active.iter().filter_map(|(player_id, last_active)| {
            let inactive = now.duration_since(*last_active);

            if inactive >= self.session_timeout {
                Some(*player_id)
            } else {
                None
            }

        }).collect::<Vec<_>>();

        to_remove.iter().for_each(|player_id| {
            let nickname = self.sessions_nicknames.get(player_id).unwrap();
            warn!("removing player {} - inactive for too long", nickname);

            self.nicknames_sessions.remove(nickname);
            self.sessions_nicknames.remove(player_id);
            self.last_active.remove(player_id);

            if let Some(peer_id) = self.sessions_peers.remove(player_id) {
                self.peers_sessions.remove(&peer_id);
                commands.push(Command::Close(peer_id));
            }

            // handle if the session is in any game
            match self.sessions_games.get(player_id) {
                None => {
                    if let Some(pending_player_id) = self.pending_player {
                        if pending_player_id == *player_id {
                            info!("removing player {} from game pending queue", self.sessions_nicknames.get(&player_id).unwrap());
                            self.pending_player = None;
                        }
                    } else {
                        trace!("not in any game");
                    }
                }
                Some(game_id) => {
                    let game = self.games.remove(&game_id).unwrap();
                    let opponent_id = game.other_player(player_id);

                    info!("removing player {} from game with {}",
                          self.sessions_nicknames.get(&player_id).unwrap(),
                          self.sessions_nicknames.get(&opponent_id).unwrap());
                    trace!("notifying opponent");

                    self.sessions_games.remove(&player_id);
                    self.sessions_games.remove(&opponent_id);

                    if let Some(opponent_peer_id) = self.sessions_peers.get(&opponent_id) {
                        commands.push(Message(*opponent_peer_id, ServerMessage::OpponentLeft))
                    }
                }
            }
        });

        commands
    }

    /// Do clean up of inactive sessions.
    pub fn handle_shutdown(&mut self) -> Vec<Command> {
        info!("executing shutdown cleanup");

        let mut commands = Vec::new();

        for (player_id, peer_id) in self.sessions_peers.drain() {
            debug!("notifying player {} about disconnection", self.sessions_nicknames.get(&player_id).unwrap());
            commands.push(Message(peer_id, ServerMessage::Disconnect));
        }

        self.peers_sessions.clear();
        self.nicknames_sessions.clear();
        self.sessions_nicknames.clear();
        self.last_active.clear();
        self.games.clear();
        self.sessions_games.clear();

        commands
    }

    /// Get a unique id for a session.
    fn unique_session_key(&self) -> usize {
        loop {
            let key = rand::thread_rng().gen();
            if !self.sessions_nicknames.contains_key(&key) {
                break key;
            }
        }
    }

    /// Get a unique id for a game.
    fn unique_game_id(&self) -> usize {
        loop {
            let id = rand::thread_rng().gen();
            if !self.games.contains_key(&id) {
                break id;
            }
        }
    }
}