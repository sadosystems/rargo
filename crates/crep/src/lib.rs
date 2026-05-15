//! crep: Cargo Remote Execution Protocol
//!
//! This crate defines the protocol used by the abrasive cli and the abrasive broker.
//! It defines serialization for the messages in the protocol and it also serves
//! as specification / documentation.   
//!
//! The crep is inspired in many ways by the REAPI, these are the key differences in
//! the design goals.
//! 1. NOT horizontally scalable, the abrasive solution to horizontal scale is run
//!    another instance.
//! 2. Low Latency.
//!
//! Taking these two things together, in crep both the client and the server hold more
//! session state than the equivalent in REAPI.
//! that lets us do more cheap tricks to keep latency low. All logic that relies on
//! ephemeral session state, must have a fallback path. If the client or the server
//! restarts mid session the worst that should be able to happen is a slowdown.

mod errors;

pub use errors::DecodeError;

use serde::{Deserialize, Serialize};

// ANSI-colored origin tags. (w/ reset)
// [LOCAL] is teal-ish blue
// [REMOTE] in abrasive's gold/orange.

pub const LOCAL: &str = "\x1b[38;2;100;200;220m[LOCAL] \x1b[0m";
pub const REMOTE: &str = "\x1b[38;2;232;185;49m[REMOTE]\x1b[0m";

#[macro_export]
macro_rules! local {
    ($($arg:tt)*) => {
        eprintln!("{} {}", $crate::LOCAL, format_args!($($arg)*))
    };
}

#[macro_export]
macro_rules! remote {
    ($($arg:tt)*) => {
        eprintln!("{} {}", $crate::REMOTE, format_args!($($arg)*))
    };
}

// ---- Messages -----

#[derive(Debug, Serialize, Deserialize)]
pub struct Message {
    msg_type: MessageType,
    metadata: MessageMetadata,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MessageMetadata {
    /// The instance name is a unique identifier for a given instance of abrasive.
    /// An instance has: exactly one broker, N workers and optionally an  
    /// observability SPA server.
    instance_name: String,

    /// Unique user id, scoped by instance (not universally unique).
    user_id: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum MessageType {
    /// The client asks the broker to execute a cargo command.
    CommandRequest(CommandRequest),

    /// The broker returns the exit code for the cargo command and a list of
    /// CAS paths for the binaries built by this cargo invocation.
    ///
    /// Binaries are found with this flag:
    ///     --message-format=json-render-diagnostics
    CommandResponse(CommandResponse),

    /// The client asks for files from the CAS
    BulkFileRequest {
        files: Vec<Digest>,
    },

    /// The broker returns files from the CAS
    BulkFileResponse {
        files: Vec<File>,
    },

    CargoStdout(Vec<u8>),

    CargoStderr(Vec<u8>),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CommandRequest {
    command: CargoCommand,
    /// For speculative sync. This can be empty, the broker will ask for any
    /// missing files.
    files: Vec<File>,
    /// src everything digest so the broker can check if it needs to ask for
    /// any missing files.
    source_digest: [u8; 32],
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CommandResponseSuccess {
    exit_code: u8,
    files: Vec<File>, // bins
}

#[derive(Debug, Serialize, Deserialize)]
pub enum CommandResponse {
    Complete(CommandResponseSuccess),
    NeedFiles(Vec<File>),
}

// ---- Types -----

#[derive(Debug, Serialize, Deserialize)]
pub struct CargoCommand {
    /// The arguments passed to abrasive meant for cargo, split on whitespace.
    /// this does not include the 'cargo' / 'abrasive' part of the command.
    /// for example:
    /// ["build", "--lib", "-p", "crep"]
    args: Vec<String>,

    /// If the cargo flag '--target' is not explicitly set, the broker target
    /// host_platform.
    ///
    /// This defines the target platform the binary must be built for. For remote
    /// test execute, this constrains the execution environment. The server MUST
    /// choose to execute the test on a worker satisfying the requirements of the
    /// selected target.
    ///
    /// The expected use for this is the client detects the host Platform triple.
    /// That way the server defaults to targeting the same arch as the client.
    host_platform: PlatformTriple,

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

#[derive(Debug, Serialize, Deserialize)]
pub struct Digest {
    // The hash, represented as a lowercase hexadecimal string, padded with
    // leading zeroes up to the hash function length.
    hash: String,
    // The size of the blob, in bytes.
    size_bytes: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct File {
    /// The full path of the file relative to the working directory, including the
    /// filename. The path separator is a forward slash `/`. Since this is a
    /// relative path, it MUST NOT begin with a leading forward slash.
    path: String,

    /// The digest of the file's content.
    digest: Digest,

    /// The contents of the file if inlining is enabled for the message.
    /// The session-state-ful part of the protocol decides if it wants to
    /// populate this or not (speculates the client is likely to have the
    /// contents in its local cache.
    contents: Vec<u8>,
}

// ---- Platform -----

/// Platform triple, used to specify the cross compilation target.
#[derive(Debug, Serialize, Deserialize)]
pub struct PlatformTriple {
    pub arch: Arch,
    pub os: Os,
    pub abi: Abi,
}

impl PlatformTriple {
    pub fn as_cargo_target_string(&self) -> String {
        match (&self.arch, &self.os, &self.abi) {
            (Arch::X86_64, Os::Linux, Abi::Gnu) => "x86_64-unknown-linux-gnu",
            (Arch::X86_64, Os::Linux, Abi::Musl) => "x86_64-unknown-linux-musl",
            (Arch::Aarch64, Os::Linux, Abi::Gnu) => "aarch64-unknown-linux-gnu",
            (Arch::Aarch64, Os::Linux, Abi::Musl) => "aarch64-unknown-linux-musl",
            (Arch::X86_64, Os::Windows, Abi::Msvc) => "x86_64-pc-windows-msvc",
            (Arch::X86_64, Os::Windows, Abi::Gnu) => "x86_64-pc-windows-gnu",
            (Arch::Aarch64, Os::Windows, Abi::Msvc) => "aarch64-pc-windows-msvc",
            (Arch::X86_64, Os::Mac, _) => "x86_64-apple-darwin",
            (Arch::Aarch64, Os::Mac, _) => "aarch64-apple-darwin",
            _ => unimplemented!(),
        }
        .to_string()
    }
}

/// Architecture
#[derive(Debug, Serialize, Deserialize)]
#[repr(u8)]
pub enum Arch {
    X86_64 = 0,
    Aarch64 = 1,
}

/// Operating System
#[derive(Debug, Serialize, Deserialize)]
#[repr(u8)]
pub enum Os {
    Windows = 0,
    Linux = 1,
    Mac = 2,
}

/// Application Binary Interface
#[derive(Debug, Serialize, Deserialize)]
#[repr(u8)]
pub enum Abi {
    Gnu = 0,
    Musl = 1,
    Msvc = 2,
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
