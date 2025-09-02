use std::fs;
use std::path::Path;

use tar::{Builder, Header};
use tempfile::TempDir;

use nve_core::ports::archive::Archive;
use nve_infra::archive::tar_xz::TarXzArchive;
use xz2::write::XzEncoder;

fn build_tar_xz_bytes(root_dir: &str) -> Vec<u8> {
    let mut out = Vec::new();
    {
        let enc = XzEncoder::new(&mut out, 6);
        let mut tar = Builder::new(enc);

        // Directorio raíz
        let mut dir_hdr = Header::new_gnu();
        dir_hdr.set_entry_type(tar::EntryType::Directory);
        dir_hdr.set_mode(0o755);
        dir_hdr.set_size(0);
        dir_hdr.set_cksum();
        tar.append_data(&mut dir_hdr, format!("{}/", root_dir), &[][..])
            .unwrap();

        // Directorio bin
        let mut bin_hdr = Header::new_gnu();
        bin_hdr.set_entry_type(tar::EntryType::Directory);
        bin_hdr.set_mode(0o755);
        bin_hdr.set_size(0);
        bin_hdr.set_cksum();
        tar.append_data(&mut bin_hdr, format!("{}/bin/", root_dir), &[][..])
            .unwrap();

        // bin/node
        let node_contents = b"binary-node";
        let mut node_hdr = Header::new_gnu();
        node_hdr.set_entry_type(tar::EntryType::Regular);
        node_hdr.set_mode(0o755);
        node_hdr.set_size(node_contents.len() as u64);
        node_hdr.set_cksum();
        tar.append_data(
            &mut node_hdr,
            format!("{}/bin/node", root_dir),
            &node_contents[..],
        )
        .unwrap();

        // README.md
        let readme = b"# demo\n";
        let mut readme_hdr = Header::new_gnu();
        readme_hdr.set_entry_type(tar::EntryType::Regular);
        readme_hdr.set_mode(0o644);
        readme_hdr.set_size(readme.len() as u64);
        readme_hdr.set_cksum();
        tar.append_data(
            &mut readme_hdr,
            format!("{}/README.md", root_dir),
            &readme[..],
        )
        .unwrap();

        // Cierra tar y encoder
        let enc = tar.into_inner().unwrap();
        enc.finish().unwrap();
    }
    out
}
fn read_string(p: &Path) -> String {
    fs::read_to_string(p).unwrap()
}

#[tokio::test]
async fn extract_aplana_y_preserva_contenido() {
    let tmp = TempDir::new().unwrap();
    let dest = tmp.path().join("dest");
    fs::create_dir_all(&dest).unwrap();

    let root_dir = "node-v23.11.1-darwin-arm64"; // nombre típico de node
    let txz = build_tar_xz_bytes(root_dir);

    let arch = TarXzArchive::new().unwrap();
    // el parámetro _version no afecta al resultado en tu impl actual
    arch.extract(&txz, &dest, "23.11.1").await.unwrap();

    // Debe aplanar: archivos en `dest/` directamente
    assert!(dest.join("bin/node").exists());
    assert_eq!(read_string(&dest.join("README.md")), "# demo\n");

    // No debería quedar el directorio raíz del tar
    assert!(
        !dest.join(root_dir).exists(),
        "No debería mantener el directorio raíz del tar"
    );
}

#[tokio::test]
async fn extract_falla_con_archivo_vacio() {
    let tmp = TempDir::new().unwrap();
    let dest = tmp.path().join("dest");
    fs::create_dir_all(&dest).unwrap();

    use tar::Builder as TarBuilder;
    use xz2::write::XzEncoder;

    let mut out = Vec::new();
    {
        let enc = XzEncoder::new(&mut out, 6);
        let tar = TarBuilder::new(enc);
        let enc = tar.into_inner().unwrap();
        enc.finish().unwrap();
    }

    let arch = TarXzArchive::new().unwrap();
    let res = arch.extract(&out, &dest, "x.y.z").await;
    assert!(res.is_err(), "Debe fallar con 'empty archive'");
}

#[tokio::test]
async fn extract_falla_si_hay_conflicto_en_destino() {
    let tmp = TempDir::new().unwrap();
    let dest = tmp.path().join("dest");
    fs::create_dir_all(dest.join("bin")).unwrap();
    fs::write(dest.join("bin/node"), "stale").unwrap();

    let txz = build_tar_xz_bytes("node-vX-linux-x64");
    let arch = TarXzArchive::new().unwrap();
    let res = arch.extract(&txz, &dest, "X").await;

    assert!(
        res.is_err(),
        "Debe fallar cuando ya existe `dest/bin/node` y se intenta renombrar encima"
    );
}
