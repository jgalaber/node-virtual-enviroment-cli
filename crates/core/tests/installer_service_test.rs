use std::sync::{
    atomic::{AtomicU32, Ordering},
    Mutex,
};

use tempfile::TempDir;

use nve_core::constants::NODEJS_API_BASE;
use nve_core::domain::version::ParsedVersion;
use nve_core::error::NveError;
use nve_core::ports::archive::Archive;
use nve_core::ports::fs::FileSystem;
use nve_core::ports::http::HttpClient;
use nve_core::ports::platform::Platform;
use nve_core::services::InstallService;
use nve_core::state::layout::NveLayout;

struct FakeHttpOk {
    last_url: Mutex<Option<String>>,
    bytes: Vec<u8>,
    get_json_calls: AtomicU32,
}
impl FakeHttpOk {
    fn new(bytes: Vec<u8>) -> Self {
        Self {
            last_url: Mutex::new(None),
            bytes,
            get_json_calls: AtomicU32::new(0),
        }
    }
}

#[async_trait::async_trait]
impl HttpClient for FakeHttpOk {
    async fn get_bytes(&self, url: &str) -> Result<Vec<u8>, NveError> {
        *self.last_url.lock().unwrap() = Some(url.to_string());
        Ok(self.bytes.clone())
    }

    async fn get_json<T: serde::de::DeserializeOwned + Send>(
        &self,
        _url: &str,
    ) -> Result<T, NveError> {
        self.get_json_calls.fetch_add(1, Ordering::SeqCst);
        let data = serde_json::json!([
          { "version": "v23.4.0", "lts": false, "date": "2024-02-01", "files": [], "security": false },
          { "version": "v23.2.1", "lts": false, "date": "2024-01-01", "files": [], "security": false },
          { "version": "v23.1.0", "lts": false, "date": "2023-12-01", "files": [], "security": false }
        ]);
        serde_json::from_value(data).map_err(|e| NveError::ExtractError(e.to_string()))
    }
}

struct FakeHttpErr;
#[async_trait::async_trait]
impl HttpClient for FakeHttpErr {
    async fn get_bytes(&self, _url: &str) -> Result<Vec<u8>, NveError> {
        Err(NveError::ExtractError("download failed".into()))
    }
    async fn get_json<T: serde::de::DeserializeOwned + Send>(
        &self,
        _url: &str,
    ) -> Result<T, NveError> {
        Err(NveError::ExtractError("unexpected".into()))
    }
}

struct FakePlatform;
#[async_trait::async_trait]
impl Platform for FakePlatform {
    fn os_arch(&self) -> (String, String) {
        ("win".to_string(), "x64".to_string())
    }
    fn archive_name(&self, version: &str) -> String {
        format!("node-v{version}-win-x64.zip")
    }
    async fn set_current(
        &self,
        _version_dir: &std::path::Path,
        _current_dir: &std::path::Path,
    ) -> Result<(), NveError> {
        Ok(())
    }
    async fn is_current(
        &self,
        _version: &str,
        _current_dir: &std::path::Path,
    ) -> Result<bool, NveError> {
        Ok(false)
    }
}

struct FakeFs;
impl FileSystem for FakeFs {
    fn create_dir_all(&self, path: &std::path::Path) -> Result<(), NveError> {
        std::fs::create_dir_all(path)?;
        Ok(())
    }
    fn remove_dir_all(&self, path: &std::path::Path) -> Result<(), NveError> {
        if path.exists() {
            std::fs::remove_dir_all(path)?;
        }
        Ok(())
    }
    fn read_dir_names(&self, path: &std::path::Path) -> Result<Vec<String>, NveError> {
        let mut out = vec![];
        if !path.exists() {
            return Ok(out);
        }
        for e in std::fs::read_dir(path)? {
            let e = e?;
            if e.file_type()?.is_dir() {
                if let Some(s) = e.file_name().to_str() {
                    out.push(s.to_string());
                }
            }
        }
        Ok(out)
    }
    fn exists(&self, path: &std::path::Path) -> bool {
        path.exists()
    }
    fn copy_dir_recursive(
        &self,
        _from: &std::path::Path,
        _to: &std::path::Path,
    ) -> Result<(), NveError> {
        unreachable!("not used in these tests")
    }
}

