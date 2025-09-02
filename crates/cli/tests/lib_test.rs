use async_trait::async_trait;
use std::sync::{
    atomic::{AtomicU32, Ordering},
    Mutex,
};
use tempfile::TempDir;

use nve_cli::{
    cmd_install, cmd_list, cmd_remote, cmd_remove, cmd_use, parse_spec, resolve_installed,
};
use nve_core::ports::{archive::Archive, fs::FileSystem, http::HttpClient, platform::Platform};
use nve_core::{
    constants::NODEJS_API_BASE, domain::version::ParsedVersion, error::NveError,
    state::layout::NveLayout,
};

// ----------------- FAKES THREAD-SAFE -----------------

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
#[async_trait]
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
        // Índice mínimo compatible con ResolveService
        let data = serde_json::json!([
          {"version":"v23.2.1","lts":false,"date":"2024-01-01","files":[],"security":false},
          {"version":"v23.1.0","lts":false,"date":"2023-12-01","files":[],"security":false}
        ]);
        serde_json::from_value(data).map_err(|e| NveError::ExtractError(e.to_string()))
    }
}

struct FakeHttpErr;
#[async_trait]
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

struct FakeFs;
impl FileSystem for FakeFs {
    fn create_dir_all(&self, p: &std::path::Path) -> Result<(), NveError> {
        std::fs::create_dir_all(p)?;
        Ok(())
    }
    fn remove_dir_all(&self, p: &std::path::Path) -> Result<(), NveError> {
        if p.exists() {
            std::fs::remove_dir_all(p)?;
        }
        Ok(())
    }
    fn read_dir_names(&self, p: &std::path::Path) -> Result<Vec<String>, NveError> {
        let mut out = vec![];
        if !p.exists() {
            return Ok(out);
        }
        for e in std::fs::read_dir(p)? {
            let e = e?;
            if e.file_type()?.is_dir() {
                if let Some(s) = e.file_name().to_str() {
                    out.push(s.to_string());
                }
            }
        }
        Ok(out)
    }
    fn exists(&self, p: &std::path::Path) -> bool {
        p.exists()
    }
    fn copy_dir_recursive(
        &self,
        _from: &std::path::Path,
        _to: &std::path::Path,
    ) -> Result<(), NveError> {
        unreachable!()
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
#[async_trait]
impl Archive for FakeArchiveOk {
    async fn extract(
        &self,
        data: &[u8],
        target_dir: &std::path::Path,
        _v: &str,
    ) -> Result<(), NveError> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        std::fs::create_dir_all(target_dir)?;
        std::fs::write(target_dir.join("EXTRACTED.ok"), data)?;
        Ok(())
    }
}
#[allow(dead_code)]
struct FakeArchiveErr;
#[async_trait]
impl Archive for FakeArchiveErr {
    async fn extract(
        &self,
        _data: &[u8],
        _target_dir: &std::path::Path,
        _v: &str,
    ) -> Result<(), NveError> {
        Err(NveError::ExtractError("extract failed".into()))
    }
}

struct FakePlatform {
    is_current_flag: Mutex<bool>,
    set_current_calls: AtomicU32,
}
impl FakePlatform {
    fn new() -> Self {
        Self {
            is_current_flag: Mutex::new(false),
            set_current_calls: AtomicU32::new(0),
        }
    }
}
#[async_trait]
impl Platform for FakePlatform {
    fn os_arch(&self) -> (String, String) {
        ("win".into(), "x64".into())
    }
    fn archive_name(&self, v: &str) -> String {
        format!("node-v{v}-win-x64.zip")
    }
    async fn set_current(
        &self,
        _version_dir: &std::path::Path,
        current_dir: &std::path::Path,
    ) -> Result<(), NveError> {
        self.set_current_calls.fetch_add(1, Ordering::SeqCst);
        if !current_dir.exists() {
            std::fs::create_dir_all(current_dir)?;
        }
        Ok(())
    }
    async fn is_current(
        &self,
        _version: &str,
        _current_dir: &std::path::Path,
    ) -> Result<bool, NveError> {
        Ok(*self.is_current_flag.lock().unwrap())
    }
}

// ----------------- HELPERS -----------------
fn mk_layout() -> (TempDir, NveLayout) {
    let tmp = TempDir::new().unwrap();
    let layout = NveLayout {
        base: tmp.path().to_path_buf(),
    };
    (tmp, layout)
}

// ----------------- TESTS -----------------

#[test]
fn parse_spec_ok_y_err() {
    let ok = parse_spec("23.2.1").unwrap();
    assert_eq!(ok.major, 23);
    assert_eq!(ok.minor, Some(2));
    assert_eq!(ok.patch, Some(1));
    assert!(parse_spec("foo").is_err());
    assert!(parse_spec("1.2.3.4").is_err());
}

#[test]
fn resolve_installed_cases() {
    let installed = vec![
        "23.1.0".into(),
        "23.2.0".into(),
        "23.2.1".into(),
        "22.9.9".into(),
    ];
    // Mejor de 23.x
    let spec = ParsedVersion::parse("23").unwrap();
    assert_eq!(
        resolve_installed(&installed, &spec).as_deref(),
        Some("23.2.1")
    );
    // Mejor de 23.2.x
    let spec_minor = ParsedVersion::parse("23.2").unwrap();
    assert_eq!(
        resolve_installed(&installed, &spec_minor).as_deref(),
        Some("23.2.1")
    );
    // Exacta
    let spec_exact = ParsedVersion::parse("22.9.9").unwrap();
    assert_eq!(
        resolve_installed(&installed, &spec_exact).as_deref(),
        Some("22.9.9")
    );
    // Sin match
    let spec_none = ParsedVersion::parse("24").unwrap();
    assert!(resolve_installed(&installed, &spec_none).is_none());
}

#[tokio::test]
async fn cmd_list_vacio_y_con_contenido() {
    let (_tmp, layout) = mk_layout();
    let fs = FakeFs;

    // vacío
    cmd_list(&fs, &layout).await.unwrap();

    // con versiones (ejecuta rama exists=true + sort + println)
    std::fs::create_dir_all(layout.versions_dir().join("23.1.0")).unwrap();
    std::fs::create_dir_all(layout.versions_dir().join("23.2.1")).unwrap();
    cmd_list(&fs, &layout).await.unwrap();
}

#[tokio::test]
async fn cmd_install_happy_path() {
    let (_tmp, layout) = mk_layout();
    let http = FakeHttpOk::new(b"ZIPDATA".to_vec());
    let fs = FakeFs;
    let plat = FakePlatform::new();
    let arch = FakeArchiveOk::new();

    cmd_install(&http, &fs, &plat, &arch, &layout, "23.2.1")
        .await
        .unwrap();

    let vdir = layout.version_dir("23.2.1");
    assert!(vdir.join("EXTRACTED.ok").exists());
    let expected = format!(
        "{}/v{}/{}",
        NODEJS_API_BASE,
        "23.2.1",
        plat.archive_name("23.2.1")
    );
    assert_eq!(
        http.last_url.lock().unwrap().as_deref(),
        Some(expected.as_str())
    );
}

#[tokio::test]
async fn cmd_install_falla_si_descarga_falla() {
    let (_tmp, layout) = mk_layout();
    let http = FakeHttpErr;
    let fs = FakeFs;
    let plat = FakePlatform::new();
    let arch = FakeArchiveOk::new();

    let _err = cmd_install(&http, &fs, &plat, &arch, &layout, "23.2.1")
        .await
        .unwrap_err();
}

#[tokio::test]
async fn cmd_use_activa_mejor_version() {
    let (_tmp, layout) = mk_layout();
    let fs = FakeFs;
    let plat = FakePlatform::new();

    std::fs::create_dir_all(layout.versions_dir().join("23.1.0")).unwrap();
    std::fs::create_dir_all(layout.versions_dir().join("23.2.1")).unwrap();

    cmd_use(&fs, &plat, &layout, "23").await.unwrap();
    assert!(layout.current_dir().exists());
}

#[tokio::test]
async fn cmd_use_error_si_no_instalada() {
    let (_tmp, layout) = mk_layout();
    let fs = FakeFs;
    let plat = FakePlatform::new();

    // no hay versiones instaladas
    let err = cmd_use(&fs, &plat, &layout, "99.0.0").await.unwrap_err();
    let _ = err; // basta con que devuelva Err (VersionNotInstalled)
}

#[tokio::test]
async fn cmd_remove_borra_version_y_resetea_current_si_estaba_activa() {
    let (_tmp, layout) = mk_layout();
    let fs = FakeFs;
    let plat = FakePlatform::new();

    let v = "23.1.0";
    let vdir = layout.version_dir(v);
    std::fs::create_dir_all(&vdir).unwrap();
    std::fs::create_dir_all(layout.current_dir()).unwrap();

    // Fuerza rama is_current=true
    *plat.is_current_flag.lock().unwrap() = true;

    cmd_remove(&fs, &plat, &layout, v).await.unwrap();

    assert!(!vdir.exists());
    assert!(layout.current_dir().exists()); // recreado vacío
}

#[tokio::test]
async fn cmd_remove_borra_version_sin_tocar_current_si_no_estaba_activa() {
    let (_tmp, layout) = mk_layout();
    let fs = FakeFs;
    let plat = FakePlatform::new();

    let v = "23.2.1";
    let vdir = layout.version_dir(v);
    std::fs::create_dir_all(&vdir).unwrap();

    // is_current=false (por defecto), y current no existe
    cmd_remove(&fs, &plat, &layout, v).await.unwrap();

    assert!(!vdir.exists());
    assert!(!layout.current_dir().exists());
}

#[tokio::test]
async fn cmd_remove_error_si_no_instalada() {
    let (_tmp, layout) = mk_layout();
    let fs = FakeFs;
    let plat = FakePlatform::new();

    // no hay versiones ⇒ resolve_installed = None
    let err = cmd_remove(&fs, &plat, &layout, "77.7.7").await.unwrap_err();
    let _ = err;
}

#[tokio::test]
async fn cmd_remote_resuelve_y_muestra() {
    let http = FakeHttpOk::new(b"IGNORED".to_vec());
    cmd_remote(&http, "23").await.unwrap();
}
