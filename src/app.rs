use std::collections::HashMap;
use crate::types::{SessionKey, Nickname, RestoreState, Layout, Position, Who};
use crate::session::Session;
use crate::proto::{ClientMessage, ServerMessage};
use crate::Command;
use crate::Command::Message;
use std::cell::{RefCell, RefMut};
use log::{trace,debug,warn, info};
use crate::game::{Game, GameError, ShootResult};
use rand::Rng;

pub struct App {
    /// Limit of maximum players.
    max_players: usize,
    /// A player waiting for opponent.
    pending_player: Option<u64>,
    /// Sessions storage indexed by session keys.
    sessions: HashMap<u64, Session>,
    /// Games storage indexed by games ids.
    games: HashMap<usize, Game>,
    /// Session-to-game map indexed by session ids.
    sessions_games: HashMap<u64, usize>,
    /// Peer-to-session map indexed by pees ids.
    peers_sessions: HashMap<usize, u64>,
    /// Session-to-peer map indexed by session ids.
    sessions_peers: HashMap<u64, usize>,
}

impl App {
    pub fn new(max_players: usize) -> Self {
        App {
            max_players,
            pending_player: None,
            sessions: Default::default(),
            games: Default::default(),
            sessions_games: Default::default(),
            peers_sessions: Default::default(),
            sessions_peers: Default::default()
        }
    }

    pub fn handle_message(&mut self, peer_id: &usize, message: ClientMessage) -> Vec<Command> {
        info!("Message from peer {:0>16X}: {}", peer_id, message);

        match message {
            ClientMessage::Alive => return self.handle_alive(&peer_id),
            ClientMessage::RestoreSession(session_key) => return self.handle_restore_session(&peer_id, session_key),
            ClientMessage::Login(nickname) => return self.handle_login(&peer_id, nickname),
            ClientMessage::JoinGame => return self.handle_join_game(&peer_id),
            ClientMessage::Layout(layout) => return self.handle_layout(&peer_id, layout),
            ClientMessage::Shoot(position) => return self.handle_shoot(&peer_id, position),
            ClientMessage::LeaveGame => return self.handle_leave_game(&peer_id),
            ClientMessage::LogOut => return self.handle_logout(&peer_id),
            _ => return vec![]
        }
    }

    /// Handle the alive command from the client.
    fn handle_alive(&mut self, peer_id: &usize) -> Vec<Command> {
        debug!("peer {:0>16X} is alive", peer_id);

        match self.peers_sessions.get(peer_id) {
            None => {
                trace!("no session")
            },
            Some(session_key) => {
                trace!("with session {:0>16X}", session_key);
                self.sessions.get_mut(&session_key).unwrap().update_last_active();
            },
        }

        vec![Message(*peer_id, ServerMessage::AliveOk)]
    }

