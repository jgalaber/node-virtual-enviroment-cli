use std::path::PathBuf;
use thiserror::Error;

pub type Result<T> = std::result::Result<T, NveError>;

#[derive(Debug, Error)]
pub enum NveError {
    #[error("No se pudo encontrar el directorio HOME del usuario")]
    HomeDirNotFound,

    #[error("Error de entrada/salida: {0}")]
    Io(#[from] std::io::Error),

    #[error("Error HTTP/Red: {0}")]
    Http(#[from] reqwest::Error),

    #[error("Error parseando/serializando JSON: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Error semver: {0}")]
    Semver(#[from] semver::Error),

    // --------- Domain ---------
    #[error("Formato de versión inválido: '{0}' (usa '18', '18.19' o '18.19.1')")]
    InvalidVersionFormat(String),

    #[error("No se encontró la versión solicitada: '{0}'")]
    VersionNotFound(String),

    #[error("La versión no está instalada: '{0}'")]
    VersionNotInstalled(String),

    #[error("No hay versión activa configurada")]
    NoCurrentVersion,

    #[error("La ruta no pertenece a una instalación válida de Node: {0}")]
    InvalidInstallLayout(PathBuf),

    #[error("Conflicto de concurrencia: operación en curso")]
    ConcurrencyConflict,

    // --------- Download ---------
    #[error("El artefacto remoto no está disponible (404/403): {0}")]
    ArtifactUnavailable(String),

    #[error("Error extrayendo el archivo (zip/tar.xz): {0}")]
    ExtractError(String),

    // --------- Plataforma (OS) ---------
    #[error("Operación específica de la plataforma no soportada: {0}")]
    PlatformUnsupported(&'static str),

    #[error("Error estableciendo versión actual (atomic set)")]
    SetCurrentFailed,

    #[error("Error resolviendo nombre de archivo del artefacto para la plataforma {os}-{arch}")]
    ArchiveNameBuildFailed { os: String, arch: String },

    // --------- Unix ---------
    #[cfg(unix)]
    #[error("No se pudo crear/actualizar enlace simbólico: {0}")]
    SymlinkError(String),
}

impl NveError {
    pub fn artifact_unavailable(url: impl Into<String>) -> Self {
        NveError::ArtifactUnavailable(url.into())
    }

    pub fn extract_err(msg: impl Into<String>) -> Self {
        NveError::ExtractError(msg.into())
    }

    pub fn invalid_layout(path: impl Into<PathBuf>) -> Self {
        NveError::InvalidInstallLayout(path.into())
    }

    pub fn archive_name_build_failed(os: impl Into<String>, arch: impl Into<String>) -> Self {
        NveError::ArchiveNameBuildFailed {
            os: os.into(),
            arch: arch.into(),
        }
    }
}
