use async_trait::async_trait;
use nve_core::error::NveError;
use nve_core::ports::archive::Archive;
use std::fs::{self};
use std::io::Cursor;
use std::path::Path;
use tar::Archive as TarArchive;
use xz2::read::XzDecoder;

pub struct TarXzArchive;

impl TarXzArchive {
    pub fn new() -> Result<Self, NveError> {
        Ok(Self)
    }
}

#[async_trait]
impl Archive for TarXzArchive {
    async fn extract(
        &self,
        data: &[u8],
        target_dir: &Path,
        _version: &str,
    ) -> Result<(), NveError> {
        use tempfile::tempdir;

        let tmp = tempdir()?;
        TarArchive::new(XzDecoder::new(Cursor::new(data))).unpack(tmp.path())?;

        let root = std::fs::read_dir(tmp.path())?
            .next()
            .ok_or_else(|| NveError::ExtractError("empty archive".into()))??
            .path();

        fs::create_dir_all(target_dir)?;
        for entry in fs::read_dir(root)? {
            let e = entry?;
            fs::rename(e.path(), target_dir.join(e.file_name()))?;
        }
        Ok(())
    }
}
