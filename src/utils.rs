use serde::{Serialize, Deserialize};

use std::sync::Mutex;

#[derive(Serialize, Deserialize)]
pub struct Message {
    name: String,
    msg: String
}

#[derive(Serialize, Deserialize)]
pub struct Messages {
    pub message_count: Mutex<u32>,
    pub messages: Mutex<Vec<Message>>
}

impl Messages {
    pub fn new() -> Self {
        Messages { message_count: Mutex::new(0), messages: Mutex::new(Vec::new()) }
    }
}