    /// Handle the restore session command from client.
    fn handle_restore_session(&mut self, peer_id: &usize, session_key: SessionKey) -> Vec<Command> {
        debug!("peer {:0>16X} wants to restore session", peer_id);
        let mut commands = Vec::new();

        match self.peers_sessions.get(peer_id) {
            None => {
                let session_key = session_key.get();

                match self.sessions.get_mut(&session_key) {
                    Some(session) => {
                        trace!("session with key {:0>16X} found", session_key);

                        if let Some(id) = self.sessions_peers.get(&session_key) {
                            warn!("session is already online with peer {:0>16X}", id);
                            commands.push(Message(*peer_id, ServerMessage::RestoreSessionFail));
                        } else {
                            session.update_last_active();
                            self.sessions_peers.insert(session_key, *peer_id);
                            self.peers_sessions.insert(*peer_id, session_key);

                            match self.sessions_games.get(&session_key) {
                                None => {
                                    trace!("not in any game");
                                    commands.push(Message(*peer_id, ServerMessage::RestoreSessionOk(RestoreState::Lobby)));
                                },
                                Some(game_id) => {
                                    trace!("in game {:0>16X} - notifying opponent", game_id);

                                    let game = self.games.get(game_id).unwrap();
                                    let opponent_session_key = &game.other_player(&session_key);

                                    if let Some(opponent_peer_id) = self.sessions_peers.get(&opponent_session_key) {
                                        commands.push(Message(*opponent_peer_id, ServerMessage::OpponentReady))
                                    }

                                    let (
                                        on_turn,
                                        player_board,
                                        layout,
                                        opponent_board,
                                        sunk_ships
                                    ) = game.state(session_key);

                                    commands.push(Message(*peer_id, ServerMessage::RestoreSessionOk(RestoreState::Game {
                                        on_turn,
                                        player_board,
                                        layout,
                                        opponent_board,
                                        sunk_ships
                                    })));
                                },
                            }
                        }
                    },
                    None => {
                        warn!("no session with key {:0>16X}", session_key);
                        commands.push(Message(*peer_id, ServerMessage::RestoreSessionFail));
                    }
                }
            },
            Some(session_key) => {
                warn!("already logged with session {:0>16X}", session_key);

                self.sessions.get_mut(&session_key).unwrap().update_last_active();
                commands.push(Message(*peer_id, ServerMessage::IllegalState));
            },
        }

        commands
    }

    /// Handle login command from the client.
    fn handle_login(&mut self, peer_id: &usize, nickname: Nickname) -> Vec<Command> {
        debug!("peer {:0>16X} wants to login", peer_id);
        trace!("nickname: {}", nickname);
        let mut commands = Vec::new();

        match self.peers_sessions.get(peer_id) {
            None => {
                if self.sessions.len() >= self.max_players {
                    warn!("refused because the maximum number of players is reached: {}", self.max_players);
                    commands.push(Message(*peer_id, ServerMessage::LoginFail));
                } else {
                    let session_key = self.unique_session_key();
                    self.sessions.insert(session_key, Session::new(nickname));
                    self.peers_sessions.insert(*peer_id, session_key);
                    self.sessions_peers.insert(session_key, *peer_id);

                    trace!("logged in session {:0>16X}", session_key);

                    commands.push(Message(*peer_id, ServerMessage::LoginOk(SessionKey::new(session_key))))
                }
            },
            Some(_) => {
                warn!("already logged in");
                commands.push(Message(*peer_id, ServerMessage::IllegalState));
            },
        }

        commands
    }

    /// Handle join game command from the client.
    fn handle_join_game(&mut self, peer_id: &usize) -> Vec<Command> {
        debug!("peer {} wants to join a game", peer_id);
        let mut commands = Vec::new();

        match self.peers_sessions.get(peer_id).cloned() {
            Some(session_key) => {
                trace!("logged with session {:0>16X}", session_key);
                self.sessions.get_mut(&session_key).unwrap().update_last_active();

                match self.sessions_games.get(&session_key) {
                    None => {
                        trace!("not in any game");

                        match self.pending_player {
                            None => {
                                trace!("no pending player - set as pending");

                                self.pending_player = Some(session_key);

                                commands.push(Message(*peer_id, ServerMessage::JoinGameWait))
                            },
                            Some(opponent_session_key) => {
                                if opponent_session_key == session_key {
                                    warn!("already waiting for a game");

                                    commands.push(Message(*peer_id, ServerMessage::IllegalState));
                                } else {
                                    let game = Game::new(opponent_session_key, session_key);
                                    let game_id = self.unique_game_id();
                                    self.games.insert(game_id, game);

                                    self.sessions_games.insert(session_key, game_id);
                                    self.sessions_games.insert(opponent_session_key, game_id);

                                    trace!("a pending player {:0>16X} is present - creating game", opponent_session_key);

                                    self.pending_player = None;

                                    let opponent_peer_id = self.sessions_peers.get(&opponent_session_key).unwrap();
                                    let session = self.sessions.get(&session_key).unwrap();
                                    let opponent = self.sessions.get(&opponent_session_key).unwrap();

                                    commands.push(Message(*opponent_peer_id, ServerMessage::OpponentJoined(session.nickname().clone())));
                                    commands.push(Message(*peer_id, ServerMessage::JoinGameOk(opponent.nickname().clone())));
                                }
                            },
                        }
                    },
                    Some(game_id) => {
                        warn!("already in game {}", game_id);
                        commands.push(Message(*peer_id, ServerMessage::IllegalState));
                    },
                }
            }
            None => {
                warn!("not logged - can't join a game");
                commands.push(Message(*peer_id, ServerMessage::IllegalState))
            }
        }

        return commands
    }

