use std::env;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

pub fn saved_token() -> Option<String> {
    let path = credentials_path()?;
    let raw = fs::read_to_string(&path).ok()?;
    let parsed: toml::Value = toml::from_str(&raw).ok()?;
    parsed
        .get("abrasive")?
        .get("token")?
        .as_str()
        .map(String::from)
}

fn credentials_path() -> Option<PathBuf> {
    let home = env::var_os("HOME")
        .or_else(|| env::var_os("USERPROFILE"))
        .map(PathBuf::from)?;
    Some(home.join(".abrasive").join("credentials.toml"))
}
