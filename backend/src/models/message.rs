use actix::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Message, Serialize, Deserialize, Debug, Clone)]
#[rtype(result = "()")]
pub struct Message {
    pub content: String,
    pub is_complete: bool,
}

impl Message {
    pub fn new(content: String, is_complete: bool) -> Self {
        Self {
            content,
            is_complete,
        }
    }
}
