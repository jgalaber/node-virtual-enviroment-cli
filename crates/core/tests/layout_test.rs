use std::path::PathBuf;
use tempfile::TempDir;

use nve_core::constants::{NODEJS_DIR, VERSION_DIR};
use nve_core::state::layout::NveLayout;

#[test]
fn versions_dir_concatena_base_y_constante() {
    let tmp = TempDir::new().unwrap();
    let base = tmp.path().to_path_buf();
    let layout = NveLayout { base: base.clone() };

    let expected = base.join(VERSION_DIR);
    assert_eq!(layout.versions_dir(), expected);

    assert_eq!(layout.base, base);
}

#[test]
fn current_dir_apunta_a_nodejs_dir_bajo_base() {
    let tmp = TempDir::new().unwrap();
    let base = tmp.path().to_path_buf();
    let layout = NveLayout { base: base.clone() };

    let expected = base.join(NODEJS_DIR);
    assert_eq!(layout.current_dir(), expected);
}

#[test]
fn version_dir_anexa_la_version_sobre_versions_dir() {
    let tmp = TempDir::new().unwrap();
    let base = tmp.path().to_path_buf();
    let layout = NveLayout { base: base.clone() };

    let v = "node-v23.1.0-win-x64";
    let expected = base.join(VERSION_DIR).join(v);

    assert_eq!(layout.version_dir(v), expected);
    assert_eq!(layout.version_dir(v), layout.versions_dir().join(v));
}

#[test]
fn rutas_son_portables_pathbuf_y_no_dependen_de_separadores() {
    let base = PathBuf::from("some/base");
    let layout = NveLayout { base: base.clone() };

    let p1 = layout.versions_dir();
    let p2 = base.join(VERSION_DIR);
    assert_eq!(p1, p2);

    let p3 = layout.current_dir();
    let p4 = base.join(NODEJS_DIR);
    assert_eq!(p3, p4);
}
