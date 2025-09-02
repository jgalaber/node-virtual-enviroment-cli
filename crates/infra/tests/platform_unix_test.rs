#![cfg(unix)]

use std::fs::{self, File};
use std::io::Write;
use std::path::Path;

use tempfile::TempDir;

use nve_core::ports::platform::Platform;
use nve_infra::platform::UnixPlatform;

fn write_file(path: &Path, contents: &str) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    let mut f = File::create(path).unwrap();
    f.write_all(contents.as_bytes()).unwrap();
    f.flush().unwrap();
}

#[test]
fn os_arch_coincide_con_target_y_valores_validos() {
    let p = UnixPlatform::new().unwrap();
    let (os, arch) = p.os_arch();

    #[cfg(target_os = "macos")]
    let expected_os = "darwin";
    #[cfg(not(target_os = "macos"))]
    let expected_os = "linux";

    #[cfg(target_arch = "x86_64")]
    let expected_arch = "x64";
    #[cfg(target_arch = "aarch64")]
    let expected_arch = "arm64";
    #[cfg(all(not(target_arch = "x86_64"), not(target_arch = "aarch64")))]
    let expected_arch = "x64";

    assert_eq!(os, expected_os);
    assert_eq!(arch, expected_arch);

    assert!(os == "linux" || os == "darwin");
    assert!(arch == "x64" || arch == "arm64");
}

#[test]
fn archive_name_formatea_con_version_y_plataforma() {
    let p = UnixPlatform::new().unwrap();
    let (os, arch) = p.os_arch();

    let v = "23.1.0";
    let name = p.archive_name(v);

    let expected = format!("node-v{}-{}-{}.tar.xz", v, os, arch);
    assert_eq!(name, expected);
}

#[tokio::test]
async fn set_current_crea_symlink_y_is_current_true() {
    let p = UnixPlatform::new().unwrap();
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();

    let versions = root.join("versions");
    let current = root.join("current");
    let vdir = versions.join("node-v23.0.0");

    fs::create_dir_all(&vdir).unwrap();

    assert!(!current.exists());

    p.set_current(&vdir, &current).await.unwrap();

    let link_target = fs::read_link(&current).unwrap();
    assert_eq!(link_target, vdir);
    assert!(p.is_current("node-v23.0.0", &current).await.unwrap());
    assert!(!p.is_current("node-v22.9.0", &current).await.unwrap());
}

#[tokio::test]
async fn set_current_sobrescribe_si_current_existe_como_fichero_o_directorio() {
    let p = UnixPlatform::new().unwrap();
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();

    let versions = root.join("versions");
    let current = root.join("current");
    let v1 = versions.join("node-v23.1.0");
    let v2 = versions.join("node-v23.2.0");

    fs::create_dir_all(&v1).unwrap();
    fs::create_dir_all(&v2).unwrap();

    write_file(&current, "no debería quedar");
    assert!(current.exists() && current.is_file());

    p.set_current(&v1, &current).await.unwrap();
    let link1 = fs::read_link(&current).unwrap();
    assert_eq!(link1, v1);
    assert!(p.is_current("node-v23.1.0", &current).await.unwrap());

    fs::remove_file(&current).unwrap();
    fs::create_dir_all(&current).unwrap();
    assert!(current.is_dir());

    p.set_current(&v2, &current).await.unwrap();
    let link2 = fs::read_link(&current).unwrap();
    assert_eq!(link2, v2);
    assert!(p.is_current("node-v23.2.0", &current).await.unwrap());
    assert!(!p.is_current("node-v23.1.0", &current).await.unwrap());
}

#[tokio::test]
async fn is_current_false_si_no_hay_symlink() {
    let p = UnixPlatform::new().unwrap();
    let tmp = TempDir::new().unwrap();
    let current = tmp.path().join("current");

    assert!(!current.exists());
    let res = p.is_current("node-v99.0.0", &current).await.unwrap();
    assert!(!res);
}
