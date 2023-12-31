use actix::{Actor, Addr, AsyncContext, Handler, StreamHandler};
use actix_web_actors::ws::{self, Message, ProtocolError, WebsocketContext};
use tokio::sync::mpsc::Sender;
use uuid::Uuid;

use crate::{
    game::player::PlayerUuid,
    message::{CurverMessageToReceive, CurverMessageToSend, ForwardedMessage},
};

pub struct CurverWebSocketActor {
    pub id: PlayerUuid,
    pub internal_message_transmitter: Sender<ForwardedMessage>,
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
    fn handle(
        &mut self,
        msg: Result<actix_web_actors::ws::Message, ProtocolError>,
        ctx: &mut Self::Context,
    ) {
        if let Ok(Message::Text(text)) = msg {
            let message_serialized = serde_json::from_str::<CurverMessageToReceive>(&text);

            if let Ok(message_serialized) = message_serialized {
                self.internal_message_transmitter
                    .try_send(ForwardedMessage {
                        message: message_serialized,
                        user_id: self.id,
                        address: ctx.address(),
                    });
            } else {
                ctx.address().do_send(CurverMessageToSend::FaultyMessage {
                    message: text.to_string(),
                });
            }
        }
    }

    fn finished(&mut self, ctx: &mut Self::Context) {
        // TODO: Handle result
        let result = self
            .internal_message_transmitter
            .try_send(ForwardedMessage {
                user_id: self.id,
                address: ctx.address(),
                message: CurverMessageToReceive::LeaveRoom,
            });
    }
}

pub type CurverAddress = Addr<CurverWebSocketActor>;
