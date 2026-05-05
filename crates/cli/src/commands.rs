use std::process::ExitCode;

use crate::errors::CliResult;
use crate::workspace::get_workspace;

pub fn handle_command() -> CliResult<ExitCode> {
    let ctx = get_workspace()?;
    todo!("handle_command")
}
