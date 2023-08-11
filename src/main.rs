use actix::{Actor, Addr, AsyncContext, Handler, Message, StreamHandler};
use actix_web::{web, App, Error, HttpRequest, HttpResponse, HttpServer};
use actix_web_actors::ws;
use tokio::sync::mpsc;

struct MyWs {
    tx: mpsc::Sender<InternalMessage>,
}

impl Actor for MyWs {
    type Context = ws::WebsocketContext<Self>;
}

impl Handler<WsMessage> for MyWs {
    type Result = ();

    fn handle(&mut self, msg: WsMessage, ctx: &mut Self::Context) {
        ctx.text(msg.0);
    }
}

enum InternalMessage {
    NewAddress(Addr<MyWs>),
    SendToAll { message: String, sender: Addr<MyWs> },
}

#[derive(Message)]
#[rtype(result = "()")]
struct WsMessage(String);

/// Handler for ws::Message message
impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for MyWs {
    fn started(&mut self, ctx: &mut Self::Context) {
        let result = self.tx.try_send(InternalMessage::NewAddress(ctx.address()));

        match result {
            Ok(_) => (),
            Err(_) => ctx.text("Couldn't register client"),
        }
    }

    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Ok(ws::Message::Text(text)) => {
                let result = self.tx.try_send(InternalMessage::SendToAll {
                    sender: ctx.address(),
                    message: text.to_string(),
                });

                match result {
                    Ok(_) => (),
                    Err(_) => ctx.text("Couldn't send message"),
                }
            }
            _ => ctx.text("Error"),
        }
    }
}

struct AppState {
    tx: mpsc::Sender<InternalMessage>,
}

#[actix_web::get("/ws")]
async fn index(
    req: HttpRequest,
    stream: web::Payload,
    state: web::Data<AppState>,
) -> Result<HttpResponse, Error> {
    let tx = state.tx.clone();
    let resp = ws::start(MyWs { tx }, &req, stream);
    println!("{:?}", resp);
    resp
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let (tx, mut rx) = mpsc::channel::<InternalMessage>(100);
    let mut address_vec: Vec<Addr<MyWs>> = Vec::new();

    tokio::spawn(async move {
        loop {
            let received = rx.recv().await.unwrap();

            match received {
                InternalMessage::NewAddress(addr) => {
                    address_vec.push(addr);
                }
                InternalMessage::SendToAll { message, sender } => {
                    for addr in address_vec.iter() {
                        if addr == &sender {
                            continue;
                        }

                        addr.do_send(WsMessage(message.to_string()));
                    }
                }
            }
        }
    });

    let app_state = web::Data::new(AppState { tx });

    HttpServer::new(move || App::new().app_data(app_state.clone()).service(index))
        .bind(("127.0.0.1", 8080))?
        .run()
        .await
}
