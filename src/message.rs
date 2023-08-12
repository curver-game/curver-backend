use actix::Message;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{curver_ws_actor::CurverAddress, game::GameState};

#[derive(Debug, PartialEq, Clone)]
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

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct Player {
    pub id: UuidSerde,
    pub x: f32,
    pub y: f32,
    pub angle_unit_vector_x: f32,
    pub angle_unit_vector_y: f32,
}

#[derive(Debug, Message, Serialize, Deserialize, PartialEq, Clone)]
#[rtype(result = "()")]
#[serde(tag = "type")]
pub enum CurverMessageToSend {
    #[serde(rename = "created-room")]
    CreatedRoom { room_id: UuidSerde },
    #[serde(rename = "joined-room-error")]
    JoinedRoomError { reason: String },
    #[serde(rename = "joined-room")]
    JoinedRoom { room_id: UuidSerde },
    #[serde(rename = "left-room")]
    LeftRoom,
    #[serde(rename = "leave-room-error")]
    LeaveRoomError { reason: String },
    #[serde(rename = "update")]
    Update { client_state: Vec<Player> },
    #[serde(rename = "game-state")]
    GameState { current_state: GameState },
    #[serde(rename = "user-won")]
    UserWon { user_id: UuidSerde },
    #[serde(rename = "user-eliminated")]
    UserEliminated { user_id: UuidSerde },
}

#[derive(Debug, Message, Serialize, Deserialize, PartialEq)]
#[rtype(result = "()")]
#[serde(tag = "type")]
pub enum CurverMessageToReceive {
    #[serde(rename = "create-room")]
    CreateRoom,
    #[serde(rename = "join-room")]
    JoinRoom { room_id: UuidSerde },
    #[serde(rename = "leave-room")]
    LeaveRoom,
    #[serde(rename = "rotate")]
    Rotate {
        angle_unit_vector_x: f32,
        angle_unit_vector_y: f32,
    },
}

pub struct ForwardedMessage {
    pub message: CurverMessageToReceive,
    pub user_id: Uuid,
    pub address: CurverAddress,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serialize_deserialize_uuid() {
        let uuid = Uuid::new_v4();
        let uuid_serializable = UuidSerde(uuid);
        let serialized = serde_json::to_string(&uuid_serializable).unwrap();
        let deserialized: UuidSerde = serde_json::from_str(&serialized).unwrap();
        assert_eq!(uuid, deserialized.get_uuid());
    }

    #[test]
    fn test_serialize_deserialize_client_state() {
        let uuid = Uuid::new_v4();
        let client_state = Player {
            id: UuidSerde(uuid),
            x: 1.0,
            y: 2.0,
            angle_unit_vector_x: 3.0,
            angle_unit_vector_y: 4.0,
        };
        let serialized = serde_json::to_string(&client_state).unwrap();
        let deserialized: Player = serde_json::from_str(&serialized).unwrap();
        assert_eq!(client_state, deserialized);
    }

    #[test]
    fn test_serialize_deserialize_game_state() {
        let game_state = GameState::Waiting;
        let serialized = serde_json::to_string(&game_state).unwrap();
        let deserialized: GameState = serde_json::from_str(&serialized).unwrap();
        assert_eq!(game_state, deserialized);
    }

    #[test]
    fn test_serialize_deserialize_websocket_message() {
        let uuid = Uuid::new_v4();
        let client_state = Player {
            id: UuidSerde(uuid),
            x: 1.0,
            y: 2.0,
            angle_unit_vector_x: 3.0,
            angle_unit_vector_y: 4.0,
        };
        let websocket_message = CurverMessageToSend::Update {
            client_state: vec![client_state],
        };
        let serialized = serde_json::to_string(&websocket_message).unwrap();
        let deserialized: CurverMessageToSend = serde_json::from_str(&serialized).unwrap();
        assert_eq!(websocket_message, deserialized);
    }

    #[test]
    fn test_serialize_deserialize_websocket_message_rotate() {
        let websocket_message = CurverMessageToReceive::Rotate {
            angle_unit_vector_x: 1.0,
            angle_unit_vector_y: 2.0,
        };
        let serialized = serde_json::to_string(&websocket_message).unwrap();
        let deserialized: CurverMessageToReceive = serde_json::from_str(&serialized).unwrap();
        assert_eq!(websocket_message, deserialized);
    }
}
