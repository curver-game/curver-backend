use actix_web::{web, App, Error, HttpRequest, HttpResponse, HttpServer};
use actix_web_actors::ws;
use clap::Parser;
use curver_backend::{
    config::Config, curver_ws_actor::CurverWebSocketActor, game::player::PlayerUuid,
    message::ForwardedMessage,
};
use tokio::sync::mpsc::{self, Sender};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let config = Config::parse();

    let (internal_message_transmitter, internal_message_receiver) =
        mpsc::channel::<ForwardedMessage>(100);

    let server_handler = curver_backend::server::ServerHandler::new(internal_message_receiver);
    tokio::spawn(async move { server_handler.message_handler().await });

    let app_state = web::Data::new(AppState {
        internal_message_transmitter,
    });

    let app_generator = move || App::new().service(health).service(web_socket).app_data(app_state.clone());

    HttpServer::new(app_generator)
        .bind((config.address, config.port))?
        .run()
        .await
}

#[actix_web::get("/health")]
async fn health() -> HttpResponse {
    HttpResponse::Ok().finish()
}

#[actix_web::get("/ws")]
async fn web_socket(
    req: HttpRequest,
    stream: web::Payload,
    app_state: web::Data<AppState>,
) -> Result<HttpResponse, Error> {
    let id = PlayerUuid::new();
    let actor = CurverWebSocketActor {
        id,
        internal_message_transmitter: app_state.internal_message_transmitter.clone(),
    };

    let resp = ws::start(actor, &req, stream);
    resp
}

struct AppState {
    internal_message_transmitter: Sender<ForwardedMessage>,
}