    /// Handle the layout command from client
    fn handle_layout(&mut self, peer_id: &usize, layout: Layout) -> Vec<Command> {
        debug!("peer {} wants to choose a game layout", peer_id);
        trace!("layout: {}", layout);
        let mut commands = Vec::new();

        match self.peers_sessions.get(peer_id).cloned() {
            Some(session_key) => {
                trace!("logged with session {:0>16X}", session_key);
                self.sessions.get_mut(&session_key).unwrap().update_last_active();

                match self.sessions_games.get(&session_key) {
                    None => {
                        trace!("not in game");
                        commands.push(Message(*peer_id, ServerMessage::IllegalState))
                    },
                    Some(game_id) => {
                        trace!("in game {}", game_id);

                        let game = self.games.get_mut(game_id).unwrap();

                        if game.playing() {
                            warn!("already playing - can't choose layout");
                            commands.push(Message(*peer_id, ServerMessage::IllegalState))
                        } else {
                            match game.set_layout(session_key, layout) {
                                Ok(_) => {
                                    trace!("layout set");

                                    let opponent_session_key = game.other_player(&session_key);
                                    let opponent_peer_id = self.sessions_peers.get(&opponent_session_key).unwrap();

                                    commands.push(Message(*peer_id, ServerMessage::LayoutOk));
                                    commands.push(Message(*opponent_peer_id, ServerMessage::OpponentReady));
                                },
                                Err(error) => {
                                    match error {
                                        GameError::AlreadyHasLayout => {
                                            warn!("already has a layout");
                                            commands.push(Message(*peer_id, ServerMessage::IllegalState))
                                        },
                                        GameError::InvalidLayout => {
                                            warn!("layout is invalid");
                                            commands.push(Message(*peer_id, ServerMessage::LayoutFail))
                                        },
                                        _ => {},
                                    }
                                }
                            }
                        }
                    },
                }
            }
            None => {
                warn!("not logged - can't choose layout");
                commands.push(Message(*peer_id, ServerMessage::IllegalState))
            }
        }

        return commands
    }

