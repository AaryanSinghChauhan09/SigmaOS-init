use std::env;
//! SigmaOS-init - SigmaOS Component CLI
//!
//! Command-line interface for SigmaOS-init component.

use anyhow::Result;
use clap::{Parser, Subcommand};
use log::info;
use init::{Component, ComponentStatus, Config, init};

#[derive(Parser)]
#[command(name = "init")]
#[command(about = "SigmaOS SigmaOS-init component", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize the component
    Init {
        /// Enable debug mode
        #[arg(short, long)]
        debug: bool,
    },
    /// Start the component
    Start,
    /// Stop the component
    Stop,
    /// Get component status
    Status,
    /// Run the component
    Run {
        /// Enable debug mode
        #[arg(short, long)]
        debug: bool,
    },
}

fn main() -> Result<()> { -> Result<()> {
    let cli = Cli::parse();
    
    let config = Config::default();
    let mut component = init::new(config);

    match cli.command {
        Commands::Init { debug } => {
            if debug {
                println!("Initializing SigmaOS-init in debug mode...");
            } else {
                println!("Initializing SigmaOS-init...");
            }
            component.initialize()?;
            println!("SigmaOS-init initialized successfully");
        }
        Commands::Start => {
            component.initialize()?;
            component.start()?;
            println!("SigmaOS-init started successfully");
        }
        Commands::Stop => {
            component.stop()?;
            println!("SigmaOS-init stopped successfully");
        }
        Commands::Status => {
            let status = component.status();
            println!("SigmaOS-init status: {:?}", status);
        }
        Commands::Run { debug } => {
            if debug {
                println!("Running SigmaOS-init in debug mode...");
            } else {
                println!("Running SigmaOS-init...");
            }
            component.initialize()?;
            component.start()?;
            println!("SigmaOS-init is running. Press Ctrl+C to stop.");
            
            // Keep running
            tokio::runtime::Runtime::new()?.block_on(async {
                tokio::signal::ctrl_c().await.unwrap();
                info!("Received Ctrl+C, shutting down...");
            });
            
            component.stop()?;
            println!("SigmaOS-init stopped");
        }
    }

    Ok(())
}

#[derive(Debug, Clone)]
struct Config {
    verbose: bool,
    dry_run: bool,
}

impl Config {
    fn from_env() -> Self {
        Config {
            verbose: env::var("VERBOSE").is_ok(),
            dry_run: env::var("DRY_RUN").is_ok(),
        }
    }
}
