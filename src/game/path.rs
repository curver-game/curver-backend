use serde::{Deserialize, Serialize};

use crate::constants::DELTA_POS_PER_TICK;

use super::player::Player;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Path {
    pub nodes: Vec<Node>,
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

    fn check_if_line_segments_intersect(
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
pub struct Node(pub f32, pub f32);