    /// Handle the shoot command from client
    fn handle_shoot(&mut self, peer_id: &usize, position: Position) -> Vec<Command> {
        debug!("peer {} wants to shoot", peer_id);
        trace!("position: {}", position);

        let mut commands = Vec::new();

        match self.peers_sessions.get(peer_id).cloned() {
            Some(session_key) => {
                trace!("logged with session {:0>16X}", session_key);
                self.sessions.get_mut(&session_key).unwrap().update_last_active();

                match self.sessions_games.get(&session_key) {
                    None => {
                        trace!("not in game");
                        commands.push(Message(*peer_id, ServerMessage::IllegalState))
                    },
                    Some(game_id) => {
                        trace!("in game {}", game_id);

                        let game = self.games.get_mut(game_id).unwrap();

                        if !game.playing() {
                            warn!("not playing - can't shoot");
                            commands.push(Message(*peer_id, ServerMessage::IllegalState))
                        } else {
                            match game.shoot(session_key, position) {
                                Ok(result) => {
                                    let opponent_session_key = game.other_player(&session_key);

                                    match result {
                                        ShootResult::Missed => {
                                            trace!("missed");

                                            commands.push(Message(*peer_id, ServerMessage::ShootMissed));
                                            if let Some(opponent_peer_id) = self.sessions_peers.get(&opponent_session_key) {
                                                commands.push(Message(*opponent_peer_id, ServerMessage::OpponentMissed(position)));
                                            }
                                        },
                                        ShootResult::Hit => {
                                            trace!("hit");

                                            commands.push(Message(*peer_id, ServerMessage::ShootHit));
                                            if let Some(opponent_peer_id) = self.sessions_peers.get(&opponent_session_key) {
                                                commands.push(Message(*opponent_peer_id, ServerMessage::OpponentHit(position)));
                                            }

                                        },
                                        ShootResult::Sunk(ship_kind, placement) => {
                                            trace!("sunk a ship");

                                            commands.push(Message(*peer_id, ServerMessage::ShootSunk(ship_kind, placement)));
                                            if let Some(opponent_peer_id) = self.sessions_peers.get(&opponent_session_key) {
                                                commands.push(Message(*opponent_peer_id, ServerMessage::OpponentHit(position)));
                                            }
                                        },
                                    }

                                    if let Some(winner) = game.winner() {
                                        trace!("game over, winner: {:0>16X}", winner);

                                        commands.push(Message(
                                            *peer_id,
                                            ServerMessage::GameOver(
                                                if winner == session_key {Who::You} else {Who::Opponent}
                                            )));

                                        if let Some(opponent_peer_id) = self.sessions_peers.get(&opponent_session_key) {
                                            commands.push(Message(
                                                *opponent_peer_id,
                                                ServerMessage::GameOver(
                                                    if winner == opponent_session_key {Who::You} else {Who::Opponent}
                                                )));
                                        }

                                        trace!("removing the game {:0>16X}", game_id);

                                        self.games.remove(game_id);
                                        self.sessions_games.remove(&session_key);
                                        self.sessions_games.remove(&opponent_session_key);
                                    }
                                },
                                Err(_) => {
                                    warn!("not on turn");
                                    commands.push(Message(*peer_id, ServerMessage::IllegalState))
                                }
                            }
                        }
                    },
                }
            }
            None => {
                warn!("not logged - can't choose layout");
                commands.push(Message(*peer_id, ServerMessage::IllegalState))
            }
        }

        return commands
    }

    /// Handle the leave game command from client
    fn handle_leave_game(&mut self, peer_id: &usize) -> Vec<Command> {
        debug!("peer {} wants to leave the game", peer_id);
        let mut commands = Vec::new();

        match self.peers_sessions.get(peer_id).cloned() {
            Some(session_key) => {
                trace!("logged with session {:0>16X}", session_key);
                self.sessions.get_mut(&session_key).unwrap().update_last_active();

                match self.sessions_games.get(&session_key) {
                    None => {
                        trace!("not in game");

                        match self.pending_player {
                            None => {
                                trace!("not waiting for a game");
                                warn!("can't leave - not in any a game");

                                commands.push(Message(*peer_id, ServerMessage::IllegalState))
                            },
                            Some(pending_session_key) => {
                                if pending_session_key == session_key {
                                    trace!("waiting for a game - removing");

                                    self.pending_player = None;

                                    commands.push(Message(*peer_id, ServerMessage::LeaveGameOk));
                                }
                            },
                        }
                    },
                    Some(game_id) => {
                        trace!("in game {} - removing game and notifying opponent", game_id);

                        let game = self.games.remove(game_id).unwrap();
                        let opponent_session_key = &game.other_player(&session_key);

                        self.sessions_games.remove(&session_key);
                        self.sessions_games.remove(opponent_session_key);

                        if let Some(opponent_peer_id) = self.sessions_peers.get(&opponent_session_key) {
                            commands.push(Message(*opponent_peer_id, ServerMessage::OpponentLeft))
                        }

                        commands.push(Message(*peer_id, ServerMessage::LeaveGameOk));
                    },
                }
            }
            None => {
                warn!("not logged - can't join a game");
                commands.push(Message(*peer_id, ServerMessage::IllegalState))
            }
        }

        return commands
    }

