#![cfg(target_os = "windows")]

use async_trait::async_trait;
use nve_core::error::NveError;
use nve_core::ports::platform::Platform;
use nve_core::state::layout::NveLayout;
use std::fs;
use std::path::Path;

pub struct WindowsPlatform;

impl WindowsPlatform {
    pub fn new() -> Result<Self, NveError> {
        Ok(Self)
    }

    fn version_dir_name_for(&self, version: &str) -> String {
        let (os, arch) = self.os_arch();
        format!("node-v{version}-{os}-{arch}")
    }

    pub async fn set_current_with_layout_name(
        &self,
        layout: &NveLayout,
        version_dir_name: &str,
    ) -> Result<(), NveError> {
        let vdir = layout.version_dir(version_dir_name);
        let cdir = layout.current_dir();
        self.set_current(&vdir, &cdir).await
    }

    pub async fn set_current_with_layout(
        &self,
        layout: &NveLayout,
        version: &str,
    ) -> Result<String, NveError> {
        let name = self.version_dir_name_for(version);
        self.set_current_with_layout_name(layout, &name).await?;
        Ok(name)
    }

    pub async fn is_current_with_layout_name(
        &self,
        layout: &NveLayout,
        version_dir_name: &str,
    ) -> Result<bool, NveError> {
        self.is_current(version_dir_name, &layout.current_dir())
            .await
    }

    pub async fn is_current_with_layout(
        &self,
        layout: &NveLayout,
        version: &str,
    ) -> Result<bool, NveError> {
        let name = self.version_dir_name_for(version);
        self.is_current(&name, &layout.current_dir()).await
    }
}

#[async_trait]
impl Platform for WindowsPlatform {
    fn os_arch(&self) -> (String, String) {
        let os = "win".to_string();
        let arch = if cfg!(target_arch = "aarch64") {
            "arm64"
        } else {
            "x64"
        }
        .to_string();
        (os, arch)
    }

    fn archive_name(&self, version: &str) -> String {
        let (_os, arch) = self.os_arch();
        format!("node-v{version}-win-{arch}.zip")
    }

    async fn set_current(&self, version_dir: &Path, current_dir: &Path) -> Result<(), NveError> {
        if current_dir.exists() {
            let meta = fs::symlink_metadata(current_dir)?;
            if meta.file_type().is_symlink() {
                fs::remove_dir(current_dir)?;
            } else {
                fs::remove_dir_all(current_dir)?;
            }
        }
        if let Some(parent) = current_dir.parent() {
            fs::create_dir_all(parent)?;
        }

        junction::create(version_dir, current_dir).map_err(NveError::Io)?;
        Ok(())
    }

    async fn is_current(
        &self,
        version_dir_name: &str,
        current_dir: &Path,
    ) -> Result<bool, NveError> {
        if !current_dir.exists() {
            return Ok(false);
        }
        let target = fs::canonicalize(current_dir)?;
        Ok(target.ends_with(version_dir_name))
    }
}
