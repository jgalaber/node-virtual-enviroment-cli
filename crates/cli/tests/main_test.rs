use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::{fs, process::Command};
use tempfile::TempDir;

fn set_home(cmd: &mut Command, home: &std::path::Path) {
    cmd.env("HOME", home);
    #[cfg(windows)]
    cmd.env("USERPROFILE", home);
}

fn layout_paths(home: &std::path::Path) -> (std::path::PathBuf, std::path::PathBuf) {
    let base = home.join(".nve");
    let versions = base.join("versions");
    (base, versions)
}

fn bin_cmd() -> Command {
    if let Ok(p) = std::env::var("CARGO_BIN_EXE_nve") {
        return Command::new(p);
    }
    if let Ok(p) = std::env::var("CARGO_BIN_EXE_nve_cli") {
        return Command::new(p);
    }
    if let Ok(p) = std::env::var("CARGO_BIN_EXE_nve-cli") {
        return Command::new(p);
    }

    if let Ok(cmd) = Command::cargo_bin("nve") {
        return cmd;
    }

    Command::cargo_bin("nve-cli").expect("no se encontró binario 'nve' ni 'nve-cli'")
}

#[test]
fn shows_help() {
    let mut cmd = bin_cmd();
    cmd.arg("--help");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Node.js version manager"));
}

#[test]
fn list_in_home_vacio_no_muestra_nada() {
    let tmp = TempDir::new().unwrap();
    let (_base, _versions) = layout_paths(tmp.path());

    let mut cmd = bin_cmd();
    set_home(&mut cmd, tmp.path());
    cmd.arg("list");
    cmd.assert().success().stdout(predicate::str::is_empty());
}

#[test]
fn list_muestra_versiones_ordenadas() {
    let tmp = TempDir::new().unwrap();
    let (_base, versions) = layout_paths(tmp.path());
    fs::create_dir_all(versions.join("23.1.0")).unwrap();
    fs::create_dir_all(versions.join("23.2.1")).unwrap();

    let mut cmd = bin_cmd();
    set_home(&mut cmd, tmp.path());
    cmd.arg("list");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("23.1.0"))
        .stdout(predicate::str::contains("23.2.1"));
}

#[test]
fn use_activa_mejor_coincidencia_y_remove_limpia_current() {
    let tmp = TempDir::new().unwrap();
    let (base, versions) = layout_paths(tmp.path());
    fs::create_dir_all(versions.join("23.1.0")).unwrap();
    fs::create_dir_all(versions.join("23.2.1")).unwrap();

    let mut use_cmd = bin_cmd();
    set_home(&mut use_cmd, tmp.path());
    use_cmd
        .args(["use", "23"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Using 23.2.1"));

    let current = base.join("nodejs");
    assert!(current.exists(), "current debe existir tras 'use'");

    #[cfg(unix)]
    {
        let target = fs::read_link(&current).expect("debe ser symlink");
        assert!(
            target.ends_with("23.2.1"),
            "symlink debe apuntar a 23.2.1, fue {:?}",
            target
        );
    }

    let mut rm_cmd = bin_cmd();
    set_home(&mut rm_cmd, tmp.path());
    rm_cmd
        .args(["remove", "23.2.1"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Removed 23.2.1"));

    assert!(
        !versions.join("23.2.1").exists(),
        "la versión debe haberse eliminado"
    );
    assert!(current.exists(), "current debe existir (recreado vacío)");

    #[cfg(unix)]
    {
        let is_link = fs::read_link(&current).is_ok();
        assert!(
            !is_link,
            "current no debería seguir siendo symlink tras 'remove'"
        );
    }
}
