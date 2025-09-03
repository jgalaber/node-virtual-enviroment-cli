use async_trait::async_trait;
use nve_core::error::NveError;
use nve_core::ports::archive::Archive;
use std::fs::{self, File};
use std::io::Cursor;
use std::path::{Path, PathBuf};
use zip::read::ZipArchive as ZipReadArchive;

pub struct ZipArchive;

impl ZipArchive {
    pub fn new() -> Result<Self, NveError> {
        Ok(Self)
    }
}

#[async_trait]
impl Archive for ZipArchive {
    async fn extract(
        &self,
        data: &[u8],
        target_dir: &Path,
        _version: &str,
    ) -> Result<(), NveError> {
        fs::create_dir_all(target_dir)?;

        let reader = Cursor::new(data);
        let mut zip =
            ZipReadArchive::new(reader).map_err(|e| NveError::ExtractError(e.to_string()))?;

        let mut extracted_any = false;

        for i in 0..zip.len() {
            let mut file = zip
                .by_index(i)
                .map_err(|e| NveError::ExtractError(e.to_string()))?;

            let name = file.mangled_name();
            let rel: PathBuf = name.iter().skip(1).collect();
            if rel.as_os_str().is_empty() {
                continue;
            }

            let outpath = target_dir.join(rel);

            if file.name().ends_with('/') || file.is_dir() {
                fs::create_dir_all(&outpath)?;
            } else {
                if let Some(parent) = outpath.parent() {
                    fs::create_dir_all(parent)?;
                }
                let mut outfile = File::create(&outpath)?;
                std::io::copy(&mut file, &mut outfile)?;
            }

            extracted_any = true;
        }

        if !extracted_any {
            return Err(NveError::ExtractError("empty archive".into()));
        }

        Ok(())
    }
}
