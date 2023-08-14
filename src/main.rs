use actix_web::{
    web::{self, ServiceConfig},
    Error, HttpRequest, HttpResponse,
};
use actix_web_actors::ws;
use curver_backend::{
    curver_ws_actor::CurverWebSocketActor, game::player::PlayerUuid, message::ForwardedMessage,
};
use shuttle_actix_web::ShuttleActixWeb;
use tokio::sync::mpsc::{self, Sender};

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

#[shuttle_runtime::main]
async fn main() -> ShuttleActixWeb<impl FnOnce(&mut ServiceConfig) + Send + Clone + 'static> {
    let (internal_message_transmitter, internal_message_receiver) =
        mpsc::channel::<ForwardedMessage>(100);

    let server_handler = curver_backend::server::ServerHandler::new(internal_message_receiver);
    tokio::spawn(async move { server_handler.message_handler().await });

    let app_state = web::Data::new(AppState {
        internal_message_transmitter,
    });

    let service_config = move |cfg: &mut ServiceConfig| {
        cfg.app_data(app_state.clone())
            .service(health)
            .service(web_socket);
    };

    Ok(service_config.into())
}
