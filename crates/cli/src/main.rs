use clap::{Parser, Subcommand};
use nve_core::error::NveError;
use nve_core::domain::version::ParsedVersion;
use nve_core::services::{ResolveService, InstallService};
use nve_core::state::layout::NveLayout;
use nve_core::ports::{http::HttpClient, fs::FileSystem, platform::Platform, archive::Archive};

use nve_infra::http_client::ReqwestHttp;
use nve_infra::fs_std::StdFs;

#[cfg(unix)]
use nve_infra::platform::UnixPlatform as HostPlatform;
#[cfg(unix)]
use nve_infra::archive::TarXzArchive as HostArchive;

#[cfg(windows)]
use nve_infra::platform::WindowsPlatform as HostPlatform;
#[cfg(windows)]
use nve_infra::archive::ZipArchive as HostArchive;

/// nve - Node Version Environment
#[derive(Parser, Debug)]
#[command(name = "nve", version, about = "Node.js version manager (without system privileges)")]
struct Cli {
    #[command(subcommand)]
    cmd: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    #[command(alias = "add")]
    Install { spec: String },
    #[command(alias = "uninstall")]
    Remove { spec: String },
    List,
    Use { spec: String },
    Remote { spec: String },
}

#[tokio::main]
async fn main() -> Result<(), NveError> {
    let cli = Cli::parse();
    
    let base = dirs::home_dir().ok_or(NveError::HomeDirNotFound)?.join(".nve");
    let layout = NveLayout { base };

    let httpc = ReqwestHttp::default();
    let fs = StdFs::new();
    let plat = HostPlatform::new()?;
    let arch = HostArchive::new()?; 

    match cli.cmd {
        Commands::Install { spec } => cmd_install(&httpc, &fs, &plat, &arch, &layout, &spec).await?,
        Commands::Remove  { spec } => cmd_remove(&fs, &plat, &layout, &spec).await?,
        Commands::List             => cmd_list(&fs, &layout).await?,
        Commands::Use     { spec } => cmd_use(&fs, &plat, &layout, &spec).await?,
        Commands::Remote  { spec } => cmd_remote(&httpc, &spec).await?,
    }

    Ok(())
}

// Commands
async fn cmd_install<H, F, P, A>(http: &H, fs: &F, plat: &P, arch: &A, layout: &NveLayout, spec_str: &str) -> Result<(), NveError>
where H: HttpClient, F: FileSystem, P: Platform, A: Archive {
    let spec = parse_spec(spec_str)?;
    let svc = InstallService { http, fs, plat, arch, layout };
    let exact = svc.install(&spec).await?;
    println!("Installed {}", exact);
    Ok(())
}

async fn cmd_remove<F, P>(fs: &F, plat: &P, layout: &NveLayout, spec_str: &str) -> Result<(), NveError>
where F: FileSystem, P: Platform {
    let spec = parse_spec(spec_str)?;
    let versions = fs.read_dir_names(&layout.versions_dir())?;
    let exact = resolve_installed(&versions, &spec)
        .ok_or_else(|| NveError::VersionNotInstalled(spec.full_version.clone()))?;
    let version_dir = layout.version_dir(&exact);

    if plat.is_current(&exact, &layout.current_dir()).await.unwrap_or(false) {
        let cur = layout.current_dir();
        if fs.exists(&cur) {
            fs.remove_dir_all(&cur)?;
            fs.create_dir_all(&cur)?;
        }
    }

    fs.remove_dir_all(&version_dir)?;
    println!("Removed {}", exact);
    Ok(())
}

async fn cmd_list<F>(fs: &F, layout: &NveLayout) -> Result<(), NveError>
where F: FileSystem {
    let mut versions = if fs.exists(&layout.versions_dir()) {
        fs.read_dir_names(&layout.versions_dir())?
    } else { vec![] };
    versions.sort();
    for v in versions { println!("{v}"); }
    Ok(())
}

async fn cmd_use<F, P>(fs: &F, plat: &P, layout: &NveLayout, spec_str: &str) -> Result<(), NveError>
where F: FileSystem, P: Platform {
    let spec = parse_spec(spec_str)?;
    let versions = if fs.exists(&layout.versions_dir()) {
        fs.read_dir_names(&layout.versions_dir())?
    } else { vec![] };
    let exact = resolve_installed(&versions, &spec)
        .ok_or_else(|| NveError::VersionNotInstalled(spec.full_version.clone()))?;
    let version_dir = layout.version_dir(&exact);
    plat.set_current(&version_dir, &layout.current_dir()).await?;
    println!("Using {}", exact);
    Ok(())
}

async fn cmd_remote<H: HttpClient>(http: &H, spec_str: &str) -> Result<(), NveError> {
    let spec = parse_spec(spec_str)?;
    let resolver = ResolveService { http };
    let exact = resolver.resolve(&spec).await?;
    println!("{exact}");
    Ok(())
}

// Helpers
fn parse_spec(input: &str) -> Result<ParsedVersion, NveError> {
    ParsedVersion::parse(input).map_err(|_| NveError::InvalidVersionFormat(input.to_string()))
}

fn resolve_installed(installed: &[String], spec: &ParsedVersion) -> Option<String> {
    use semver::Version;
    let mut best: Option<Version> = None;
    for s in installed {
        if let Ok(v) = Version::parse(s) {
            let matches =
                v.major == spec.major &&
                spec.minor.map_or(true, |mn| v.minor == mn) &&
                spec.patch.map_or(true, |pt| v.patch == pt);
            if matches {
                best = Some(best.map_or(v.clone(), |curr| std::cmp::max(curr, v)));
            }
        }
    }
    best.map(|v| v.to_string())
}
