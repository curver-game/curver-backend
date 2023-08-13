use std::{collections::HashMap, sync::Arc};

use parking_lot::RwLock;
use tokio::sync::mpsc::{self, Receiver, Sender};

use crate::{
    curver_error::ServerError,
    curver_ws_actor::CurverAddress,
    debug_ui::DebugUi,
    game::player::PlayerUuid,
    message::{CurverMessageToReceive, CurverMessageToSend, ForwardedMessage},
    room::{Room, RoomUuid},
};

pub struct ServerHandler {
    room_message_transmitters: Arc<RwLock<HashMap<RoomUuid, Sender<ForwardedMessage>>>>,
    room_map: HashMap<PlayerUuid, RoomUuid>,
    internal_message_receiver: Receiver<ForwardedMessage>,

    debug_ui: DebugUi,
}

impl ServerHandler {
    pub fn new(internal_message_receiver: Receiver<ForwardedMessage>) -> Self {
        let mut debug_ui = DebugUi::new();
        debug_ui.clear();

        Self {
            room_message_transmitters: Arc::new(RwLock::new(HashMap::new())),
            room_map: HashMap::new(),
            internal_message_receiver,
            debug_ui,
        }
    }

    /// This thread will always be running.
    pub async fn message_handler(mut self) {
        loop {
            if let Some(forwarded_message) = self.internal_message_receiver.recv().await {
                match forwarded_message.message {
                    CurverMessageToReceive::CreateRoom => {
                        let room_id = self.create_room();

                        self.join_room_and_forward_message(
                            room_id,
                            forwarded_message.user_id,
                            forwarded_message.address.clone(),
                        );

                        forwarded_message
                            .address
                            .do_send(CurverMessageToSend::JoinedRoom {
                                room_id,
                                user_id: forwarded_message.user_id,
                            });
                    }

                    CurverMessageToReceive::JoinRoom { room_id } => {
                        if !self.check_if_room_exists(room_id) {
                            forwarded_message
                                .address
                                .do_send(CurverMessageToSend::JoinRoomError {
                                    reason: ServerError::RoomDoesNotExist(room_id.get_uuid())
                                        .to_string(),
                                });
                            continue;
                        }

                        self.join_room_and_forward_message(
                            room_id,
                            forwarded_message.user_id,
                            forwarded_message.address.clone(),
                        );

                        forwarded_message
                            .address
                            .do_send(CurverMessageToSend::JoinedRoom {
                                room_id,
                                user_id: forwarded_message.user_id,
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
                        self.send_message_to_room_by_user_id(
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
    fn create_room(&mut self) -> RoomUuid {
        let room_id = RoomUuid::new();
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
        room_id: RoomUuid,
        user_id: PlayerUuid,
        address: CurverAddress,
    ) {
        self.room_map.insert(user_id, room_id);
        self.debug_ui.draw_rooms(self.room_map.clone());

        self.send_message_to_room(
            room_id,
            ForwardedMessage {
                user_id,
                address,
                message: CurverMessageToReceive::JoinRoom { room_id },
            },
        );
    }

    fn leave_room_and_forward_message(&mut self, user_id: PlayerUuid, address: CurverAddress) {
        self.send_message_to_room_by_user_id(
            user_id,
            ForwardedMessage {
                user_id,
                address,
                message: CurverMessageToReceive::LeaveRoom,
            },
        );

        self.room_map.remove(&user_id);
        self.debug_ui.draw_rooms(self.room_map.clone());
    }

    fn check_if_room_exists(&self, room_id: RoomUuid) -> bool {
        self.room_message_transmitters.read().contains_key(&room_id)
    }

    //  --- Message Forwarding ---
    fn send_message_to_room_by_user_id(&mut self, user_id: PlayerUuid, message: ForwardedMessage) {
        if let Some(room_id) = self.room_map.get(&user_id) {
            self.send_message_to_room(*room_id, message);
        } else {
            println!("User {} is not in a room", user_id);
        }
    }

    fn send_message_to_room(&self, room_id: RoomUuid, message: ForwardedMessage) {
        let transmitter_lock = self.room_message_transmitters.read();

        if let Some(transmitter) = transmitter_lock.get(&room_id) {
            transmitter.try_send(message).unwrap();
        } else {
            println!("Room {} does not exist", room_id);
        }
    }

    // --- Message Transmitter ---
    fn add_room_transmitter(&mut self, room_id: RoomUuid, transmitter: Sender<ForwardedMessage>) {
        let mut transmitter_lock = self.room_message_transmitters.write();
        transmitter_lock.insert(room_id, transmitter);
    }
}
