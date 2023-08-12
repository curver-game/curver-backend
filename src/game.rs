use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::message::{Player, UuidSerde};

const TICK_RATE: f32 = 1.0;
const DELTA_POS_PER_SECOND: f32 = 1.0;

pub const MS_PER_TICK: f32 = 1000.0 / TICK_RATE;
const DELTA_POS_PER_TICK: f32 = DELTA_POS_PER_SECOND / TICK_RATE;

const MAP_WIDTH: f32 = 100.0;
const MAP_HEIGHT: f32 = 100.0;

#[derive(Debug)]
pub struct Game {
    pub players: HashMap<Uuid, Player>,
    pub paths: HashMap<Uuid, Path>,
    pub state: GameState,
}

impl Game {
    pub fn new() -> Game {
        Game {
            players: HashMap::new(),
            state: GameState::Waiting,
            paths: HashMap::new(),
        }
    }

    pub fn add_player(&mut self, player_id: Uuid) {
        let random_x = rand::random::<f32>() * MAP_WIDTH;
        let random_y = rand::random::<f32>() * MAP_HEIGHT;
        let random_angle = rand::random::<f32>() * 2.0 * std::f32::consts::PI;
        let random_angle_unit_vector_x = random_angle.cos();
        let random_angle_unit_vector_y = random_angle.sin();
        let player = Player::new(
            player_id,
            random_x,
            random_y,
            random_angle_unit_vector_x,
            random_angle_unit_vector_y,
        );
        let path = Path::new();

        self.players.insert(player_id, player);
        self.paths.insert(player_id, path);
    }

    pub fn remove_player(&mut self, player_id: Uuid) {
        self.players.remove(&player_id);

        // Send message to all players that player has eliminated
    }

    fn check_is_player_out_of_bounds(player: &Player) -> bool {
        player.x < 0.0 || player.x > MAP_WIDTH || player.y < 0.0 || player.y > MAP_HEIGHT
    }

    /// Returns the winner
    pub fn tick(&mut self) -> Option<Uuid> {
        let mut players_to_remove: Vec<Uuid> = Vec::new();

        for player in self.players.values_mut() {
            player.calculate_new_position();
            if Self::check_is_player_out_of_bounds(player) {
                println!("Player out of bounds: {:?}", player);
                players_to_remove.push(player.id.get_uuid());
                continue;
            }

            for path in self.paths.values() {
                if path.check_collision(player) {
                    println!("Player collided with path: {:?}", player);
                    players_to_remove.push(player.id.get_uuid());
                    break;
                }
            }

            if let Some(path) = self.paths.get_mut(&player.id.get_uuid()) {
                path.push(Node(player.x, player.y));
            }
        }

        println!("Tick: {:?}", self.players);

        for player_id in players_to_remove {
            self.remove_player(player_id);
        }

        if self.players.len() == 1 {
            return Some(self.players.keys().next().cloned()?);
        }

        None
    }
}

impl Player {
    pub fn new(
        id: Uuid,
        x: f32,
        y: f32,
        angle_unit_vector_x: f32,
        angle_unit_vector_y: f32,
    ) -> Player {
        Player {
            id: UuidSerde(id),
            x,
            y,
            angle_unit_vector_x,
            angle_unit_vector_y,
        }
    }

    pub fn calculate_new_position(&mut self) {
        self.x += self.angle_unit_vector_x * DELTA_POS_PER_TICK;
        self.y += self.angle_unit_vector_y * DELTA_POS_PER_TICK;
    }
}

#[derive(Debug, Clone)]
pub struct Path {
    nodes: Vec<Node>,
}

impl Path {
    pub fn new() -> Path {
        Path { nodes: Vec::new() }
    }

    pub fn push(&mut self, node: Node) {
        self.nodes.push(node);
    }

    pub fn check_collision(&self, player: &Player) -> bool {
        if self.nodes.len() < 2 {
            return false;
        }

        let player_nodes = (
            Node(
                player.x - player.angle_unit_vector_x * DELTA_POS_PER_TICK,
                player.y - player.angle_unit_vector_y * DELTA_POS_PER_TICK,
            ),
            Node(player.x, player.y),
        );

        for i in 0..self.nodes.len() - 1 {
            let path_nodes = (self.nodes[i].clone(), self.nodes[i + 1].clone());

            if Self::check_if_line_segments_intersect(&path_nodes, &player_nodes) {
                return true;
            }
        }

        false
    }

    pub fn check_if_line_segments_intersect(
        path_nodes: &(Node, Node),
        player_nodes: &(Node, Node),
    ) -> bool {
        // Treat path_nodes as a line segment and check if they intersect with player_nodes line
        // segment
        let x1 = path_nodes.0 .0;
        let y1 = path_nodes.0 .1;
        let x2 = path_nodes.1 .0;
        let y2 = path_nodes.1 .1;

        let x3 = player_nodes.0 .0;
        let y3 = player_nodes.0 .1;
        let x4 = player_nodes.1 .0;
        let y4 = player_nodes.1 .1;

        let denominator = (x1 - x2) * (y3 - y4) - (y1 - y2) * (x3 - x4);

        // If denominator is 0, lines are parallel
        if denominator == 0.0 {
            return false;
        }

        let t = ((x1 - x3) * (y3 - y4) - (y1 - y3) * (x3 - x4)) / denominator;
        let u = -((x1 - x2) * (y1 - y3) - (y1 - y2) * (x1 - x3)) / denominator;

        if t > 0.0 && t < 1.0 && u > 0.0 && u < 1.0 {
            return true;
        }

        false
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct Node(f32, f32);

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub enum GameState {
    #[serde(rename = "waiting")]
    Waiting,
    #[serde(rename = "started")]
    Started,
}
