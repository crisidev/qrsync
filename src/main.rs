use std::env;
use std::path::Path;
use std::process;

use clap::Parser;
use qrsync::{QrSyncHttp, QrSyncResult};
use tracing_subscriber::filter::LevelFilter;
use tracing_subscriber::{prelude::*, EnvFilter};

/// Clap derived command line options.
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Opts {
    /// File to be send to the mobile device.
    filename: Option<String>,
    /// Root directory to store files in receive mode.
    #[clap(short = 'r', long = "root-dir")]
    root_dir: Option<String>,
    /// Enable QrSync debug.
    #[clap(short = 'd', long = "debug")]
    debug: bool,
    /// Port to bind the HTTP server to.
    #[clap(short = 'p', long = "port", default_value = "5566")]
    port: u16,
    /// IP address to bind the HTTP server to. Default to primary interface.
    #[clap(short = 'i', long = "ip-address")]
    ip_address: Option<String>,
    /// Draw QR in a terminal with light background.
    #[clap(short = 'l', long = "light-term")]
    light_term: bool,
    /// Prefer IPv6 over IPv4.
    #[clap(short = '6', long = "ipv6")]
    ipv6: bool,
}

/// Setup `tracing::subscriber` to read the log level from RUST_LOG environment variable.
fn setup_tracing() {
    let format = tracing_subscriber::fmt::layer()
        .with_ansi(true)
        .with_line_number(true)
        .with_level(true)
        .without_time();
    match EnvFilter::try_from_default_env() {
        Ok(filter) => {
            tracing_subscriber::registry().with(format).with(filter).init();
        }
        Err(_) => {
            tracing_subscriber::registry()
                .with(format)
                .with(LevelFilter::INFO)
                .init();
        }
    }
}

/// Register signal handlers for SIGTERM, SIGINT and SIGQUIT
fn register_signal_handlers() -> QrSyncResult<()> {
    ctrlc::set_handler(move || {
        tracing::warn!("Shutting down QrSync server");
        process::exit(0);
    })?;
    Ok(())
}

/// Parse command line flags, configure logging, register signal handlers and run QrSync.
async fn run() -> QrSyncResult<()> {
    let opts = Opts::parse();
    setup_tracing();
    tracing::debug!("Command line options are {:#?}", opts);
    register_signal_handlers()?;
    let root_dir = match opts.root_dir {
        Some(r) => Path::new(&r).to_path_buf(),
        None => env::current_dir()?,
    };
    let http = QrSyncHttp::new(
        opts.ip_address,
        opts.port,
        opts.filename,
        root_dir,
        opts.light_term,
        opts.ipv6,
    );
    http.run().await?;
    Ok(())
}

/// The main!
#[tokio::main]
async fn main() -> ! {
    match run().await {
        Ok(_) => {
            tracing::info!("QrSync run successfully");
            process::exit(0);
        }
        Err(e) => {
            tracing::error!("Error running QrSync: {}", e);
            process::exit(1);
        }
    }
}
