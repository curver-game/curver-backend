use std::{collections::HashMap, sync::Arc};

use parking_lot::RwLock;
use tokio::sync::mpsc::{self, Receiver, Sender};
use uuid::Uuid;

use crate::{
    curver_ws_actor::CurverAddress,
    message::{CurverMessageToReceive, CurverMessageToSend, ForwardedMessage, UuidSerde},
    room::Room,
};

pub struct ServerHandler {
    room_message_transmitters: Arc<RwLock<HashMap<Uuid, Sender<ForwardedMessage>>>>,
    room_map: HashMap<Uuid, Uuid>,
    internal_message_receiver: Receiver<ForwardedMessage>,
}

impl ServerHandler {
    pub fn new(internal_message_receiver: Receiver<ForwardedMessage>) -> Self {
        Self {
            room_message_transmitters: Arc::new(RwLock::new(HashMap::new())),
            room_map: HashMap::new(),
            internal_message_receiver,
        }
    }

    /// This thread will always be running.
    pub async fn message_handler(mut self) {
        loop {
            if let Some(forwarded_message) = self.internal_message_receiver.recv().await {
                match forwarded_message.message {
                    CurverMessageToReceive::CreateRoom => {
                        let room_id = self.create_room();

                        forwarded_message
                            .address
                            .do_send(CurverMessageToSend::CreatedRoom {
                                room_id: UuidSerde(room_id),
                            });
                    }

                    CurverMessageToReceive::JoinRoom { room_id } => {
                        self.join_room_and_forward_message(
                            room_id.get_uuid(),
                            forwarded_message.user_id,
                            forwarded_message.address.clone(),
                        );

                        forwarded_message
                            .address
                            .do_send(CurverMessageToSend::JoinedRoom {
                                room_id: UuidSerde(room_id.get_uuid()),
                            });
                    }

                    CurverMessageToReceive::LeaveRoom => {
                        self.leave_room_and_forward_message(
                            forwarded_message.user_id,
                            forwarded_message.address.clone(),
                        );

                        forwarded_message
                            .address
                            .do_send(CurverMessageToSend::LeftRoom);
                    }

                    CurverMessageToReceive::Rotate {
                        angle_unit_vector_x,
                        angle_unit_vector_y,
                    } => {
                        self.send_message_to_room(
                            forwarded_message.user_id,
                            ForwardedMessage {
                                user_id: forwarded_message.user_id,
                                address: forwarded_message.address.clone(),
                                message: CurverMessageToReceive::Rotate {
                                    angle_unit_vector_x,
                                    angle_unit_vector_y,
                                },
                            },
                        );
                    }

                    CurverMessageToReceive::IsReady { is_ready } => {
                        self.send_message_to_room_by_user_id(
                            forwarded_message.user_id,
                            ForwardedMessage {
                                user_id: forwarded_message.user_id,
                                address: forwarded_message.address.clone(),
                                message: CurverMessageToReceive::IsReady { is_ready },
                            },
                        );
                    }
                }
            }
        }
    }

    // --- Room Handling ---
    fn create_room(&mut self) -> Uuid {
        let room_id = Uuid::new_v4();
        let (room_message_transmitter, room_message_receiver) = mpsc::channel(100);
        let room_message_transmitters_clone = self.room_message_transmitters.clone();

        let room = Room::new(room_message_receiver);

        tokio::spawn(async move {
            room.message_handler().await;

            print!("Room {} dropped", room_id);
            let mut transmitter_lock = room_message_transmitters_clone.write();
            transmitter_lock.remove(&room_id);
        });

        self.add_room_transmitter(room_id, room_message_transmitter);

        room_id
    }

    fn join_room_and_forward_message(
        &mut self,
        room_id: Uuid,
        user_id: Uuid,
        address: CurverAddress,
    ) {
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

    fn leave_room_and_forward_message(&mut self, user_id: Uuid, address: CurverAddress) {
        self.send_message_to_room_by_user_id(
            user_id,
            ForwardedMessage {
                user_id,
                address,
                message: CurverMessageToReceive::LeaveRoom,
            },
        );

        self.room_map.remove(&user_id);
    }

    //  --- Message Forwarding ---
    fn send_message_to_room_by_user_id(&mut self, user_id: Uuid, message: ForwardedMessage) {
        if let Some(room_id) = self.room_map.get(&user_id) {
            self.send_message_to_room(*room_id, message);
        } else {
            println!("User {} is not in a room", user_id);
        }
    }

    fn send_message_to_room(&self, room_id: Uuid, message: ForwardedMessage) {
        let transmitter_lock = self.room_message_transmitters.read();

        if let Some(transmitter) = transmitter_lock.get(&room_id) {
            transmitter.try_send(message).unwrap();
        } else {
            println!("Room {} does not exist", room_id);
        }
    }

    // --- Message Transmitter ---
    fn add_room_transmitter(&mut self, room_id: Uuid, transmitter: Sender<ForwardedMessage>) {
        let mut transmitter_lock = self.room_message_transmitters.write();
        transmitter_lock.insert(room_id, transmitter);
    }
}
