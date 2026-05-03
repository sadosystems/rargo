mod errors;

pub use errors::DecodeError;

use serde::{Deserialize, Serialize};

// ---- Messages -----

#[derive(Debug, Serialize, Deserialize)]
pub struct Message {
    msg_type: MessageType,
    metadata: MessageMetadata,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MessageMetadata {
    instance_name: String,
    user_id: u32
}

#[derive(Debug, Serialize, Deserialize)]
pub enum MessageType {
    /// Vanity.
    /// tips are basically just our ping.
    TipRequest,
    TipResponse(String),
}

// ---- Types -----



// ---- Helpers -----

// bincode serde wrappers

pub fn serialize(msg: &Message) -> Vec<u8> {
    bincode::serialize(msg).unwrap()
}

pub fn deserialize(raw: &[u8]) -> Result<Message, DecodeError> {
    match bincode::deserialize(raw) {
        Ok(value) => value,
        Err(_) => Err(DecodeError::Deserialize),
    }
}
