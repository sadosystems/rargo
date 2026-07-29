use crate::errors::CliError;
use crate::workspace::rargoContext;

pub type CliResult<T> = Result<T, CliError>;

pub fn command_request(ctx: &rargoContext, cargo_args: Vec<String>) -> CliResult<()> {
    todo!("")
}
