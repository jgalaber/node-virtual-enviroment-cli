#[cfg(unix)]
pub mod unix;

#[cfg(windows)]
pub mod windows;

#[cfg(unix)]
pub use unix::UnixPlatform;

#[cfg(windows)]
pub use windows::WindowsPlatform;
