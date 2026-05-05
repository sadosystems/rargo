use crate::errors::CliResult;
use std::path::PathBuf;

pub struct RemoteConfig {
    host: String,
    team: String,
    scope: String,
}

pub struct WorkspaceContext {
    root_dir: PathBuf,
    /// None if abrasive is called from the workspace root
    subdir: Option<String>,
    remote_config: RemoteConfig,
}

pub fn get_workspace() -> CliResult<Option<WorkspaceContext>> {
    todo!("get_workspace")
}
