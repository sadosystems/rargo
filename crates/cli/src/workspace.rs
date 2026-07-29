use crate::client::CliResult;
use crate::errors::CliError;
use crep::EnvironmentVariable;
use serde::Deserialize;
use std::env;
use std::fs::read_to_string;
#[cfg(unix)]
use std::os::unix::ffi::OsStringExt;
use std::path::{Path, PathBuf};
use std::process::Command;

// These three Struct defs only exist for the purpose
// of deriving a more typed toml serde. Manifest
// WorkspaceSection and MetadataSection are here to make it
// easy to extract the [workspace.metadata.rargo] table
// from Cargo.toml

#[derive(Deserialize)]
struct Manifest {
    workspace: Option<WorkspaceSection>,
}
#[derive(Deserialize)]
struct WorkspaceSection {
    metadata: Option<MetadataSection>,
}
#[derive(Deserialize)]
struct MetadataSection {
    rargo: rargoConfig,
}

/// The actual configuration values stored in
/// [workspace.metadata.rargo]
#[derive(Deserialize, Debug)]
pub struct rargoConfig {
    pub host: String,
    pub team: String,
    pub scope: String,
}

/// Context for an rargo call. This stores all the information the CLI
/// or the broker will need which is missing from the args passed directly
/// to the CLI. See crep::CargoCommand
pub struct rargoContext {
    pub subdir: PathBuf,
    pub rargo_config: rargoConfig,
    pub environment_variables: Vec<EnvironmentVariable>,
}

impl rargoContext {
    pub fn from_paths(cargo_toml_path: &Path, called_from: &Path) -> CliResult<Self> {
        let parent = cargo_toml_path.parent();
        assert!(
            parent.is_some(),
            "cargo_toml_path must end in a filename: {cargo_toml_path:?}"
        );
        let root_dir = parent.unwrap().to_path_buf();
        let subdir = relative_subdir(&root_dir, called_from)?;

        let cargo_toml = read_to_string(cargo_toml_path)
            .ok()
            .ok_or(CliError::NoToml)?;
        // let cargo_toml: toml::Value =
        //     toml::from_str(&cargo_toml).map_err(|e| CliError::InvalidToml(e.to_string()))?;

        let cargo_toml: Manifest =
            toml::from_str(&cargo_toml).map_err(|e| CliError::InvalidToml(e.to_string()))?;
        let rargo_config = cargo_toml
            .workspace
            .and_then(|w| w.metadata)
            .map(|m| m.rargo)
            .ok_or(CliError::NoMetaData)?;

        let ctx = Self {
            rargo_config,
            subdir,
            environment_variables: vec![],
        };
        Ok(ctx)
    }
}

/// Get called_from path relative to the provided root.
/// for example, "c/d" from ("a/b", "a/b/c/d")
fn relative_subdir(project_root: &Path, called_from: &Path) -> CliResult<PathBuf> {
    called_from
        .strip_prefix(project_root)
        .map(Path::to_path_buf)
        .map_err(|_| CliError::InvalidPath(called_from.display().to_string()))
}

/// Using cargo's mechanism for finding the workspace toml, this is NOT
/// THE SAME as crawling up parent dirs and stopping on the first one
/// with a Cargo.toml. Ref:
/// https://doc.rust-lang.org/cargo/commands/cargo-locate-project.html
/// https://doc.rust-lang.org/cargo/reference/workspaces.html
fn get_workspace_toml_path() -> CliResult<Option<PathBuf>> {
    let output = Command::new("cargo")
        .args(["locate-project", "--workspace", "--message-format=plain"])
        .output()
        .ok()
        .ok_or(CliError::CargoNotFound)?;

    if !output.status.success() {
        // handles the negative cargo response "error: could not find
        // `Cargo.toml` in {cwd} or any parent directory
        return Ok(None);
    }

    // Reading the response from cargo. With the --message-format=plain
    // flag, the response will just be the bare path to the Cargo.toml
    // at the workspace root.
    let mut bytes = output.stdout;
    while matches!(bytes.last(), Some(b) if b.is_ascii_whitespace()) {
        bytes.pop();
    }

    #[cfg(unix)]
    let path = { PathBuf::from(std::ffi::OsString::from_vec(bytes)) };
    #[cfg(windows)]
    let path = PathBuf::from(
        String::from_utf8(bytes)
            .expect("expect is warranted here because cargo always emits UTF-8 for this command"),
    );
    Ok(Some(path))
}

/// Returns the rargo Workspace context. The configuration for the
/// rargo workspace is stored in the workspace root Cargo.toml
/// file. Uses the blessed mechanism for storing custom tool specific
/// metadata in the Cargo.toml file at the workspace root. Ref:
/// https://doc.rust-lang.org/cargo/reference/workspaces.html#the-metadata-table
///
/// For rargo, that metadata looks like:
///
/// ```
/// [workspace.metadata.rargo]
/// host = "157.180.55.180"
/// team = "public"
/// scope = "rargo"
/// ```
pub fn get_workspace() -> CliResult<Option<rargoContext>> {
    let cwd = env::current_dir().ok().ok_or(CliError::NoCwd)?;

    let rargo_ctx = get_workspace_toml_path()?
        .map(|config| rargoContext::from_paths(&config, &cwd))
        .transpose()?;
    Ok(rargo_ctx)
}
