use crate::services::chat::chat_server::ChatServer;
use crate::services::chat::chat_session::ChatSession;
use actix_web::{web, Error, HttpRequest, HttpResponse};
use actix_web_actors::ws;
use log::{error, info};

pub async fn chat_route(
    req: HttpRequest,
    stream: web::Payload,
    srv: web::Data<actix::Addr<ChatServer>>,
) -> Result<HttpResponse, Error> {
    info!("Received WebSocket connection request");
    let chat_server_address = srv.get_ref().clone();
    let chat_session = ChatSession::new(chat_server_address);

    info!("Starting new WebSocket connection");
    ws::start(chat_session, &req, stream).map_err(|e| {
        error!("Failed to start WebSocket: {}", e);
        e
    })
}