    /// Handle logout command from the client.
    fn handle_logout(&mut self, peer_id: &usize) -> Vec<Command> {
        debug!("peer {:0>16X} wants to logout", peer_id);

        let mut commands = Vec::new();

        match self.peers_sessions.get(peer_id).cloned() {
            None => {
                warn!("can't log out because not logged yet");
                commands.push(Message(*peer_id, ServerMessage::IllegalState))
            },
            Some(session_key) => {
                trace!("session {:0>16X}", session_key);

                // handle if the session is in any game
                match self.sessions_games.get(&session_key) {
                    None => {
                        trace!("not in any game");

                        if let Some(player) = self.pending_player {
                            if player == session_key {
                                trace!("waiting for a game - removing");
                                self.pending_player = None;
                            }
                        }
                    },
                    Some(game_id) => {
                        let game = self.games.remove(&game_id).unwrap();
                        let opponent_session_key = game.other_player(&session_key);

                        trace!("in a game {:0>16X} - removing game and notifying opponent {}", game_id, opponent_session_key);

                        self.sessions_games.remove(&session_key);
                        self.sessions_games.remove(&opponent_session_key);

                        if let Some(opponent_peer_id) = self.sessions_peers.get(&opponent_session_key) {
                            commands.push(Message(*opponent_peer_id, ServerMessage::OpponentLeft))
                        }
                    },
                }

                self.sessions.remove(&session_key);
                self.sessions_peers.remove(&session_key);
                self.peers_sessions.remove(&peer_id);

                commands.push(Message(*peer_id, ServerMessage::LogoutOk));
            },
        }

        commands
    }

    /// Handle the peer socket disconnection.
    pub fn handle_offline(&mut self, peer_id: &usize) -> Vec<Command> {
        debug!("peer {:0>16X} switching to offline", peer_id);

        let mut commands = Vec::new();

        match self.peers_sessions.get(&peer_id).cloned() {
            None => {
                trace!("no session");
            },
            Some(session_key) => {
                trace!("session {:0>16X}", session_key);

                // handle if the session is in any game
                match self.sessions_games.get(&session_key).cloned() {
                    None => {
                        trace!("not in any game");

                        if let Some(player) = self.pending_player {
                            if player == session_key {
                                trace!("waiting for a game - removing");
                                self.pending_player = None;
                            }
                        }
                    },
                    Some(game_id) => {
                        let game = self.games.get(&game_id).unwrap();
                        let opponent_session_key = game.other_player(&session_key);

                        if !game.playing() {
                            trace!("in the non-started game {:0>16X} - removing game and notifying opponent {:0>16X}", game_id, &opponent_session_key);

                            self.sessions_games.remove(&session_key);
                            self.sessions_games.remove(&opponent_session_key);
                            self.games.remove(&game_id);

                            if let Some(opponent_peer_id) = self.sessions_peers.get(&opponent_session_key) {
                                commands.push(Message(*opponent_peer_id, ServerMessage::OpponentLeft))
                            }
                        } else {
                            trace!("in the game {:0>16X} - notifying opponent {:0>16X}", game_id, &opponent_session_key);

                            if let Some(opponent_peer_id) = self.sessions_peers.get(&opponent_session_key) {
                                commands.push(Message(*opponent_peer_id, ServerMessage::OpponentOffline))
                            }
                        }
                    },
                }

                self.sessions_peers.remove(&session_key);
                self.peers_sessions.remove(&peer_id);
            },
        }

        commands
    }

    /// Do clean up of inactive sessions.
    pub fn handle_cleanup(&mut self, peer_id: &usize) -> Vec<Command> {
        // TODO: implement cleanup
        let commands = Vec::new();

        commands
    }

        /// Get a unique id for a session.
    fn unique_session_key(&self) -> u64 {
        loop {
            let key = rand::thread_rng().gen();
            if !self.sessions.contains_key(&key) {
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