struct FakeArchiveOk {
    calls: AtomicU32,
}
impl FakeArchiveOk {
    fn new() -> Self {
        Self {
            calls: AtomicU32::new(0),
        }
    }
}
#[async_trait::async_trait]
impl Archive for FakeArchiveOk {
    async fn extract(
        &self,
        data: &[u8],
        target_dir: &std::path::Path,
        _version: &str,
    ) -> Result<(), NveError> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        std::fs::create_dir_all(target_dir)?;
        std::fs::write(target_dir.join("EXTRACTED.ok"), data)?;
        Ok(())
    }
}

struct FakeArchiveErr;
#[async_trait::async_trait]
impl Archive for FakeArchiveErr {
    async fn extract(
        &self,
        _data: &[u8],
        _target_dir: &std::path::Path,
        _version: &str,
    ) -> Result<(), NveError> {
        Err(NveError::ExtractError("extract failed".into()))
    }
}

fn read_bytes(p: impl AsRef<std::path::Path>) -> Vec<u8> {
    std::fs::read(p).unwrap()
}

#[tokio::test]
async fn install_returns_early_when_version_exists() {
    let tmp = TempDir::new().unwrap();
    let layout = NveLayout {
        base: tmp.path().to_path_buf(),
    };

    let spec = ParsedVersion::parse("23.1.0").expect("ParsedVersion exact");
    let exact = "23.1.0";
    let version_dir = layout.version_dir(exact);
    std::fs::create_dir_all(&version_dir).unwrap();

    let http = FakeHttpOk::new(b"ZIPDATA".to_vec());
    let fs = FakeFs;
    let plat = FakePlatform;
    let arch = FakeArchiveOk::new();

    let svc = InstallService {
        http: &http,
        fs: &fs,
        plat: &plat,
        arch: &arch,
        layout: &layout,
    };
    let out = svc.install(&spec).await.expect("install ok");

    assert_eq!(out, exact);
    assert_eq!(arch.calls.load(Ordering::SeqCst), 0);
    assert!(http.last_url.lock().unwrap().is_none());
}

#[tokio::test]
async fn install_downloads_builds_correct_url_and_extracts() {
    let tmp = TempDir::new().unwrap();
    let layout = NveLayout {
        base: tmp.path().to_path_buf(),
    };

    let spec = ParsedVersion::parse("23.2.1").expect("ParsedVersion exact");
    let exact = "23.2.1";

    let http = FakeHttpOk::new(b"ZIPDATA".to_vec());
    let fs = FakeFs;
    let plat = FakePlatform;
    let arch = FakeArchiveOk::new();

    let svc = InstallService {
        http: &http,
        fs: &fs,
        plat: &plat,
        arch: &arch,
        layout: &layout,
    };
    let out = svc.install(&spec).await.expect("install ok");
    assert_eq!(out, exact);

    let expected_name = plat.archive_name(exact);
    let expected_url = format!("{}/v{}/{}", NODEJS_API_BASE, exact, expected_name);
    assert_eq!(
        http.last_url.lock().unwrap().as_deref(),
        Some(expected_url.as_str())
    );

    let version_dir = layout.version_dir(exact);
    assert!(version_dir.join("EXTRACTED.ok").exists());
    assert_eq!(
        read_bytes(version_dir.join("EXTRACTED.ok")),
        b"ZIPDATA".to_vec()
    );
}

#[tokio::test]
async fn install_propagates_http_error() {
    let tmp = TempDir::new().unwrap();
    let layout = NveLayout {
        base: tmp.path().to_path_buf(),
    };

    let spec = ParsedVersion::parse("23.0.0").expect("ParsedVersion exact");

    let http = FakeHttpErr;
    let fs = FakeFs;
    let plat = FakePlatform;
    let arch = FakeArchiveOk::new();

    let svc = InstallService {
        http: &http,
        fs: &fs,
        plat: &plat,
        arch: &arch,
        layout: &layout,
    };
    let _err = svc.install(&spec).await.unwrap_err();
}

#[tokio::test]
async fn install_propagates_extract_error() {
    let tmp = TempDir::new().unwrap();
    let layout = NveLayout {
        base: tmp.path().to_path_buf(),
    };

    let spec = ParsedVersion::parse("23.4.0").expect("ParsedVersion exact");

    let http = FakeHttpOk::new(b"ZIPDATA".to_vec());
    let fs = FakeFs;
    let plat = FakePlatform;
    let arch = FakeArchiveErr;

    let svc = InstallService {
        http: &http,
        fs: &fs,
        plat: &plat,
        arch: &arch,
        layout: &layout,
    };
    let _err = svc.install(&spec).await.unwrap_err();
}
