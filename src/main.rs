use std::{collections::HashMap, sync::Arc};

use actix_web::{web, App, Error, HttpRequest, HttpResponse, HttpServer};
use actix_web_actors::ws;
use curver_backend::{
    curver_ws_actor::{CurverAddress, CurverWebSocketActor},
    message::{CurverMessageToReceive, CurverMessageToSend, InternalMessage, UuidSerde},
};
use parking_lot::RwLock;
use tokio::sync::mpsc::{self, Sender};
use uuid::Uuid;

struct Room {
    id: Uuid,
    clients: RwLock<Vec<CurverAddress>>,
}

struct AppState {
    internal_message_transmitter: Sender<InternalMessage>,
}

#[actix_web::get("/ws")]
async fn index(
    req: HttpRequest,
    stream: web::Payload,
    app_state: web::Data<AppState>,
) -> Result<HttpResponse, Error> {
    let id = Uuid::new_v4();
    let actor = CurverWebSocketActor {
        id,
        room_id: None,
        room_message_transmitter: None,
        internal_message_transmitter: app_state.internal_message_transmitter.clone(),
    };

    let resp = ws::start(actor, &req, stream);
    println!("{:?}", resp);
    resp
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let (internal_message_transmitter, mut internal_message_receiver) =
        mpsc::channel::<InternalMessage>(100);
    let client_addresses: Arc<RwLock<HashMap<Uuid, CurverAddress>>> =
        Arc::new(RwLock::new(HashMap::new()));

    // Internal message handler
    tokio::spawn(async move {
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

                        address.do_send(CurverMessageToSend::CreatedRoom {
                            room_id: UuidSerde(Uuid::new_v4()),
                        });
                    }
                    _ => {}
                },
            }
        }
    });

    let app_state = web::Data::new(AppState {
        internal_message_transmitter,
    });

    let app_generator = move || App::new().service(index).app_data(app_state.clone());

    HttpServer::new(app_generator)
        .bind(("127.0.0.1", 8080))?
        .run()
        .await
}
