use async_trait::async_trait;
use nve_core::error::NveError;
use nve_core::ports::archive::Archive;
use std::fs::{self, File};
use std::io::{Cursor, Read, Write};
use std::path::{Path, PathBuf};
use zip::ZipArchive;

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
        let mut zip = ZipArchive::new(reader).map_err(|e| NveError::extract_err(e.to_string()))?;

        for i in 0..zip.len() {
            let mut file = zip
                .by_index(i)
                .map_err(|e| NveError::extract_err(e.to_string()))?;
            let outpath = target_dir.join(file.mangled_name());

            if (&*file.name()).ends_with('/') {
                fs::create_dir_all(&outpath)?;
            } else {
                if let Some(p) = outpath.parent() {
                    fs::create_dir_all(p)?;
                }
                let mut outfile = File::create(&outpath)?;
                std::io::copy(&mut file, &mut outfile)?;
            }
        }
        Ok(())
    }
}
