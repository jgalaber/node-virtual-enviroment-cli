use nve_core::error::NveError;

#[test]
fn display_de_variantes_simples_y_helpers() {
    assert_eq!(
        NveError::HomeDirNotFound.to_string(),
        "No se pudo encontrar el directorio HOME del usuario"
    );
    assert!(NveError::InvalidVersionFormat("18.x".into())
        .to_string()
        .contains("Formato de versión inválido"));
    assert!(NveError::VersionNotFound("23.9.9".into())
        .to_string()
        .contains("No se encontró la versión solicitada"));
    assert!(NveError::VersionNotInstalled("20.0.0".into())
        .to_string()
        .contains("La versión no está instalada"));
    assert_eq!(
        NveError::NoCurrentVersion.to_string(),
        "No hay versión activa configurada"
    );
    assert!(NveError::invalid_layout("/tmp/fake")
        .to_string()
        .contains("La ruta no pertenece a una instalación válida de Node"));

    assert_eq!(
        NveError::ConcurrencyConflict.to_string(),
        "Conflicto de concurrencia: operación en curso"
    );
    assert!(NveError::PlatformUnsupported("symlink")
        .to_string()
        .contains("Operación específica de la plataforma no soportada"));

    assert!(NveError::artifact_unavailable("http://example.com/x.zip")
        .to_string()
        .contains("El artefacto remoto no está disponible"));
    assert!(NveError::extract_err("boom")
        .to_string()
        .contains("Error extrayendo el archivo"));

    let msg = NveError::archive_name_build_failed("win", "x64").to_string();
    assert!(
        msg.contains(
            "Error resolviendo nombre de archivo del artefacto para la plataforma win-x64"
        ),
        "{msg}"
    );
}

#[cfg(unix)]
#[test]
fn unix_symlink_error_display() {
    assert!(NveError::SymlinkError("permiso denegado".into())
        .to_string()
        .contains("No se pudo crear/actualizar enlace simbólico"));
}

#[test]
fn from_io_error() {
    use std::io::Error;
    let e = Error::other("iofail");
    let nve = NveError::from(e);
    let s = nve.to_string();
    assert!(matches!(nve, NveError::Io(_)));
    assert!(s.contains("Error de entrada/salida"));
    assert!(s.contains("iofail"));
}

#[test]
fn from_serde_json_error() {
    let json_err = serde_json::from_str::<serde_json::Value>("not-json").unwrap_err();
    let nve = NveError::from(json_err);
    let s = nve.to_string();
    assert!(matches!(nve, NveError::Json(_)));
    assert!(s.contains("Error parseando/serializando JSON"));
}

#[test]
fn from_semver_error() {
    let semver_err = semver::Version::parse("bad.version").unwrap_err();
    let nve = NveError::from(semver_err);
    let s = nve.to_string();
    assert!(matches!(nve, NveError::Semver(_)));
    assert!(s.contains("Error semver"));
}

#[tokio::test]
async fn from_reqwest_error_sin_red() {
    let err = reqwest::Client::new()
        .get("ftp://example.com/file")
        .send()
        .await
        .unwrap_err();
    let nve = NveError::from(err);
    let s = nve.to_string();
    assert!(matches!(nve, NveError::Http(_)));
    assert!(s.starts_with("Error HTTP/Red:"));
}
