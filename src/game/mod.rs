pub mod path;
pub mod player;

use std::{collections::HashMap, sync::Arc};

use parking_lot::RwLock;
use serde::{Deserialize, Serialize};

use crate::{
    constants::TICK_COUNT_TO_SYNC, curver_ws_actor::CurverAddress, message::CurverMessageToSend,
};

use self::{
    path::{Node, Path},
    player::{Player, PlayerUuid},
};

pub type Clients = HashMap<PlayerUuid, CurverAddress>;
pub type Players = HashMap<PlayerUuid, Player>;

pub struct Game {
    pub paths: HashMap<PlayerUuid, Path>,
    pub state: Arc<RwLock<GameState>>,

    pub clients: Arc<RwLock<Clients>>,
    pub players: Arc<RwLock<Players>>,

    tick_count: u32,
}

impl Game {
    pub fn new(
        state: Arc<RwLock<GameState>>,
        clients: Arc<RwLock<Clients>>,
        players: Arc<RwLock<Players>>,
    ) -> Game {
        Game {
            state,
            paths: HashMap::new(),
            clients,
            players,
            tick_count: 0,
        }
    }

    /// Returns the winner
    pub fn tick(&mut self) -> Option<GameOutcome> {
        let mut players_to_eliminate: Vec<PlayerUuid> = Vec::new();

        for player in self.players.write().values_mut() {
            player.calculate_new_position();

            if player.check_if_out_of_bounds() {
                players_to_eliminate.push(player.id);
                continue;
            }

            for path in self.paths.values() {
                if path.check_collision(player) {
                    players_to_eliminate.push(player.id);
                    break;
                }
            }

            Game::add_players_location_to_path(
                &mut self.paths,
                player.id,
                Node(player.x, player.y),
            );
        }

        players_to_eliminate.iter().for_each(|player_id| {
            self.eliminate_player_and_notify_all(*player_id);
        });

        self.send_update_to_all();

        let remaining_player_count = self.players.read().len();

        let outcome = match remaining_player_count {
            0 => Some(GameOutcome::Tie),
            1 => {
                let winner = self.players.read().keys().next().cloned()?;
                Some(GameOutcome::Winner { user_id: winner })
            }
            _ => return None,
        };

        match outcome.clone() {
            Some(outcome) => {
                self.send_message_to_all(CurverMessageToSend::GameEnded { outcome });

                self.reset_all_players();
                *self.state.write() = GameState::Waiting;
                self.send_update_to_all();
            }
            None => (),
        }

        if self.tick_count % TICK_COUNT_TO_SYNC == 0 {
            self.send_sync_to_all();
        }

        self.tick_count += 1;

        outcome
    }

    // --- Player Handling ---
    pub fn rotate_player(
        &mut self,
        user_id: PlayerUuid,
        angle_unit_vector_x: f32,
        angle_unit_vector_y: f32,
    ) {
        if let Some(player) = self.players.write().get_mut(&user_id) {
            player.angle_unit_vector_x = angle_unit_vector_x;
            player.angle_unit_vector_y = angle_unit_vector_y;
        };
    }

    fn eliminate_player_and_notify_all(&mut self, player_id: PlayerUuid) {
        self.remove_player(player_id);

        self.send_message_to_all(CurverMessageToSend::UserEliminated { user_id: player_id });
    }

    fn remove_player(&mut self, player_id: PlayerUuid) {
        self.players.write().remove(&player_id);
    }

    fn add_players_location_to_path(
        paths: &mut HashMap<PlayerUuid, Path>,
        player_id: PlayerUuid,
        node: Node,
    ) {
        if let Some(path) = paths.get_mut(&player_id) {
            path.push(node);
        } else {
            let path = Path { nodes: vec![node] };

            paths.insert(player_id, path);
        }
    }

    fn reset_all_players(&mut self) {
        for player in self.players.write().values_mut() {
            player.x = 0.0;
            player.y = 0.0;
            player.angle_unit_vector_x = 0.0;
            player.angle_unit_vector_y = 0.0;
            player.is_ready = false;
        }
    }

    // --- Message Sending ---
    fn send_sync_to_all(&self) {
        let sync = CurverMessageToSend::SyncPaths {
            paths: self.paths.clone(),
        };

        self.send_message_to_all(sync);
    }

    fn send_update_to_all(&self) {
        let update = CurverMessageToSend::Update {
            players: self.players.read().values().cloned().collect(),
            game_state: self.state.read().clone(),
        };

        self.send_message_to_all(update);
    }

    fn send_message_to_all(&self, message: CurverMessageToSend) {
        for client in self.clients.read().values() {
            client.do_send(message.clone());
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type")]
pub enum GameOutcome {
    #[serde(rename = "winner")]
    Winner {
        #[serde(rename = "userId")]
        user_id: PlayerUuid,
    },
    #[serde(rename = "tie")]
    Tie,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub enum GameState {
    #[serde(rename = "waiting")]
    Waiting,
    #[serde(rename = "countdown")]
    Countdown,
    #[serde(rename = "started")]
    Started,
}
