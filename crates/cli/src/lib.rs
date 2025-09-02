use nve_core::domain::version::ParsedVersion;
use nve_core::error::NveError;
use nve_core::ports::{archive::Archive, fs::FileSystem, http::HttpClient, platform::Platform};
use nve_core::services::{InstallService, ResolveService};
use nve_core::state::layout::NveLayout;

pub async fn cmd_install<H, F, P, A>(
    http: &H,
    fs: &F,
    plat: &P,
    arch: &A,
    layout: &NveLayout,
    spec_str: &str,
) -> Result<(), NveError>
where
    H: HttpClient,
    F: FileSystem,
    P: Platform,
    A: Archive,
{
    let spec = parse_spec(spec_str)?;
    let svc = InstallService {
        http,
        fs,
        plat,
        arch,
        layout,
    };
    let exact = svc.install(&spec).await?;
    println!("Installed {}", exact);
    Ok(())
}

pub async fn cmd_remove<F, P>(
    fs: &F,
    plat: &P,
    layout: &NveLayout,
    spec_str: &str,
) -> Result<(), NveError>
where
    F: FileSystem,
    P: Platform,
{
    let spec = parse_spec(spec_str)?;
    let versions = fs.read_dir_names(&layout.versions_dir())?;
    let exact = resolve_installed(&versions, &spec)
        .ok_or_else(|| NveError::VersionNotInstalled(spec.full_version.clone()))?;
    let version_dir = layout.version_dir(&exact);

    if plat
        .is_current(&exact, &layout.current_dir())
        .await
        .unwrap_or(false)
    {
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

pub async fn cmd_list<F>(fs: &F, layout: &NveLayout) -> Result<(), NveError>
where
    F: FileSystem,
{
    let mut versions = if fs.exists(&layout.versions_dir()) {
        fs.read_dir_names(&layout.versions_dir())?
    } else {
        vec![]
    };
    versions.sort();
    for v in versions {
        println!("{v}");
    }
    Ok(())
}

pub async fn cmd_use<F, P>(
    fs: &F,
    plat: &P,
    layout: &NveLayout,
    spec_str: &str,
) -> Result<(), NveError>
where
    F: FileSystem,
    P: Platform,
{
    let spec = parse_spec(spec_str)?;
    let versions = if fs.exists(&layout.versions_dir()) {
        fs.read_dir_names(&layout.versions_dir())?
    } else {
        vec![]
    };
    let exact = resolve_installed(&versions, &spec)
        .ok_or_else(|| NveError::VersionNotInstalled(spec.full_version.clone()))?;
    let version_dir = layout.version_dir(&exact);
    plat.set_current(&version_dir, &layout.current_dir())
        .await?;
    println!("Using {}", exact);
    Ok(())
}

pub async fn cmd_remote<H: HttpClient>(http: &H, spec_str: &str) -> Result<(), NveError> {
    let spec = parse_spec(spec_str)?;
    let resolver = ResolveService { http };
    let exact = resolver.resolve(&spec).await?;
    println!("{exact}");
    Ok(())
}

pub fn parse_spec(input: &str) -> Result<ParsedVersion, NveError> {
    ParsedVersion::parse(input).map_err(|_| NveError::InvalidVersionFormat(input.to_string()))
}

pub fn resolve_installed(installed: &[String], spec: &ParsedVersion) -> Option<String> {
    use semver::Version;
    let mut best: Option<Version> = None;
    for s in installed {
        if let Ok(v) = Version::parse(s) {
            let matches = v.major == spec.major
                && spec.minor.is_none_or(|mn| v.minor == mn)
                && spec.patch.is_none_or(|pt| v.patch == pt);
            if matches {
                best = Some(best.map_or(v.clone(), |curr| std::cmp::max(curr, v)));
            }
        }
    }
    best.map(|v| v.to_string())
}
