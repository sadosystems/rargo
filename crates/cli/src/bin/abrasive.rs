use rargo::commands::handle_command;
use std::process::ExitCode;

/// This is sort of a hacky way to make the CLI feel faster.
/// Avoid the Websocket handshake by leaving it open in another
/// process. The next time the CLI needs to message the broker it will
/// pipe the message to the long lived process (the rargo-agent)
///
/// The agent forwards the message to the broker.
/// The speed up is only noticeable for cache hits (100ms-1s)
// fn spawn_agent_for_next_time() {
//     todo!("spawn_agent_for_next_time")
// }

fn main() -> ExitCode {
    // spawn_agent_for_next_time();
    match handle_command() {
        Ok(code) => code,
        Err(e) => e.exit(),
    }
}
