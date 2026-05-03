mod errors;

pub use errors::DecodeError;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum Message {
    /// Vanity.
    /// tips are basically just our ping.
    TipRequest,
    TipResponse(String),
}

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
