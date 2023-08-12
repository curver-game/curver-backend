use std::{collections::HashMap, sync::Arc};

use parking_lot::RwLock;
use tokio::sync::mpsc::{self, Receiver, Sender};
use uuid::Uuid;

use crate::{
    curver_ws_actor::CurverAddress,
    message::{
        ClientState, CurverMessageToReceive, CurverMessageToSend, ForwardedMessage, UuidSerde,
    },
};

pub struct RoomHandler {
    pub room_message_transmitters: Arc<RwLock<HashMap<Uuid, Sender<ForwardedMessage>>>>,
    pub room_map: HashMap<Uuid, Uuid>,
}

impl RoomHandler {
    pub fn new() -> Self {
        Self {
            room_message_transmitters: Arc::new(RwLock::new(HashMap::new())),
            room_map: HashMap::new(),
        }
    }

    pub fn create_room(&mut self) -> Uuid {
        let id = Uuid::new_v4();
        let (room_message_transmitter, room_message_receiver) = mpsc::channel(100);
        let room = Room::new(id, room_message_receiver);
        let room_message_transmitters_clone = self.room_message_transmitters.clone();

        tokio::spawn(async move {
            room.message_handler().await;
            let mut transmitter_lock = room_message_transmitters_clone.write();
            transmitter_lock.remove(&id);
        });

        self.add_transmitter(id, room_message_transmitter);

        id
    }

    pub fn join_room(&mut self, room_id: Uuid, user_id: Uuid, address: CurverAddress) {
        self.send_message_to_room(
            room_id,
            ForwardedMessage {
                user_id,
                address,
                message: CurverMessageToReceive::JoinRoom {
                    room_id: UuidSerde(room_id),
                },
            },
        );

        self.room_map.insert(user_id, room_id);
    }

    pub fn leave_room(&mut self, user_id: Uuid, address: CurverAddress) {
        if let Some(room_id) = self.room_map.get(&user_id) {
            self.send_message_to_room(
                *room_id,
                ForwardedMessage {
                    user_id,
                    address,
                    message: CurverMessageToReceive::LeaveRoom,
                },
            );

            self.room_map.remove(&user_id);
        }
    }

    pub fn send_message_to_room(&self, room_id: Uuid, message: ForwardedMessage) {
        let transmitter_lock = self.room_message_transmitters.read();

        if let Some(transmitter) = transmitter_lock.get(&room_id) {
            transmitter.try_send(message).unwrap();
        }
    }

    fn add_transmitter(&mut self, room_id: Uuid, transmitter: Sender<ForwardedMessage>) {
        let mut transmitter_lock = self.room_message_transmitters.write();
        transmitter_lock.insert(room_id, transmitter);
    }
}

pub struct Room {
    pub clients: Arc<RwLock<HashMap<Uuid, CurverAddress>>>,
    pub id: Uuid,
    pub receiver: Receiver<ForwardedMessage>,
}

const TICK_RATE: f32 = 10.0;
const MS_PER_TICK: f32 = 1000.0 / TICK_RATE;
const SPEED_PER_MS: f32 = 1.0;

impl Room {
    pub fn new(id: Uuid, receiver: Receiver<ForwardedMessage>) -> Self {
        Self {
            clients: Arc::new(RwLock::new(HashMap::new())),
            id,
            receiver,
        }
    }

    async fn message_handler(mut self) {
        let clients_clone = self.clients.clone();

        tokio::spawn(async move {
            let mut client_states = vec![ClientState {
                id: UuidSerde(Uuid::new_v4()),
                x: 0.0,
                y: 0.0,
                angle_unit_vector_x: 1.0,
                angle_unit_vector_y: 0.0,
            }];

            loop {
                for client_state in client_states.iter_mut() {
                    client_state.x += client_state.angle_unit_vector_x * SPEED_PER_MS / MS_PER_TICK;
                    client_state.y += client_state.angle_unit_vector_y * SPEED_PER_MS / MS_PER_TICK;
                }

                for (_, address) in clients_clone.read().iter() {
                    address.try_send(CurverMessageToSend::Update {
                        client_state: client_states.clone(),
                    });
                }

                tokio::time::sleep(tokio::time::Duration::from_millis(MS_PER_TICK as u64)).await;
            }
        });

        loop {
            if let Some(forwarded_message) = self.receiver.recv().await {
                match forwarded_message.message {
                    CurverMessageToReceive::JoinRoom { .. } => {
                        let mut clients_lock = self.clients.write();
                        clients_lock.insert(forwarded_message.user_id, forwarded_message.address);
                    }

                    CurverMessageToReceive::LeaveRoom => {
                        let mut clients_lock = self.clients.write();
                        clients_lock.remove(&forwarded_message.user_id);

                        if clients_lock.is_empty() {
                            break;
                        }
                    }

                    CurverMessageToReceive::Rotate { .. } => {}

                    _ => {}
                }
            }
        }
    }
}
