mod cagen;
mod daemon;
mod error;
mod proxy;
mod serve;

use anyhow::Result;
use clap::{Args, Parser, Subcommand};
use reqwest::Url;
use std::{net::SocketAddr, path::PathBuf};

#[global_allocator]
static ALLOC: mimalloc::MiMalloc = mimalloc::MiMalloc;

#[derive(Parser)]
#[clap(author, version, about, arg_required_else_help = true)]
#[command(args_conflicts_with_subcommands = true)]
pub struct Opt {
    #[clap(subcommand)]
    pub commands: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Run server
    Run(BootArgs),
    /// Start server daemon
    Start(BootArgs),
    /// Restart server daemon  
    Restart(BootArgs),
    /// Stop server daemon
    Stop,
    /// Show the server daemon log
    Log,
    /// Show the server daemon process
    #[cfg(target_family = "unix")]
    PS,
    /// Show the server daemon process
    #[cfg(target_family = "windows")]
    Status,
}

#[derive(Args, Clone, Debug)]
pub struct BootArgs {
    /// Debug mode
    #[clap(short, long)]
    pub debug: bool,

    /// Bind address
    #[clap(short, long, default_value = "0.0.0.0:1080")]
    pub bind: SocketAddr,

    /// Upstream proxy
    #[clap(short, long)]
    pub proxy: Option<Url>,

    /// MITM server CA certificate file path
    #[clap(long, default_value = "ca/cert.crt", requires = "bind")]
    pub cert: PathBuf,

    /// MITM server CA private key file path
    #[clap(long, default_value = "ca/key.pem", requires = "bind")]
    pub key: PathBuf,
}

fn main() -> Result<()> {
    let opt = Opt::parse();

    match opt.commands {
        Commands::Run(args) =>  serve::Serve(args).run()?,
        Commands::Start(args) => daemon::start(args)?,
        Commands::Restart(args) => daemon::restart(args)?,
        Commands::Stop => daemon::stop()?,
        Commands::Log => daemon::log()?,
        #[cfg(target_family = "unix")]
        Commands::PS => daemon::status()?,
        #[cfg(target_family = "windows")]
        Commands::Status => daemon::status()?,
    };

    Ok(())
}
