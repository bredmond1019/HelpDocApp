use crate::models::message::Message;
use crate::services::ai_service::AIModel;
use actix::prelude::*;
use futures::StreamExt;
use log::{error, info};
use serde::Deserialize;
use std::collections::HashMap;

use super::chat_session::SessionId;

pub struct ChatServer {
    sessions: HashMap<SessionId, Recipient<Message>>,
    ai_model: AIModel,
}

impl ChatServer {
    pub fn new() -> Self {
        Self {
            sessions: HashMap::new(),
            ai_model: AIModel::new(),
        }
    }
}

impl Actor for ChatServer {
    type Context = Context<Self>;
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct Connect {
    pub addr: Recipient<Message>,
    pub id: SessionId,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct Disconnect {
    pub id: SessionId,
}

#[derive(Message, Deserialize)]
#[rtype(result = "()")]
pub struct ClientMessage {
    pub session_id: SessionId,
    pub message: String,
}

impl Handler<Connect> for ChatServer {
    type Result = ();

    fn handle(&mut self, msg: Connect, _: &mut Context<Self>) {
        info!("ChatSession connected: {:?}", msg.id);
        self.sessions.insert(msg.id, msg.addr);
        info!("Total active sessions: {}", self.sessions.len());
    }
}

impl Handler<Disconnect> for ChatServer {
    type Result = ();

    fn handle(&mut self, msg: Disconnect, _: &mut Context<Self>) {
        info!("ChatSession disconnected: {:?}", msg.id);
        self.sessions.remove(&msg.id);
        info!("Total active sessions: {}", self.sessions.len());
    }
}

impl Handler<ClientMessage> for ChatServer {
    type Result = ResponseFuture<()>;

    fn handle(&mut self, client_message: ClientMessage, _: &mut Context<Self>) -> Self::Result {
        info!(
            "Received message from session {:?}: {}",
            client_message.session_id, client_message.message
        );
        let mut ai_model = self.ai_model.clone();
        let sessions = self.sessions.clone();
        let id = client_message.session_id;

        Box::pin(async move {
            info!("Generating AI response for session {:?}", id);
            match ai_model.generate_response(client_message.message).await {
                Ok(stream) => {
                    info!("AI response stream generated for session {:?}", id);
                    let addr = sessions.get(&id).cloned();
                    if let Some(addr) = addr {
                        tokio::spawn(async move {
                            let mut stream = stream;
                            while let Some(chunk_result) = stream.next().await {
                                match chunk_result {
                                    Ok(chunk) => {
                                        if !chunk.is_empty() {
                                            let ai_message = Message::new(chunk, false);
                                            addr.do_send(ai_message);
                                        }
                                    }
                                    Err(e) => {
                                        error!("Error in AI response stream: {}", e);
                                        let error_message = Message::new(
                                            "Sorry, there was an error processing your request."
                                                .to_string(),
                                            true,
                                        );
                                        addr.do_send(error_message);
                                        break;
                                    }
                                }
                            }
                            // Send end of stream message
                            let end_message = Message::new("".to_string(), true);
                            addr.do_send(end_message);
                        });
                    } else {
                        error!("Session {:?} not found", id);
                    }
                }
                Err(e) => {
                    error!("Failed to generate AI response for session {:?}: {}", id, e);
                    if let Some(addr) = sessions.get(&id) {
                        let error_message = Message::new(
                            "Sorry, I couldn't process your request. Please try again.".to_string(),
                            true,
                        );
                        info!("Sending error message to session {:?}", id);
                        addr.do_send(error_message);
                    }
                }
            }
        })
    }
}
