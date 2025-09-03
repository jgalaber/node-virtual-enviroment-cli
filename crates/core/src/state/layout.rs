use crate::{
    constants::{NODEJS_DIR, VERSION_DIR},
    error::NveError,
};
use std::{env, path::PathBuf};

pub struct NveLayout {
    pub base: PathBuf,
}

impl NveLayout {
    pub fn from_env_or_home() -> Result<Self, NveError> {
        if let Some(p) = env::var_os("NVE_BASE") {
            return Ok(Self {
                base: PathBuf::from(p),
            });
        }
        let home = home_dir_from_os().ok_or(NveError::HomeDirNotFound)?;
        Ok(Self {
            base: home.join(".nve"),
        })
    }

    pub fn versions_dir(&self) -> PathBuf {
        self.base.join(VERSION_DIR)
    }
    pub fn current_dir(&self) -> PathBuf {
        self.base.join(NODEJS_DIR)
    }
    pub fn version_dir(&self, v: &str) -> PathBuf {
        self.versions_dir().join(v)
    }
}

#[cfg(windows)]
fn home_dir_from_os() -> Option<PathBuf> {
    if let Some(up) = env::var_os("USERPROFILE") {
        return Some(up.into());
    }
    match (env::var_os("HOMEDRIVE"), env::var_os("HOMEPATH")) {
        (Some(d), Some(p)) => Some(PathBuf::from(d).join(p)),
        _ => None,
    }
}

#[cfg(not(windows))]
fn home_dir_from_os() -> Option<PathBuf> {
    env::var_os("HOME").map(PathBuf::from)
}
