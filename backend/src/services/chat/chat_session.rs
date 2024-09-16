use crate::models::message::Message;
use crate::services::chat::chat_server::{ChatServer, ClientMessage, Connect, Disconnect};
use actix::prelude::*;
use actix::{Actor, ActorContext, AsyncContext, Handler, StreamHandler};
use actix_web_actors::ws;
use log::{error, info};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct SessionId(pub Uuid);

pub struct ChatSession {
    id: SessionId,
    addr: Addr<ChatServer>,
}

impl ChatSession {
    pub fn new(addr: Addr<ChatServer>) -> Self {
        Self {
            id: SessionId(Uuid::new_v4()),
            addr,
        }
    }
}

impl Actor for ChatSession {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        info!("Starting new ChatSession: {:?}", self.id);
        let addr = ctx.address();
        self.addr
            .send(Connect {
                addr: addr.recipient(),
                id: self.id,
            })
            .into_actor(self)
            .then(|res, act, ctx| {
                match res {
                    Ok(_) => info!("ChatSession connected: {:?}", act.id),
                    Err(e) => {
                        error!("Error connecting ChatSession {:?}: {}", act.id, e);
                        ctx.stop();
                    }
                }
                fut::ready(())
            })
            .wait(ctx);

        let response = serde_json::json!({
            "type": "chat_session_started",
            "session_id": self.id.0,
        });

        info!(
            "Sending chat_session_started message to client: {:?}",
            self.id
        );
        ctx.text(response.to_string());
    }

    fn stopping(&mut self, ctx: &mut Self::Context) -> Running {
        info!("Stopping ChatSession: {:?}", self.id);
        self.addr.do_send(Disconnect { id: self.id });
        ctx.text("Chat Session Stopped");
        Running::Stop
    }
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for ChatSession {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Ok(ws::Message::Text(text)) => {
                let msg = text.trim();
                if !msg.is_empty() {
                    match serde_json::from_str::<ClientMessage>(msg) {
                        Ok(client_message) => {
                            self.addr.do_send(client_message);
                        }
                        Err(e) => {
                            error!("Error deserializing ClientMessage: {:?}", e);
                            ctx.text(format!("Error: Invalid message format - {}", e));
                        }
                    }
                }
            }
            Ok(ws::Message::Ping(msg)) => {
                info!("Received ping from client: {:?}", self.id);
                ctx.pong(&msg)
            }
            Ok(ws::Message::Close(reason)) => {
                info!("WebSocket closed for session {:?}: {:?}", self.id, reason);
                ctx.close(reason);
                ctx.stop();
            }
            Err(e) => {
                error!("WebSocket error on session {:?}: {}", self.id, e);
                ctx.stop();
            }
            _ => info!("Received other type of message from client: {:?}", self.id),
        }
    }
}

impl Handler<Message> for ChatSession {
    type Result = ();

    fn handle(&mut self, msg: Message, ctx: &mut Self::Context) {
        let ai_message = serde_json::json!({
            "type": "ai_message",
            "message": msg,
        });
        ctx.text(ai_message.to_string());
    }
}
