pub mod path;
pub mod player;

use std::{collections::HashMap, sync::Arc};

use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    curver_ws_actor::CurverAddress,
    message::{CurverMessageToSend, UuidSerde},
};

use self::{
    path::{Node, Path},
    player::Player,
};

pub type Clients = HashMap<Uuid, CurverAddress>;
pub type Players = HashMap<Uuid, Player>;

pub struct Game {
    pub paths: HashMap<Uuid, Path>,
    pub state: Arc<RwLock<GameState>>,

    pub clients: Arc<RwLock<Clients>>,
    pub players: Arc<RwLock<Players>>,
    pub game_state: Arc<RwLock<GameState>>,
}

impl Game {
    pub fn new(
        game_state: Arc<RwLock<GameState>>,
        clients: Arc<RwLock<Clients>>,
        players: Arc<RwLock<Players>>,
    ) -> Game {
        Game {
            state: game_state,
            paths: HashMap::new(),
            clients,
            players,
            game_state,
        }
    }

    /// Returns the winner
    pub fn tick(&mut self) -> Option<Uuid> {
        let mut players_to_eliminate: Vec<Uuid> = Vec::new();

        for player in self.players.write().values_mut() {
            player.calculate_new_position();

            if player.check_if_out_of_bounds() {
                players_to_eliminate.push(player.id.get_uuid());
                continue;
            }

            for path in self.paths.values() {
                if path.check_collision(player) {
                    players_to_eliminate.push(player.id.get_uuid());
                    break;
                }
            }

            Game::add_players_location_to_path(
                &mut self.paths,
                player.id.get_uuid(),
                Node(player.x, player.y),
            );
        }

        players_to_eliminate.iter().for_each(|player_id| {
            self.eliminate_player_and_notify_all(*player_id);
        });

        self.send_update_to_all();

        if self.players.read().len() == 1 {
            let winner = self.players.read().keys().next().cloned()?;

            self.send_message_to_all(CurverMessageToSend::UserWon {
                user_id: UuidSerde(winner),
            });

            *self.game_state.write() = GameState::Waiting;
            return Some(winner);
        }

        None
    }

    // --- Player Handling ---
    pub fn rotate_player(
        &mut self,
        user_id: Uuid,
        angle_unit_vector_x: f32,
        angle_unit_vector_y: f32,
    ) {
        if let Some(player) = self.players.write().get_mut(&user_id) {
            player.angle_unit_vector_x = angle_unit_vector_x;
            player.angle_unit_vector_y = angle_unit_vector_y;
        };
    }

    fn eliminate_player_and_notify_all(&mut self, player_id: Uuid) {
        self.remove_player(player_id);

        self.send_message_to_all(CurverMessageToSend::UserEliminated {
            user_id: UuidSerde(player_id),
        });
    }

    fn remove_player(&mut self, player_id: Uuid) {
        self.players.write().remove(&player_id);
    }

    fn add_players_location_to_path(paths: &mut HashMap<Uuid, Path>, player_id: Uuid, node: Node) {
        if let Some(path) = paths.get_mut(&player_id) {
            path.push(node);
        } else {
            let path = Path { nodes: vec![node] };

            paths.insert(player_id, path);
        }
    }

    // --- Message Sending ---
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

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub enum GameState {
    #[serde(rename = "waiting")]
    Waiting,
    #[serde(rename = "countdown")]
    Countdown,
    #[serde(rename = "started")]
    Started,
}
