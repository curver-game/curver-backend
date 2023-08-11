use std::{collections::HashMap, sync::Arc};

use parking_lot::RwLock;
use tokio::sync::mpsc::{self, Receiver, Sender};
use uuid::Uuid;

use crate::{
    curver_ws_actor::CurverAddress,
    message::{CurverMessageToReceive, CurverMessageToSend, GameState, InternalMessage, UuidSerde},
};

pub async fn internal_message_handler(mut internal_message_receiver: Receiver<InternalMessage>) {
    let client_addresses: Arc<RwLock<HashMap<Uuid, CurverAddress>>> =
        Arc::new(RwLock::new(HashMap::new()));
    let room_map: HashMap<Uuid, Uuid> = HashMap::new();
    let room_message_transmitters: RwLock<HashMap<Uuid, Sender<InternalMessage>>> =
        RwLock::new(HashMap::new());

    while let Some(message) = internal_message_receiver.recv().await {
        match message {
            InternalMessage::AddAddress { address, user_id } => {
                client_addresses.write().insert(user_id, address);
            }
            InternalMessage::RemoveAddress { user_id } => {
                client_addresses.write().remove(&user_id);
            }
            InternalMessage::HandleMessage { message, user_id } => match message {
                CurverMessageToReceive::CreateRoom => {
                    let addresses_lock = client_addresses.read();
                    let address = addresses_lock.get(&user_id).unwrap();

                    // TODO: Check if user is already in a room
                    let room_id = Uuid::new_v4();
                    let (room_message_transmitter, room_message_receiver) =
                        mpsc::channel::<InternalMessage>(100);

                    let mut room_thread_transmitters_lock = room_message_transmitters.write();
                    room_thread_transmitters_lock.insert(room_id, room_message_transmitter);
                    drop(room_thread_transmitters_lock);

                    let client_addresses_clone = client_addresses.clone();

                    // Room message handler
                    tokio::spawn(async move {
                        room_message_handler(room_message_receiver, client_addresses_clone)
                    });

                    address.do_send(CurverMessageToSend::CreatedRoom {
                        room_id: UuidSerde(room_id.clone()),
                    });
                }
                _ => {}
            },
        }
    }
}

pub async fn room_message_handler(
    mut room_message_receiver: Receiver<InternalMessage>,
    client_addresses: Arc<RwLock<HashMap<Uuid, CurverAddress>>>,
) {
    let mut clients: HashMap<Uuid, CurverAddress> = HashMap::new();

    loop {
        let message = room_message_receiver.recv().await;

        if let Some(message) = message {
            match message {
                InternalMessage::HandleMessage { user_id, message } => {
                    match message {
                        CurverMessageToReceive::JoinRoom { .. } => {
                            let client_addresses_lock = client_addresses.read();
                            let address = client_addresses_lock.get(&user_id).unwrap().clone();
                            clients.insert(user_id, address);
                        }

                        CurverMessageToReceive::LeaveRoom => {
                            clients.remove(&user_id);

                            // Kill the thread if there are no more clients
                            if clients.len() == 0 {
                                break;
                            }
                        }

                        CurverMessageToReceive::Rotate { .. } => clients
                            .get(&user_id)
                            .unwrap()
                            .do_send(CurverMessageToSend::GameState {
                                current_state: GameState::Waiting,
                            }),
                        _ => {}
                    }
                }
                _ => {}
            }
        }
    }
}
