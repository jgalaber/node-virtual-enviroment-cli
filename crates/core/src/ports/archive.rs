use std::path::Path;

use crate::error::NveError;

#[async_trait::async_trait]
pub trait Archive: Send + Sync {
    async fn extract(&self, data: &[u8], target_dir: &Path, version: &str) -> Result<(), NveError>;
}
