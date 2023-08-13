use core::fmt;

use uuid::Uuid;

#[derive(Debug, Clone)]
pub enum ServerError {
    RoomDoesNotExist(Uuid),
}

impl fmt::Display for ServerError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ServerError::RoomDoesNotExist(room_id) => write!(f, "Room {} does not exist", room_id),
        }
    }
}
