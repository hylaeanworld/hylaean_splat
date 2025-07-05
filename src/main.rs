use clap::{Parser, Subcommand};
use hylaean_splat::cli::Commands;
use hylaean_splat::core::HylaeanSplat;
use log::info;

#[derive(Parser)]
#[command(name = "hylaeansplat")]
#[command(about = "Operating system for 3D Gaussian splatting tools")]
#[command(version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
    
    #[arg(short, long, global = true)]
    pub verbose: bool,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    
    // Initialize logging
    if cli.verbose {
        env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug")).init();
    } else {
        env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    }
    
    info!("Starting Hylaean Splat...");
    
    // Initialize the core system
    let mut hylaeansplat = HylaeanSplat::new().await?;
    
    // Execute the command
    hylaeansplat.execute_command(cli.command).await?;
    
    Ok(())
}