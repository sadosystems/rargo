use crate::auth;
use crate::errors::CliError;
use crate::client::CliResult;
use crate::workspace::{AbrasiveContext, get_workspace};
use clap::builder::styling::{AnsiColor, Styles};
use clap::{CommandFactory, Parser, Subcommand};
use crep::local;
use std::env;
use std::fs;
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::process::{Command as Cmd, ExitCode};
use std::thread;
use std::time::Duration;

const STYLES: Styles = Styles::styled()
    .header(AnsiColor::Yellow.on_default().bold())
    .usage(AnsiColor::Yellow.on_default().bold())
    .literal(AnsiColor::Yellow.on_default().bold())
    .placeholder(AnsiColor::Yellow.on_default());

pub struct Bin {
    pub name: String,
    pub contents: Vec<u8>,
}

enum BuildOutcome {
    Done(u8, Option<Bin>),
    SlotsBusy,
}

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
    "build", "run", "test", "bench", "check", "clippy", "doc", "nop", "clean", "install",
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
    _ctx: &Option<AbrasiveContext>, // I'll probably use this.
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

fn extract_post_dash(cargo_args: &[String]) -> Vec<String> {
    cargo_args
        .iter()
        .position(|a| a == "--")
        .map(|idx| cargo_args[idx + 1..].to_vec())
        .unwrap_or_default()
}

fn is_run(cargo_args: &[String]) -> bool {
    cargo_args.first().map(String::as_str) == Some("run")
}

fn attempt_build(
    ctx: &AbrasiveContext,
    cargo_args: &[String],
    token: &str,
) -> CliResult<BuildOutcome> {
    panic!("attempt_build")
}

fn poll_for_build(
    ctx: &AbrasiveContext,
    cargo_args: Vec<String>,
    token: &str,
) -> CliResult<(u8, Option<Bin>)> {
    loop {
        match attempt_build(ctx, &cargo_args, token)? {
            BuildOutcome::Done(code, bin) => break Ok((code, bin)),
            BuildOutcome::SlotsBusy => {
                eprintln!("dumb I know");
                thread::sleep(Duration::from_secs(2));
            }
        }
    }
}

fn run_bin(art: Bin, args: &[String]) -> CliResult<ExitCode> {
    let path = write_temp_executable(&art.name, &art.contents)?;
    local!("running {}", path.display());
    let status = Cmd::new(&path)
        .args(args)
        .status()
        .map_err(|_| CliError::WriteFail)?;
    Ok(ExitCode::from(status.code().unwrap_or(1) as u8))
}

fn write_temp_executable(name: &str, contents: &[u8]) -> CliResult<PathBuf> {
    let path = env::temp_dir().join(format!("abrasive-run-{name}"));
    fs::write(&path, contents).ok().ok_or(CliError::WriteFail)?;

    #[cfg(unix)]
    {
        let mut perms = fs::metadata(&path)
            .map_err(|_| CliError::WriteFail)?
            .permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&path, perms).map_err(|_| CliError::WriteFail)?;
    }
    Ok(path)
}

fn send_broker_cmd(ctx: &AbrasiveContext, cargo_args: Vec<String>) -> CliResult<ExitCode> {
    // Only whitelisted cargo commands get forwarded to the
    if cargo_args
        .first()
        .map_or(false, |cmd| BROKER_WHITELIST.contains(&cmd.as_str()))
    {
        return forward_args_to_local();
    }

    let post_dash = extract_post_dash(&cargo_args);
    let run = is_run(&cargo_args);

    let resp = client::command_request(ctx, cargo_args)?;

    if resp.code != 0 {
        return Ok(ExitCode::from(resp.code));
    }

    match (run, bin) {
        (true, Some(b)) => run_bin(b, &post_dash),
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
        None => return send_broker_cmd(&ctx, cli.cargo_args),
        _ => unreachable!(), // note to self, consider factoring this out
    }
}
