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
    user_id: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum MessageType {
    /// Splash are basically just ping / pong.
    SplashRequest,
    SplashResponse(String),
}

// ---- Types -----

#[derive(Debug, Serialize, Deserialize)]
pub struct CargoCommand {
    /// the arguments passed to abrasive meant for cargo, split on whitespace.
    /// this does not include the 'cargo' / 'abrasive' part of the command. 
    /// for example:
    /// ["build", "--lib", "-p", "crep"]
    args: Vec<String>,

    host_platform: Vec<String>,

    /// The environment variables to set when running the cargo command. The worker
    /// may provide its own default environment variables; these defaults can be
    /// overridden using this field. Additional variables can also be specified.
    ///
    /// In order to ensure that equivalent CargoCommands always hash to the same
    /// value, the environment variables MUST be lexicographically sorted by name.
    /// Sorting of strings is done by code point, or equivalently, by the UTF-8 bytes.
    /// 
    /// Here is how this is meant to be used with abrasive, in the abrasive.toml file
    /// a user may configure a whitelist of env vars that get picked up from the host.
    environment_variables: Vec<EnvironmentVariable>,

    /// The working directory, relative to the input root, for the command to run
    /// in. It must be a directory which exists in the workspace tree. If it is left
    /// empty, then the action is run in the input root.
    working_directory: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EnvironmentVariable {
    /// The variable name.
    name: String,
    /// The variable value.
    value: String,
}


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
