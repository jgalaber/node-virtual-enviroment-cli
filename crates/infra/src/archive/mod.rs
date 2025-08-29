#[cfg(unix)]
pub mod tar_xz;

#[cfg(windows)]
pub mod zip;

#[cfg(unix)]
pub use tar_xz::TarXzArchive;

#[cfg(windows)]
pub use zip::ZipArchive;
