use std::collections::HashMap;

use actix::Message;
use serde::{Deserialize, Serialize};

use crate::{
    curver_ws_actor::CurverAddress,
    game::{
        path::Path,
        player::{Player, PlayerUuid},
        GameOutcome, GameState,
    },
    room::RoomUuid,
};

pub struct ForwardedMessage {
    pub message: CurverMessageToReceive,
    pub user_id: PlayerUuid,
    pub address: CurverAddress,
}

#[derive(Debug, Message, Serialize, Deserialize, Clone)]
#[rtype(result = "()")]
#[serde(tag = "type")]
pub enum CurverMessageToSend {
    #[serde(rename = "joinRoomError")]
    JoinRoomError { reason: String },
    #[serde(rename = "joinedRoom")]
    JoinedRoom {
        #[serde(rename = "roomId")]
        room_id: RoomUuid,
        #[serde(rename = "userId")]
        user_id: PlayerUuid,
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
    SyncPaths { paths: HashMap<PlayerUuid, Path> },
    #[serde(rename = "gameEnded")]
    GameEnded {
        outcome: GameOutcome,
        #[serde(rename = "scoreBoard")]
        score_board: HashMap<PlayerUuid, u32>,
    },
    #[serde(rename = "userEliminated")]
    UserEliminated {
        #[serde(rename = "userId")]
        user_id: PlayerUuid,
    },
    #[serde(rename = "faultyMessage")]
    FaultyMessage { message: String },
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
        room_id: RoomUuid,
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
