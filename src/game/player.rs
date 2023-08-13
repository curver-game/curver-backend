use core::fmt;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::constants::{DELTA_POS_PER_TICK, MAP_HEIGHT, MAP_WIDTH};

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct Player {
    pub id: PlayerUuid,
    pub x: f32,
    pub y: f32,
    #[serde(rename = "angleUnitVectorX")]
    pub angle_unit_vector_x: f32,
    #[serde(rename = "angleUnitVectorY")]
    pub angle_unit_vector_y: f32,
    #[serde(rename = "isReady")]
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
            id: PlayerUuid(id),
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

#[derive(Debug, PartialEq, Clone, Hash, Eq, Copy)]
pub struct PlayerUuid(pub Uuid);

impl PlayerUuid {
    pub fn new() -> PlayerUuid {
        PlayerUuid(Uuid::new_v4())
    }

    pub fn get_uuid(&self) -> Uuid {
        self.0
    }
}

impl Serialize for PlayerUuid {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        serializer.serialize_str(&self.0.to_string())
    }
}

impl<'de> Deserialize<'de> for PlayerUuid {
    fn deserialize<D>(deserializer: D) -> Result<PlayerUuid, D::Error>
    where
        D: serde::de::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Ok(PlayerUuid(Uuid::parse_str(&s).unwrap()))
    }
}

impl fmt::Display for PlayerUuid {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0.to_string())
    }
}
