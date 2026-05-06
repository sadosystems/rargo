use crate::errors::CliResult;
use crate::workspace::{AbrasiveContext, get_workspace};
use clap::builder::styling::{AnsiColor, Styles};
use clap::{Parser, Subcommand, CommandFactory};
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
    #[command(name = "tip", aliases = ["-t"])]
    Tip,
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

/// Helper for is the second arg (first is abrasive itself) in the
/// LOCAL_ABRASIVE_COMMANDS list. These are the commands that the
/// abrasive client has special handling for (instead of simply
/// forwarding the args to the broker)
fn is_abrasive_command() -> bool {
    env::args()
        .nth(1)
        .map_or(true, |arg| ABRASIVE_COMMANDS.contains(&arg.as_str()))
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
            Command::Tip => print_tip()?,
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

pub fn print_tip() -> CliResult<()> {
    todo!("print_tip")
}

pub fn handle_command() -> CliResult<ExitCode> {
    let ctx = get_workspace()?;

    // Handle all the "abrasive commands", these are commands
    // which should not fall through to local cargo (at least not
    // right away) and which should not forward cargo args to the
    // broker. See ABRASIVE_COMMANDS
    if is_abrasive_command() {
        let cli = Cli::parse();
        return dispatch_abrasive_command(cli.command, &ctx);
    }

    match ctx {
        Some(ctx) => println!("{:?}", ctx.abrasive_config),
        None => println!("Hello"),
    }
    Ok(ExitCode::SUCCESS)
}
