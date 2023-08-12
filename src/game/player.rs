use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    constants::{DELTA_POS_PER_TICK, MAP_HEIGHT, MAP_WIDTH},
    message::UuidSerde,
};

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct Player {
    pub id: UuidSerde,
    pub x: f32,
    pub y: f32,
    pub angle_unit_vector_x: f32,
    pub angle_unit_vector_y: f32,
    pub is_ready: bool,
}

impl Player {
    pub fn new(
        id: Uuid,
        x: f32,
        y: f32,
        angle_unit_vector_x: f32,
        angle_unit_vector_y: f32,
        is_ready: bool,
    ) -> Player {
        Player {
            id: UuidSerde(id),
            x,
            y,
            angle_unit_vector_x,
            angle_unit_vector_y,
            is_ready,
        }
    }

    pub fn calculate_new_position(&mut self) {
        self.x += self.angle_unit_vector_x * DELTA_POS_PER_TICK;
        self.y += self.angle_unit_vector_y * DELTA_POS_PER_TICK;
    }

    pub fn check_if_out_of_bounds(&self) -> bool {
        self.x < 0.0 || self.x > MAP_WIDTH || self.y < 0.0 || self.y > MAP_HEIGHT
    }
}
