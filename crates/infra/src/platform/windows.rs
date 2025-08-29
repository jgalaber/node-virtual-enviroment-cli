use async_trait::async_trait;
use nve_core::error::NveError;
use nve_core::ports::platform::Platform;
use std::fs;
use std::path::{Path, PathBuf};

pub struct WindowsPlatform;

impl WindowsPlatform {
    pub fn new() -> Result<Self, NveError> { Ok(Self) }
}

#[async_trait]
impl Platform for WindowsPlatform {
    fn os_arch(&self) -> (String, String) {
        // os: "win", arch: "x64" | "arm64"
        let os = "win".to_string();
        let arch = if cfg!(target_arch = "x86_64") { "x64" } else if cfg!(target_arch = "aarch64") { "arm64" } else { "x64" }.to_string();
        (os, arch)
    }

    fn archive_name(&self, version: &str) -> String {
        let (_os, arch) = self.os_arch();
        format!("node-v{}-win-{}.zip", version, arch)
    }

    async fn set_current(&self, version_dir: &Path, current_dir: &Path) -> Result<(), NveError> {
        if current_dir.exists() {
            fs::remove_dir_all(current_dir)?;
        }
        fs::create_dir_all(current_dir)?;
        copy_dir_recursive(version_dir, current_dir)?;
        Ok(())
    }

    async fn is_current(&self, version: &str, current_dir: &Path) -> Result<bool, NveError> {
        Ok(current_dir.join("README.md").exists() || current_dir.exists())
    }
}

fn copy_dir_recursive(from: &Path, to: &Path) -> Result<(), NveError> {
    for entry in walkdir::WalkDir::new(from) {
        let entry = entry.map_err(|e| NveError::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))?;
        let rel = entry.path().strip_prefix(from).unwrap();
        let dest = to.join(rel);
        if entry.file_type().is_dir() {
            fs::create_dir_all(&dest)?;
        } else {
            if let Some(p) = dest.parent() { fs::create_dir_all(p)?; }
            fs::copy(entry.path(), &dest)?;
        }
    }
    Ok(())
}
