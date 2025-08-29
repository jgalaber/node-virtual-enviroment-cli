use std::path::Path;

use crate::error::NveError;

#[async_trait::async_trait]
pub trait Platform: Send + Sync {
    fn os_arch(&self) -> (String, String);
    fn archive_name(&self, version: &str) -> String;
    async fn set_current(&self, version_dir: &Path, current_dir: &Path) -> Result<(), NveError>;
    async fn is_current(&self, version: &str, current_dir: &Path) -> Result<bool, NveError>;
}
