use crate::errors::CliError;
use crate::workspace::AbrasiveContext;

pub type CliResult<T> = Result<T, CliError>;

pub fn command_request(ctx: &AbrasiveContext, cargo_args: Vec<String>) -> CliResult<()> {
    todo!("")
}
