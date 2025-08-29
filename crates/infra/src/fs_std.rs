use nve_core::error::NveError;
use nve_core::ports::fs::FileSystem;
use std::fs;
use std::path::{Path, PathBuf};

pub struct StdFs;

impl StdFs {
    pub fn new() -> Self {
        Self
    }
}

impl Default for StdFs {
    fn default() -> Self {
        Self::new()
    }
}

impl FileSystem for StdFs {
    fn create_dir_all(&self, path: &Path) -> Result<(), NveError> {
        Ok(fs::create_dir_all(path)?)
    }

    fn remove_dir_all(&self, path: &Path) -> Result<(), NveError> {
        // No falla si no existe
        if path.exists() {
            Ok(fs::remove_dir_all(path)?)
        } else {
            Ok(())
        }
    }

    fn read_dir_names(&self, path: &Path) -> Result<Vec<String>, NveError> {
        let mut out = Vec::new();
        if !path.exists() {
            return Ok(out);
        }
        for e in fs::read_dir(path)? {
            let e = e?;
            if e.file_type()?.is_dir() {
                if let Some(s) = e.file_name().to_str() {
                    out.push(s.to_string());
                }
            }
        }
        Ok(out)
    }

    fn exists(&self, path: &Path) -> bool {
        path.exists()
    }

    fn copy_dir_recursive(&self, from: &Path, to: &Path) -> Result<(), NveError> {
        // Helper opcional si tu trait lo define; si no, bórralo.
        if to.exists() {
            fs::remove_dir_all(to)?;
        }
        fs::create_dir_all(to)?;
        for entry in walkdir::WalkDir::new(from) {
            let entry = entry.map_err(|e| NveError::Io(std::io::Error::other(e)))?;
            let rel: PathBuf = entry.path().strip_prefix(from).unwrap().into();
            let dest = to.join(&rel);
            if entry.file_type().is_dir() {
                fs::create_dir_all(&dest)?;
            } else {
                if let Some(parent) = dest.parent() {
                    fs::create_dir_all(parent)?;
                }
                fs::copy(entry.path(), &dest)?;
            }
        }
        Ok(())
    }
}
