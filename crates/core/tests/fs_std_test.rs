use nve_core::ports::fs::FileSystem;
use nve_infra::fs_std::StdFs;
use tempfile::TempDir;

#[test]
fn creates_and_lists_dirs() {
    let tmp = TempDir::new().unwrap();
    let fs_impl = StdFs;

    let root = tmp.path().join("versions");
    fs_impl.create_dir_all(&root).unwrap();
    fs_impl.create_dir_all(&root.join("20.10.0")).unwrap();
    fs_impl.create_dir_all(&root.join("18.19.1")).unwrap();

    let mut names = fs_impl.read_dir_names(&root).unwrap();
    names.sort();
    assert_eq!(names, vec!["18.19.1".to_string(), "20.10.0".to_string()]);
}
