use clap::{Parser, Subcommand};
use nve_cli::{cmd_install, cmd_list, cmd_remote, cmd_remove, cmd_use};
use nve_core::error::NveError;
use nve_core::state::layout::NveLayout;

use nve_infra::fs_std::StdFs;
use nve_infra::http_client::ReqwestHttp;

#[cfg(unix)]
use nve_infra::archive::TarXzArchive as HostArchive;
#[cfg(unix)]
use nve_infra::platform::UnixPlatform as HostPlatform;

#[cfg(windows)]
use nve_infra::archive::ZipArchive as HostArchive;
#[cfg(windows)]
use nve_infra::platform::WindowsPlatform as HostPlatform;

/// nve - Node Version Environment
#[derive(Parser, Debug)]
#[command(
    name = "nve",
    version,
    about = "Node.js version manager (without system privileges)"
)]
struct Cli {
    #[command(subcommand)]
    cmd: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    #[command(alias = "add")]
    Install {
        spec: String,
    },
    #[command(alias = "uninstall")]
    Remove {
        spec: String,
    },
    List,
    Use {
        spec: String,
    },
    Remote {
        spec: String,
    },
}

#[tokio::main]
async fn main() -> Result<(), NveError> {
    let cli = Cli::parse();

    let base = dirs::home_dir()
        .ok_or(NveError::HomeDirNotFound)?
        .join(".nve");
    let layout = NveLayout { base };

    let httpc = ReqwestHttp::default();
    let fs = StdFs::new();
    let plat = HostPlatform::new()?;
    let arch = HostArchive::new()?;

    match cli.cmd {
        Commands::Install { spec } => {
            cmd_install(&httpc, &fs, &plat, &arch, &layout, &spec).await?
        }
        Commands::Remove { spec } => cmd_remove(&fs, &plat, &layout, &spec).await?,
        Commands::List => cmd_list(&fs, &layout).await?,
        Commands::Use { spec } => cmd_use(&fs, &plat, &layout, &spec).await?,
        Commands::Remote { spec } => cmd_remote(&httpc, &spec).await?,
    }

    Ok(())
}
