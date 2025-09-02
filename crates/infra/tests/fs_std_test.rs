use std::fs::{self, File};
use std::io::Write;
use std::path::Path;

use tempfile::TempDir;

use nve_core::ports::fs::FileSystem;
use nve_infra::fs_std::StdFs;

fn write_file(path: &Path, contents: &str) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    let mut f = File::create(path).unwrap();
    f.write_all(contents.as_bytes()).unwrap();
    f.flush().unwrap();
}

fn read_file(path: &Path) -> String {
    fs::read_to_string(path).unwrap()
}

#[test]
fn default_and_new_are_equivalent() {
    let _ = StdFs::new();
    let _ = StdFs;
}

#[test]
fn create_dir_all_creates_nested_directories() {
    let fs_std = StdFs::new();
    let tmp = TempDir::new().unwrap();
    let nested = tmp.path().join("a/b/c");

    assert!(!nested.exists());
    fs_std.create_dir_all(&nested).unwrap();
    assert!(nested.exists());
    assert!(nested.is_dir());
}

#[test]
fn remove_dir_all_removes_and_is_idempotent() {
    let fs_std = StdFs::new();
    let tmp = TempDir::new().unwrap();
    let dir = tmp.path().join("to_remove/deeper");
    fs::create_dir_all(&dir).unwrap();

    assert!(dir.exists());
    fs_std
        .remove_dir_all(&tmp.path().join("to_remove"))
        .unwrap();
    assert!(!tmp.path().join("to_remove").exists());

    fs_std
        .remove_dir_all(&tmp.path().join("does_not_exist"))
        .unwrap();
}

#[test]
fn read_dir_names_nonexistent_returns_empty() {
    let fs_std = StdFs::new();
    let tmp = TempDir::new().unwrap();
    let missing = tmp.path().join("missing");

    let v = fs_std.read_dir_names(&missing).unwrap();
    assert!(v.is_empty());
}

#[test]
fn read_dir_names_returns_only_directories() {
    let fs_std = StdFs::new();
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();

    fs::create_dir_all(root.join("alpha")).unwrap();
    fs::create_dir_all(root.join("bravo")).unwrap();
    write_file(&root.join("charlie.txt"), "file");
    write_file(&root.join("delta"), "file");

    let mut names = fs_std.read_dir_names(root).unwrap();
    names.sort();
    assert_eq!(names, vec!["alpha".to_string(), "bravo".to_string()]);
}

#[test]
fn exists_reports_correctly_for_dirs_and_files() {
    let fs_std = StdFs::new();
    let tmp = TempDir::new().unwrap();
    let dir = tmp.path().join("dir");
    let file = tmp.path().join("file.txt");

    assert!(!fs_std.exists(&dir));
    assert!(!fs_std.exists(&file));

    fs::create_dir_all(&dir).unwrap();
    write_file(&file, "hi");

    assert!(fs_std.exists(&dir));
    assert!(fs_std.exists(&file));
}

#[test]
fn copy_dir_recursive_copies_structure_and_overwrites_destination() {
    let fs_std = StdFs::new();
    let tmp = TempDir::new().unwrap();
    let src = tmp.path().join("src");
    let dst = tmp.path().join("dst");

    write_file(&src.join("root.txt"), "root");
    write_file(&src.join("nested/one.txt"), "one");
    write_file(&src.join("nested/deeper/two.txt"), "two");
    write_file(&dst.join("old/keep.txt"), "should be removed");

    fs_std.copy_dir_recursive(&src, &dst).unwrap();

    assert!(dst.join("root.txt").exists());
    assert!(dst.join("nested/one.txt").exists());
    assert!(dst.join("nested/deeper/two.txt").exists());
    assert_eq!(read_file(&dst.join("root.txt")), "root");
    assert_eq!(read_file(&dst.join("nested/one.txt")), "one");
    assert_eq!(read_file(&dst.join("nested/deeper/two.txt")), "two");

    assert!(!dst.join("old/keep.txt").exists());

    write_file(&dst.join("nested/one.txt"), "stale");
    fs_std.copy_dir_recursive(&src, &dst).unwrap();
    assert_eq!(read_file(&dst.join("nested/one.txt")), "one");
}

#[test]
fn copy_dir_recursive_returns_error_when_source_missing() {
    let fs_std = StdFs::new();
    let tmp = TempDir::new().unwrap();
    let missing_src = tmp.path().join("no_such_dir");
    let dst = tmp.path().join("dst");

    let res = fs_std.copy_dir_recursive(&missing_src, &dst);
    assert!(
        res.is_err(),
        "Debe fallar si el directorio origen no existe"
    );
}
