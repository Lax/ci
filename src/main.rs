mod adapter;
mod cli;
mod config;
mod db;
mod sync;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "ci", version, about = "Personal word frequency manager")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize a freq-db repository
    Init {
        /// Directory to initialize (default: current dir)
        dir: Option<String>,
    },
    /// Import from Rime user dictionary
    Import,
    /// Export to Rime user dictionary
    Export,
    /// Sync with remote (git pull + merge + push)
    Sync,
    /// Scan blog repos for personal vocabulary
    Scan,
    /// Show statistics
    Status,
    /// Manage devices
    Device {
        #[command(subcommand)]
        action: DeviceAction,
    },
}

#[derive(Subcommand)]
enum DeviceAction {
    /// List known devices
    List,
    /// Add a device
    Add {
        name: String,
    },
}

fn main() -> anyhow::Result<()> {
    env_logger::init();
    let cli = Cli::parse();

    match cli.command {
        Commands::Init { dir } => {
            let d = dir.unwrap_or_else(|| ".".to_string());
            cli::init::run(&d)
        }
        Commands::Import => cli::import::run(),
        Commands::Export => cli::export::run(),
        Commands::Sync => cli::sync_cmd::run(),
        Commands::Scan => cli::scan::run(),
        Commands::Status => cli::status::run(),
        Commands::Device { action } => match action {
            DeviceAction::List => cli::device::run_list(),
            DeviceAction::Add { name: _ } => cli::device::run_add(),
        },
    }
}
