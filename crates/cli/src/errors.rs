use std::borrow::Cow;
use std::process::ExitCode;

pub enum CliError {
    Connect,
    Io,
    Protocol,
    Disconnected,
    InvalidPath(String),
    NoToml,
    InvalidToml(String),
    NoCwd,
    CargoNotFound,
    NoSavedToken,
    NoHome,
    WriteToken,
    ReadStdin,
    EmptyToken,
    InvalidToken,
    NoMetaData,
    WriteFail,
}

impl CliError {
    pub fn exit(&self) -> ExitCode {
        let (message, exit_code): (Cow<str>, ExitCode) = match self {
            Self::Connect => ("Failed to connect to the broker.".into(), ExitCode::FAILURE),
            Self::Io => ("Broker IO Error.".into(), ExitCode::FAILURE),
            Self::Protocol => ("Invalid response from broker.".into(), ExitCode::FAILURE),
            Self::Disconnected => (
                "Broker closed connection before action finished".into(),
                ExitCode::FAILURE,
            ),
            Self::InvalidPath(p) => (format!("invalid path: {p}").into(), ExitCode::FAILURE),
            Self::NoToml => ("Cannot read rargo.toml".into(), ExitCode::FAILURE),
            Self::InvalidToml(p) => (format!("invalid toml: {p}").into(), ExitCode::FAILURE),
            Self::NoCwd => (
                "Cannot determine current directory.".into(),
                ExitCode::FAILURE,
            ),
            Self::CargoNotFound => ("Cargo not found".into(), ExitCode::from(127)),
            Self::NoSavedToken => (
                "Not logged in, run `rargo auth` first".into(),
                ExitCode::FAILURE,
            ),
            Self::NoHome => ("No HOME or USERPROFILE set".into(), ExitCode::FAILURE),
            Self::WriteToken => ("Could not write credentials file".into(), ExitCode::FAILURE),
            Self::ReadStdin => ("Failed to read token from stdin".into(), ExitCode::FAILURE),
            Self::EmptyToken => ("No token was entered".into(), ExitCode::FAILURE),
            Self::InvalidToken => (
                "Token must start with `rargo_`".into(),
                ExitCode::FAILURE,
            ),
            Self::NoMetaData => (
                "The workspace manifest has no [workspace.metadata.rargo] table".into(),
                ExitCode::FAILURE,
            ),
            Self::WriteFail => ("Write fail".into(), ExitCode::FAILURE),
        };
        eprintln!("{message}");
        exit_code
    }
}
