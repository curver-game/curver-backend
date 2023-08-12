use tokio::sync::mpsc::Receiver;

use crate::{
    message::{CurverMessageToReceive, CurverMessageToSend, ForwardedMessage, UuidSerde},
    room::RoomHandler,
};

pub async fn internal_message_handler(mut internal_message_receiver: Receiver<ForwardedMessage>) {
    let mut room_handler = RoomHandler::new();

    loop {
        if let Some(forwarded_message) = internal_message_receiver.recv().await {
            match forwarded_message.message {
                CurverMessageToReceive::CreateRoom => {
                    let room_id = room_handler.create_room();

                    forwarded_message
                        .address
                        .do_send(CurverMessageToSend::CreatedRoom {
                            room_id: UuidSerde(room_id),
                        });
                }

                CurverMessageToReceive::JoinRoom { room_id } => {
                    room_handler.join_room(
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
                    room_handler
                        .leave_room(forwarded_message.user_id, forwarded_message.address.clone());

                    forwarded_message
                        .address
                        .do_send(CurverMessageToSend::LeftRoom);
                }

                CurverMessageToReceive::Rotate {
                    angle_unit_vector_x,
                    angle_unit_vector_y,
                } => {
                    room_handler.send_message_to_room(
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
            }
        }
    }
}
