use std::path::PathBuf;

use crate::constants::{NODEJS_DIR, VERSION_DIR};

pub struct NveLayout { pub base: PathBuf } // ~/.nve

impl NveLayout {
    pub fn versions_dir(&self) -> PathBuf { self.base.join(VERSION_DIR) }
    pub fn current_dir(&self)  -> PathBuf { self.base.join(NODEJS_DIR) }
    pub fn version_dir(&self, v: &str) -> PathBuf { self.versions_dir().join(v) }
}
