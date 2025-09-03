use std::{env, path::PathBuf, sync::Mutex};

use tempfile::TempDir;

use nve_core::constants::{NODEJS_DIR, VERSION_DIR};
use nve_core::state::layout::NveLayout;

static ENV_LOCK: Mutex<()> = Mutex::new(());

struct EnvGuard {
    saved: Vec<(String, Option<std::ffi::OsString>)>,
}

impl EnvGuard {
    fn new() -> Self {
        Self { saved: Vec::new() }
    }

    fn set<K, V>(&mut self, k: K, v: V)
    where
        K: Into<String>,
        V: AsRef<std::ffi::OsStr>,
    {
        let k = k.into();
        let prev = env::var_os(&k);
        self.saved.push((k.clone(), prev));

        env::set_var(&k, v.as_ref());
    }

    fn unset<K: Into<String>>(&mut self, k: K) {
        let k = k.into();
        let prev = env::var_os(&k);
        self.saved.push((k.clone(), prev));
        env::remove_var(k);
    }
}

impl Drop for EnvGuard {
    fn drop(&mut self) {
        for (k, v) in self.saved.drain(..) {
            match v {
                Some(val) => env::set_var(k, val),
                None => env::remove_var(k),
            }
        }
    }
}

// ---------- Path derivation tests (pure, no env usage) ----------

#[test]
fn versions_dir_joins_base_and_constant() {
    let tmp = TempDir::new().unwrap();
    let base = tmp.path().to_path_buf();
    let layout = NveLayout { base: base.clone() };

    let expected = base.join(VERSION_DIR);
    assert_eq!(layout.versions_dir(), expected);
    assert_eq!(layout.base, base);
}

#[test]
fn current_dir_points_to_nodejs_dir_under_base() {
    let tmp = TempDir::new().unwrap();
    let base = tmp.path().to_path_buf();
    let layout = NveLayout { base: base.clone() };

    let expected = base.join(NODEJS_DIR);
    assert_eq!(layout.current_dir(), expected);
}

#[test]
fn version_dir_appends_version_under_versions_dir() {
    let tmp = TempDir::new().unwrap();
    let base = tmp.path().to_path_buf();
    let layout = NveLayout { base: base.clone() };

    let v = "node-v23.1.0-win-x64";
    let expected = base.join(VERSION_DIR).join(v);

    assert_eq!(layout.version_dir(v), expected);
    assert_eq!(layout.version_dir(v), layout.versions_dir().join(v));
}

#[test]
fn paths_are_portable_pathbuf_joins_not_string_concat() {
    let base = PathBuf::from("some/base");
    let layout = NveLayout { base: base.clone() };

    assert_eq!(layout.versions_dir(), base.join(VERSION_DIR));
    assert_eq!(layout.current_dir(), base.join(NODEJS_DIR));
}

// ---------- from_env_or_home tests (mutate env; serialized with lock) ----------

#[test]
fn from_env_or_home_uses_nve_base_when_defined() {
    let _lock = ENV_LOCK.lock().unwrap();
    let mut g = EnvGuard::new();

    let tmp = TempDir::new().unwrap();
    let forced_base = tmp.path().join("custom_base");
    g.set("NVE_BASE", &forced_base);

    let layout = NveLayout::from_env_or_home().expect("layout");
    assert_eq!(layout.base, forced_base);
    // When NVE_BASE is set, do NOT auto-append ".nve"
    assert!(!layout.base.ends_with(".nve"));
}

#[cfg(not(windows))]
#[test]
fn from_env_or_home_fallbacks_to_home_on_unix() {
    let _lock = ENV_LOCK.lock().unwrap();
    let mut g = EnvGuard::new();

    let tmp = TempDir::new().unwrap();
    g.unset("NVE_BASE");
    g.set("HOME", tmp.path());

    let layout = NveLayout::from_env_or_home().expect("layout");
    assert_eq!(layout.base, tmp.path().join(".nve"));
}

#[cfg(windows)]
#[test]
fn from_env_or_home_fallbacks_to_userprofile_on_windows() {
    let _lock = ENV_LOCK.lock().unwrap();
    let mut g = EnvGuard::new();

    let tmp = TempDir::new().unwrap();
    g.unset("NVE_BASE");
    g.set("USERPROFILE", tmp.path());
    g.unset("HOMEDRIVE");
    g.unset("HOMEPATH");

    let layout = NveLayout::from_env_or_home().expect("layout");
    assert_eq!(layout.base, tmp.path().join(".nve"));
}

#[cfg(windows)]
#[test]
fn from_env_or_home_fallbacks_to_homedrive_homepath_on_windows() {
    let _lock = ENV_LOCK.lock().unwrap();
    let mut g = EnvGuard::new();

    g.unset("NVE_BASE");
    g.unset("USERPROFILE");
    // Synthetic home C:\TempHome (does not need to exist for the path test)
    g.set("HOMEDRIVE", "C:");
    g.set("HOMEPATH", r"\TempHome");

    let layout = NveLayout::from_env_or_home().expect("layout");
    assert_eq!(layout.base, PathBuf::from(r"C:\TempHome").join(".nve"));
}
