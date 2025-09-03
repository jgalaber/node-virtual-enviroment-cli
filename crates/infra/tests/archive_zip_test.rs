use std::fs;
use std::io::Write;
use std::path::Path;

use tempfile::TempDir;

use nve_core::ports::archive::Archive;
use nve_infra::archive::zip::ZipArchive;

fn build_zip_bytes_with_root(root: &str) -> Vec<u8> {
    use std::io::Cursor;
    use zip::write::SimpleFileOptions as FileOptions;
    use zip::{CompressionMethod, ZipWriter};

    let cursor = Cursor::new(Vec::<u8>::new());
    let mut w = ZipWriter::new(cursor);

    let opts_dir = FileOptions::default().compression_method(CompressionMethod::Stored);
    let opts_deflated = FileOptions::default().compression_method(CompressionMethod::Deflated);

    w.add_directory(format!("{root}/"), opts_dir).unwrap();
    w.add_directory(format!("{root}/bin/"), opts_dir).unwrap();

    w.start_file(format!("{root}/bin/node.exe"), opts_deflated)
        .unwrap();
    w.write_all(b"exe").unwrap();

    w.start_file(format!("{root}/README.md"), opts_deflated)
        .unwrap();
    w.write_all(b"# demo\n").unwrap();

    let cursor = w.finish().unwrap();
    cursor.into_inner()
}

fn build_zip_with_root_file_and_nested(root: &str) -> Vec<u8> {
    use std::io::Cursor;
    use zip::write::SimpleFileOptions as FileOptions;
    use zip::{CompressionMethod, ZipWriter};

    let cursor = Cursor::new(Vec::<u8>::new());
    let mut w = ZipWriter::new(cursor);

    let opts_dir = FileOptions::default().compression_method(CompressionMethod::Stored);
    let opts_deflated = FileOptions::default().compression_method(CompressionMethod::Deflated);

    w.start_file("root.txt", opts_deflated).unwrap();
    w.write_all(b"ignored").unwrap();

    w.add_directory(format!("{root}/"), opts_dir).unwrap();
    w.add_directory(format!("{root}/bin/"), opts_dir).unwrap();
    w.start_file(format!("{root}/bin/node.exe"), opts_deflated)
        .unwrap();
    w.write_all(b"exe").unwrap();

    let cursor = w.finish().unwrap();
    cursor.into_inner()
}

fn build_zip_only_root_dir(root: &str) -> Vec<u8> {
    use std::io::Cursor;
    use zip::write::SimpleFileOptions as FileOptions;
    use zip::{CompressionMethod, ZipWriter};

    let cursor = Cursor::new(Vec::<u8>::new());
    let mut w = ZipWriter::new(cursor);
    let opts_dir = FileOptions::default().compression_method(CompressionMethod::Stored);
    w.add_directory(format!("{root}/"), opts_dir).unwrap();
    let cursor = w.finish().unwrap();
    cursor.into_inner()
}

fn build_empty_zip() -> Vec<u8> {
    use std::io::Cursor;
    use zip::ZipWriter;

    let cursor = Cursor::new(Vec::<u8>::new());
    let w = ZipWriter::new(cursor);
    let cursor = w.finish().unwrap();
    cursor.into_inner()
}

fn read_string(p: &Path) -> String {
    fs::read_to_string(p).unwrap()
}

#[tokio::test]
async fn extract_aplana_y_preserva_contenido() {
    let tmp = TempDir::new().unwrap();
    let dest = tmp.path().join("dest");
    fs::create_dir_all(&dest).unwrap();

    let root = "node-v23.11.1-win-x64";
    let zip_bytes = build_zip_bytes_with_root(root);

    let arch = ZipArchive::new().unwrap();
    arch.extract(&zip_bytes, &dest, "23.11.1").await.unwrap();

    assert!(!dest.join(root).exists());
    assert!(dest.join("bin/node.exe").exists());
    assert_eq!(read_string(&dest.join("README.md")), "# demo\n");
}

#[tokio::test]
async fn extract_sobrescribe_si_existe_archivo() {
    let tmp = TempDir::new().unwrap();
    let dest = tmp.path().join("dest");
    fs::create_dir_all(dest.join("bin")).unwrap();

    fs::write(dest.join("bin/node.exe"), "old").unwrap();

    let zip_bytes = build_zip_bytes_with_root("node-vX-win-x64");
    let arch = ZipArchive::new().unwrap();
    arch.extract(&zip_bytes, &dest, "X").await.unwrap();

    assert_eq!(read_string(&dest.join("bin/node.exe")), "exe");
    assert_eq!(read_string(&dest.join("README.md")), "# demo\n");
}

#[tokio::test]
async fn extract_falla_con_zip_vacio() {
    let tmp = TempDir::new().unwrap();
    let dest = tmp.path().join("dest");
    fs::create_dir_all(&dest).unwrap();

    let empty_zip = build_empty_zip();
    let arch = ZipArchive::new().unwrap();
    let res = arch.extract(&empty_zip, &dest, "X").await;
    assert!(res.is_err(), "Debe fallar con 'empty archive'");
}

#[tokio::test]
async fn extract_falla_con_zip_solo_con_carpeta_raiz() {
    let tmp = TempDir::new().unwrap();
    let dest = tmp.path().join("dest");
    fs::create_dir_all(&dest).unwrap();

    let zip_bytes = build_zip_only_root_dir("node-vY-win-x64");
    let arch = ZipArchive::new().unwrap();
    let res = arch.extract(&zip_bytes, &dest, "Y").await;
    assert!(
        res.is_err(),
        "Debe fallar si el ZIP solo contiene la carpeta raíz"
    );
}

#[tokio::test]
async fn extract_ignora_archivos_en_raiz_del_zip_pero_no_falla_si_hay_contenido_util() {
    let tmp = TempDir::new().unwrap();
    let dest = tmp.path().join("dest");
    fs::create_dir_all(&dest).unwrap();

    let root = "node-vZ-win-x64";
    let zip_bytes = build_zip_with_root_file_and_nested(root);

    let arch = ZipArchive::new().unwrap();
    arch.extract(&zip_bytes, &dest, "Z").await.unwrap();

    assert!(!dest.join("root.txt").exists());
    assert!(dest.join("bin/node.exe").exists());
}

#[tokio::test]
async fn extract_falla_si_el_parent_en_destino_es_un_fichero() {
    let tmp = TempDir::new().unwrap();
    let dest = tmp.path().join("dest");
    fs::create_dir_all(&dest).unwrap();

    fs::write(dest.join("bin"), "i am a file").unwrap();

    let zip_bytes = build_zip_bytes_with_root("node-vK-win-x64");
    let arch = ZipArchive::new().unwrap();
    let res = arch.extract(&zip_bytes, &dest, "K").await;

    assert!(
        res.is_err(),
        "Debe fallar porque el parent 'bin' no es un directorio"
    );
}

#[tokio::test]
async fn extract_falla_con_zip_corrupto() {
    let tmp = TempDir::new().unwrap();
    let dest = tmp.path().join("dest");
    fs::create_dir_all(&dest).unwrap();

    let corrupt = b"\x00\x01not-zip\xff\xff".to_vec();

    let arch = ZipArchive::new().unwrap();
    let res = arch.extract(&corrupt, &dest, "X").await;
    assert!(res.is_err(), "Debe fallar al abrir ZIP corrupto");
}
