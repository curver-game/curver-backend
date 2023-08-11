use actix_web::{web, App, Error, HttpRequest, HttpResponse, HttpServer};
use actix_web_actors::ws;
use curver_backend::{
    curver_ws_actor::CurverWebSocketActor, handler::internal_message_handler,
    message::InternalMessage,
};
use tokio::sync::mpsc::{self, Sender};
use uuid::Uuid;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let (internal_message_transmitter, internal_message_receiver) =
        mpsc::channel::<InternalMessage>(100);

    tokio::spawn(async move { internal_message_handler(internal_message_receiver).await });

    let app_state = web::Data::new(AppState {
        internal_message_transmitter,
    });

    let app_generator = move || App::new().service(index).app_data(app_state.clone());

    HttpServer::new(app_generator)
        .bind(("127.0.0.1", 8080))?
        .run()
        .await
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
        internal_message_transmitter: app_state.internal_message_transmitter.clone(),
    };

    let resp = ws::start(actor, &req, stream);
    println!("{:?}", resp);
    resp
}

struct AppState {
    internal_message_transmitter: Sender<InternalMessage>,
}
