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
    #[serde(rename = "joinRoomError")]
    JoinRoomError { reason: String },
    #[serde(rename = "joinedRoom")]
    JoinedRoom {
        #[serde(rename = "roomId")]
        room_id: UuidSerde,
        #[serde(rename = "userId")]
        user_id: UuidSerde,
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

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::game::{
        path::{Node, Path},
        player::Player,
    };

    use super::{CurverMessageToReceive, CurverMessageToSend, UuidSerde};

    #[test]
    fn write_all_possible_jsons() {
        let players = vec![
            Player {
                id: UuidSerde(uuid::Uuid::new_v4()),
                x: 0.0,
                y: 1.1,
                is_ready: false,
                angle_unit_vector_x: 1.0,
                angle_unit_vector_y: 0.0,
            },
            Player {
                id: UuidSerde(uuid::Uuid::new_v4()),
                x: 0.0,
                y: 1.1,
                is_ready: false,
                angle_unit_vector_x: 1.0,
                angle_unit_vector_y: 0.0,
            },
        ];

        let mut paths = HashMap::new();
        paths.insert(
            super::UuidSerde(uuid::Uuid::new_v4()),
            Path {
                nodes: vec![Node(0.0, 1.1), Node(2.2, 3.3)],
            },
        );

        let all_messages_to_send = vec![
            CurverMessageToSend::JoinRoomError {
                reason: "reason".to_string(),
            },
            CurverMessageToSend::JoinedRoom {
                room_id: super::UuidSerde(uuid::Uuid::new_v4()),
                user_id: super::UuidSerde(uuid::Uuid::new_v4()),
            },
            CurverMessageToSend::LeftRoom,
            CurverMessageToSend::LeaveRoomError {
                reason: "reason".to_string(),
            },
            CurverMessageToSend::Update {
                players: players.clone(),
                game_state: crate::game::GameState::Waiting,
            },
            CurverMessageToSend::Update {
                players: players.clone(),
                game_state: crate::game::GameState::Countdown,
            },
            CurverMessageToSend::Update {
                players: players.clone(),
                game_state: crate::game::GameState::Started,
            },
            CurverMessageToSend::SyncPaths { paths: paths },
            CurverMessageToSend::GameEnded {
                outcome: crate::game::GameOutcome::Tie,
            },
            CurverMessageToSend::GameEnded {
                outcome: crate::game::GameOutcome::Winner {
                    user_id: super::UuidSerde(uuid::Uuid::new_v4()),
                },
            },
            CurverMessageToSend::UserEliminated {
                user_id: super::UuidSerde(uuid::Uuid::new_v4()),
            },
        ];

        let all_messages_to_receive = vec![
            super::CurverMessageToReceive::CreateRoom,
            super::CurverMessageToReceive::JoinRoom {
                room_id: super::UuidSerde(uuid::Uuid::new_v4()),
            },
            super::CurverMessageToReceive::LeaveRoom,
            super::CurverMessageToReceive::Rotate {
                angle_unit_vector_x: 1.0,
                angle_unit_vector_y: 0.0,
            },
            super::CurverMessageToReceive::IsReady { is_ready: true },
            super::CurverMessageToReceive::IsReady { is_ready: false },
        ];

        println!("Messages sent by server:");
        for message in all_messages_to_send {
            write_send_message_as_json(message);
        }

        println!("\nMessages received by server:");
        for message in all_messages_to_receive {
            write_receive_message_as_json(message);
        }
    }

    fn write_send_message_as_json(message: CurverMessageToSend) {
        let json = serde_json::to_string(&message).unwrap();
        println!("{}", json);
    }

    fn write_receive_message_as_json(message: CurverMessageToReceive) {
        let json = serde_json::to_string(&message).unwrap();
        println!("{}", json);
    }
}
