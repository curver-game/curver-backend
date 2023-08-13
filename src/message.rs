use std::collections::HashMap;

use actix::Message;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    curver_ws_actor::CurverAddress,
    game::{path::Path, player::Player, GameOutcome, GameState},
};

pub struct ForwardedMessage {
    pub message: CurverMessageToReceive,
    pub user_id: Uuid,
    pub address: CurverAddress,
}

#[derive(Debug, Message, Serialize, Deserialize, Clone)]
#[rtype(result = "()")]
#[serde(tag = "type")]
pub enum CurverMessageToSend {
    #[serde(rename = "createdRoom")]
    CreatedRoom {
        #[serde(rename = "roomId")]
        room_id: UuidSerde,
    },
    #[serde(rename = "joinRoomError")]
    JoinRoomError { reason: String },
    #[serde(rename = "joinedRoom")]
    JoinedRoom {
        #[serde(rename = "roomId")]
        room_id: UuidSerde,
    },
    #[serde(rename = "leftRoom")]
    LeftRoom,
    #[serde(rename = "leaveRoomError")]
    LeaveRoomError { reason: String },
    #[serde(rename = "update")]
    Update {
        players: Vec<Player>,
        #[serde(rename = "gameState")]
        game_state: GameState,
    },
    #[serde(rename = "syncPaths")]
    SyncPaths { paths: HashMap<UuidSerde, Path> },
    #[serde(rename = "gameEnded")]
    GameEnded { outcome: GameOutcome },
    #[serde(rename = "userEliminated")]
    UserEliminated {
        #[serde(rename = "userId")]
        user_id: UuidSerde,
    },
}

#[derive(Debug, Message, Serialize, Deserialize, PartialEq)]
#[rtype(result = "()")]
#[serde(tag = "type")]
pub enum CurverMessageToReceive {
    #[serde(rename = "createRoom")]
    CreateRoom,
    #[serde(rename = "joinRoom")]
    JoinRoom {
        #[serde(rename = "roomId")]
        room_id: UuidSerde,
    },
    #[serde(rename = "leaveRoom")]
    LeaveRoom,
    #[serde(rename = "rotate")]
    Rotate {
        #[serde(rename = "angleUnitVectorX")]
        angle_unit_vector_x: f32,
        #[serde(rename = "angleUnitVectorY")]
        angle_unit_vector_y: f32,
    },
    #[serde(rename = "isReady")]
    IsReady {
        #[serde(rename = "isReady")]
        is_ready: bool,
    },
}

#[derive(Debug, PartialEq, Clone, Hash, Eq)]
pub struct UuidSerde(pub Uuid);

impl UuidSerde {
    pub fn get_uuid(&self) -> Uuid {
        self.0
    }
}

impl Serialize for UuidSerde {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        serializer.serialize_str(&self.0.to_string())
    }
}

impl<'de> Deserialize<'de> for UuidSerde {
    fn deserialize<D>(deserializer: D) -> Result<UuidSerde, D::Error>
    where
        D: serde::de::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Ok(UuidSerde(Uuid::parse_str(&s).unwrap()))
    }
}
