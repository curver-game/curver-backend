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
    pub state: GameState,

    pub clients: Arc<RwLock<Clients>>,
    pub players: Arc<RwLock<Players>>,
}

impl Game {
    pub fn new(clients: Arc<RwLock<Clients>>, players: Arc<RwLock<Players>>) -> Game {
        Game {
            state: GameState::Waiting,
            paths: HashMap::new(),
            clients,
            players,
        }
    }

    /// Returns the winner
    pub fn tick(&mut self) -> Option<Uuid> {
        let mut players_to_eliminate: Vec<Uuid> = Vec::new();

        let mut players_clone = self.players.read().clone();
        for player in players_clone.values_mut() {
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

            if let Some(path) = self.paths.get_mut(&player.id.get_uuid()) {
                path.push(Node(player.x, player.y));
            } else {
                let path = Path {
                    nodes: vec![Node(player.x, player.y)],
                };

                self.paths.insert(player.id.get_uuid(), path);
            }
        }

        let mut players_lock = self.players.write();
        players_lock.extend(players_clone);
        drop(players_lock);

        for player_id in players_to_eliminate {
            self.eliminate_player_and_notify_all(player_id);
        }

        if self.players.read().len() == 1 {
            return Some(self.players.read().keys().next().cloned()?);
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

    // --- Message Sending ---
    fn send_message_to_all(&self, message: CurverMessageToSend) {
        let clients_lock = self.clients.read();
        for client in clients_lock.values() {
            client.do_send(message.clone());
        }
    }

    fn send_message_to_user(&self, user_id: Uuid, message: CurverMessageToSend) {
        let clients_lock = self.clients.read();
        if let Some(client) = clients_lock.get(&user_id) {
            client.do_send(message);
        }
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub enum GameState {
    #[serde(rename = "waiting")]
    Waiting,
    #[serde(rename = "started")]
    Started,
}
