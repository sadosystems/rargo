use crate::errors::CliError;
use crate::errors::CliResult;
use crate::workspace::{AbrasiveContext, get_workspace};
use clap::builder::styling::{AnsiColor, Styles};
use clap::{CommandFactory, Parser, Subcommand};
use std::env;
use std::process::{Command as Cmd, ExitCode, Stdio};

const STYLES: Styles = Styles::styled()
    .header(AnsiColor::Yellow.on_default().bold())
    .usage(AnsiColor::Yellow.on_default().bold())
    .literal(AnsiColor::Yellow.on_default().bold())
    .placeholder(AnsiColor::Yellow.on_default());

#[derive(Parser)]
#[command(name = "abrasive", disable_version_flag = true, disable_help_flag = true, trailing_var_arg = true, styles = STYLES)]
struct Cli {
    #[command(subcommand)]
    pub command: Option<Command>,

    /// Args to forward to cargo
    #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
    cargo_args: Vec<String>,
}

#[derive(Subcommand)]
enum Command {
    /// Initialize abrasive for this project
    Setup,

    /// Authenticate with the build server
    Auth,

    /// Print abrasive and cargo versions
    #[command(name = "--version", aliases = ["-V"])]
    Version,

    /// Get help for abrasive and cargo
    #[command(name = "--help", aliases = ["-h"])]
    Help,

    /// print a fortune-cookie-like-thing from the broker.
    #[command(name = "splash", aliases = ["-s"])]
    Splash,
}

const ABRASIVE_COMMANDS: &[&str] = &[
    "setup",
    "auth",
    "--version",
    "-V",
    "--help",
    "-h",
    "tip",
    "-t",
];

const BROKER_WHITELIST: &[&str] = &[
    "build", "run", "test", "bench", "check", "clippy", "doc", "nop", "clean",
];

/// Helper for is the second arg (first is abrasive itself) in the
/// LOCAL_ABRASIVE_COMMANDS list. These are the commands that the
/// abrasive client has special handling for (instead of simply
/// forwarding the args to the broker)
fn is_abrasive_command() -> bool {
    env::args()
        .nth(1)
        .map_or(true, |arg| ABRASIVE_COMMANDS.contains(&arg.as_str()))
}

/// Helper for
fn is_on_broker_whitelist(args: &[String]) -> bool {
    args.first()
        .map_or(false, |cmd| BROKER_WHITELIST.contains(&cmd.as_str()))
}

fn dispatch_abrasive_command(
    command: Option<Command>,
    ctx: &Option<AbrasiveContext>,
) -> CliResult<ExitCode> {
    match command {
        None => print_help(),
        Some(thing) => match thing {
            Command::Setup => remote_setup()?,
            Command::Auth => login()?,
            Command::Version => print_version(),
            Command::Help => print_help(),
            Command::Splash => print_splash()?,
        },
    }
    Ok(ExitCode::SUCCESS)
}

pub fn remote_setup() -> CliResult<()> {
    todo!("remote_setup")
}

pub fn login() -> CliResult<()> {
    todo!("login")
}

/// Print the Abrasive version string first, followed by the host
/// host version. Just shells out to the host cargo.
fn print_version() {
    println!("abrasive {}", env!("CARGO_PKG_VERSION"));
    let _ = Cmd::new("cargo").arg("--version").status();
}

/// Print the Abrasive help first, followed by the cargo help
fn print_help() {
    println!("ABRASIVE {}\n", env!("CARGO_PKG_VERSION"));
    let _ = Cli::command().color(clap::ColorChoice::Always).print_help();
    let _ = Cmd::new("cargo").arg("--help").status();
}

/// Test the CAS. Get resource at fixed location. print content.
pub fn print_splash() -> CliResult<()> {
    todo!("print_splash")
}

/// Parse the args and pass them straight into the local cargo.
fn forward_args_to_local() -> CliResult<ExitCode> {
    // Transparent on unix, probably close enough on windows
    let args: Vec<String> = env::args().skip(1).collect();
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        let _err = Cmd::new("cargo").args(&args).exec();
        // only reaches here if exec failed
        Err(CliError::CargoNotFound)
    }

    #[cfg(not(unix))]
    {
        let status = Cmd::new("cargo")
            .args(&args)
            .status()
            .map_err(CliError::cargo_not_found)?;
        Ok(ExitCode::from(status.code().unwrap_or(1) as u8))
    }
}

fn try_remote(ctx: &AbrasiveContext, cargo_args: Vec<String>) -> CliResult<ExitCode> {
    // Only whitelisted cargo commands run remotely; everything else
    // (e.g. `clean`, `update`, `add`) falls through to local cargo.
    if cargo_args
        .first()
        .map_or(false, |cmd| BROKER_WHITELIST.contains(&cmd.as_str()))
    {
        return forward_args_to_local();
    }

    let run_args = extract_run_args(&cargo_args);
    let token = auth::saved_token().ok_or(errors::AuthError::NoSavedToken)?;
    let (code, artifact) = poll_for_build(ctx, cargo_args, &token)?;
    if code != 0 {
        return Ok(ExitCode::from(code));
    }
    match (run_args, artifact) {
        (Some(args), Some(art)) => exec_artifact_locally(art, &args),
        _ => Ok(ExitCode::from(code)),
    }
}

pub fn handle_command() -> CliResult<ExitCode> {
    let ctx = get_workspace()?;
    let cli = Cli::parse();

    // Handle all the "abrasive commands". These are all the cli
    // commands which are not just remote cargo commands.
    if is_abrasive_command() {
        return dispatch_abrasive_command(cli.command, &ctx);
    }

    // If The CWD is not an abrasive workspace AND the command is
    // not specific to abrasive, it passes through to the local
    // cargo.
    let ctx = match ctx {
        None => return forward_args_to_local(),
        Some(ctx) => ctx,
    };

    // At this point the command does not need special client
    // handling and the CWD must be an abrasive workspace, send the
    // command to the broker.
    match cli.command {
        None => return try_remote(&ctx, cli.cargo_args),
        _ => unreachable!(), // note to self, consider factoring this out
    }
}
