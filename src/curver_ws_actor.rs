use actix::{Actor, Addr, AsyncContext, Handler, StreamHandler};
use actix_web_actors::ws::{self, Message, ProtocolError, WebsocketContext};
use tokio::sync::mpsc::Sender;
use uuid::Uuid;

use crate::message::{CurverMessageToReceive, CurverMessageToSend, InternalMessage};

pub struct CurverWebSocketActor {
    pub id: Uuid,
    pub room_id: Option<Uuid>,
    pub room_message_transmitter: Option<Sender<CurverMessageToSend>>,
    pub internal_message_transmitter: Sender<InternalMessage>,
}

impl Actor for CurverWebSocketActor {
    type Context = WebsocketContext<Self>;
}

impl Handler<CurverMessageToSend> for CurverWebSocketActor {
    type Result = ();

    fn handle(&mut self, msg: CurverMessageToSend, ctx: &mut Self::Context) -> Self::Result {
        // TODO: remove unwrap
        ctx.text(serde_json::to_string(&msg).unwrap())
    }
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for CurverWebSocketActor {
    fn started(&mut self, ctx: &mut Self::Context) {
        // TODO: Handle result
        let result = self
            .internal_message_transmitter
            // TODO: remove try_send
            .try_send(InternalMessage::AddAddress {
                address: ctx.address(),
                user_id: self.id,
            });
    }

    fn handle(
        &mut self,
        msg: Result<actix_web_actors::ws::Message, ProtocolError>,
        _ctx: &mut Self::Context,
    ) {
        if let Ok(Message::Text(text)) = msg {
            let message_serialized = serde_json::from_str::<CurverMessageToReceive>(&text).unwrap();
            match message_serialized {
                CurverMessageToReceive::Rotate { .. } => {
                    // Rotate will be handled by the room message handler
                }
                _ => {
                    // Rest of the messages can be sent directly to the internal message handler
                    self.internal_message_transmitter
                        .try_send(InternalMessage::HandleMessage {
                            message: message_serialized,
                            user_id: self.id,
                        })
                        .unwrap();
                }
            }
        }
    }

    fn finished(&mut self, ctx: &mut Self::Context) {
        // TODO: Handle result
        let result = self
            .internal_message_transmitter
            .try_send(InternalMessage::RemoveAddress { user_id: self.id });
    }
}

pub type CurverAddress = Addr<CurverWebSocketActor>;
