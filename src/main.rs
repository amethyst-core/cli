// Modules
mod macros;
mod versions;
mod install;
mod uninstall;

// External Dependencies
use clap::{Args, Parser, Subcommand};
use std::path::PathBuf;
use color_eyre::eyre::Result;
use dirs::home_dir;
use colored::Colorize;
use tokio::signal;
use exitcode;
use std::env;

// Local Dependencies
use install::install;
use uninstall::uninstall;
use versions::get_latest_release;
use crate::versions::VersionV;

/// CLI structure to parse command-line arguments.
#[derive(Parser)]
#[command(
    version,
    about,
    long_about = None
)]
struct Cli {
    /// Commands.
    #[command(subcommand)]
    command: Commands,
    
}

/// Enum representing different commands.
#[derive(Subcommand)]
enum Commands {
    /// Install Amethyst.
    Install(InstallOptions),
    /// Uninstall Amethyst.
    Uninstall,
    // TODO: Implement These Commands
    Start, 
    Stop,
    Restart,
    Status,
    Update,
    Versions,
    Config, // Example: amethyst config --port 9999 --auto-cli-update true
}

// Commands Options
#[derive(Args)]
struct InstallOptions {
    /// Sets a custom installation path.
    #[arg(long, value_name = "PATH")]
    path: Option<PathBuf>,

    /// Specifies the version to install.
    #[arg(long, value_name = "VERSION")]
    version: Option<String>,
}

// Functions
fn check_system_requirements() -> bool {
    let os = env::consts::OS;
    match os {
        "windows" => true,
        "linux" => true,
        _ => false,
    }
}

// Constants
static GITHUB_TEXT: &str = "If you're unable to resolve the issue yourself, you can open an issue at https://github.com/amethyst-core/cli/issues for further assistance and troubleshooting.";

thread_local! {
    pub static CLI_VERSION: semver::Version = semver::Version::parse(env!("CARGO_PKG_VERSION")).unwrap();
}

#[tokio::main]
async fn main() -> Result<()> {

    // Parse command-line arguments into the CLI structure.
    let cli = Cli::parse();
    let _ = color_eyre::install().map_err(|e| warn!("An error occurred during initialization of the Color Eye module. {e}"));

    // TODO: Prints CLI Help if no subcommand is provided.

    // TODO: Shows CLI Information (Versions).
    
    // Check System Requirements.
    if !check_system_requirements() {
        error!("System requirements not met. Please check the system requirements and try again.");
        std::process::exit(exitcode::CONFIG);
    }
    
    // Spawn a task to handle the CTRL+C signal
    tokio::spawn(async move {
        signal::ctrl_c().await.unwrap();
        warn!("The system has detected a CTRL+C (interrupt) signal and is exiting now.\n\t Please note: Any unsaved changes will be forever lost.");
        std::process::exit(exitcode::DATAERR);
    });
    
    // TODO: Check For Core Updates Automatically (if enabled in config)

    // Match the command and execute corresponding logic.
    match &cli.command {
        Commands::Install(opts) => {

            let install_version: VersionV;

            if let Some(version) = &opts.version {
                install_version = VersionV::from_str(version)?;
            } else {
                install_version = get_latest_release().await?;
            };

            // Install Amethyst Core & Panel.
            let install_path: PathBuf;

            if let Some(path) = &opts.path {
                install_path = home_dir()
                    .expect("Could not find home directory")
                    .join(path)
                    .join(".amethyst");
            } else {
                install_path = home_dir()
                    .expect("Could not find home directory")
                    .join(".amethyst");
            }
            match install(&install_path, install_version, CLI_VERSION.with(|v| v.clone())).await {
                Ok(_) => {
                    success!("Installation Completed");
                },
                Err(e) => {
                    error!("Failed to Install: {}\n", e);
                    info!("{}", GITHUB_TEXT);
                },
            };
        }
        Commands::Uninstall => {

            // Uninstall Amethyst Core & Panel.
            match uninstall() {
                Ok(_) => {
                    success!("Uninstallation Completed");
                },
                Err(e) => {
                    error!("Failed to Uninstall: {}\n", e);
                    info!("{}", GITHUB_TEXT);
                },
            };
        }
        _ => {}
    }

    Ok(())
}
