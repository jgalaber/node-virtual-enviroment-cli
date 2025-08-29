use std::path::Path;

use crate::error::NveError;

pub trait FileSystem: Send + Sync {
    fn create_dir_all(&self, path: &Path) -> Result<(), NveError>;
    fn remove_dir_all(&self, path: &Path) -> Result<(), NveError>;
    fn read_dir_names(&self, path: &Path) -> Result<Vec<String>, NveError>;
    fn exists(&self, path: &Path) -> bool;
    fn copy_dir_recursive(&self, from: &Path, to: &Path) -> Result<(), NveError>;
}
