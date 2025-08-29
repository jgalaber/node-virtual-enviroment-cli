use async_trait::async_trait;
use nve_core::error::NveError;
use nve_core::ports::platform::Platform;
use std::fs;
use std::path::{Path};
use std::os::unix::fs as unix_fs;

pub struct UnixPlatform;

impl UnixPlatform {
    pub fn new() -> Result<Self, NveError> { Ok(Self) }
}

#[async_trait]
impl Platform for UnixPlatform {
    fn os_arch(&self) -> (String, String) {
        // os: "linux" | "darwin", arch: "x64" | "arm64"
        let os = if cfg!(target_os = "macos") { "darwin" } else { "linux" }.to_string();
        let arch = if cfg!(target_arch = "x86_64") { "x64" } else if cfg!(target_arch = "aarch64") { "arm64" } else { "x64" }.to_string();
        (os, arch)
    }

    fn archive_name(&self, version: &str) -> String {
        let (os, arch) = self.os_arch();
        format!("node-v{}-{}-{}.tar.xz", version, os, arch)
    }

    async fn set_current(&self, version_dir: &Path, current_dir: &Path) -> Result<(), NveError> {
        if current_dir.exists() || current_dir.is_symlink() {
            let _ = fs::remove_file(current_dir);
            let _ = fs::remove_dir_all(current_dir);
        }
        unix_fs::symlink(version_dir, current_dir)?;
        Ok(())
    }

    async fn is_current(&self, version: &str, current_dir: &Path) -> Result<bool, NveError> {
        if let Ok(link) = fs::read_link(current_dir) {
            Ok(link.ends_with(version))
        } else {
            Ok(false)
        }
    }
}
