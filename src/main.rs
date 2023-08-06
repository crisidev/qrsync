use std::env;
use std::path::Path;
use std::process;

use argh::FromArgs;
use qrsync::{QrSyncHttp, QrSyncResult};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

/// qrsync - copy files over WiFI using QR codes.
#[derive(FromArgs, Debug)]
struct Opts {
    /// file to be send to the mobile device.
    #[argh(positional)]
    filename: Option<String>,
    /// root directory to store files in receive mode.
    #[argh(option, short = 'r')]
    root_dir: Option<String>,
    /// enable QrSync debug.
    #[argh(switch, short = 'd')]
    debug: bool,
    /// port to bind the HTTP server to.
    #[argh(option, short = 'p', default = "5566")]
    port: u16,
    /// ip address to bind the HTTP server to. Default to primary interface.
    #[argh(option, short = 'i')]
    ip_address: Option<String>,
    /// draw QR in a terminal with light background.
    #[argh(switch, short = 'l')]
    light_term: bool,
    /// prefer IPv6 over IPv4.
    #[argh(switch, short = '6')]
    ipv6: bool,
    /// show version info.
    #[argh(switch, short = 'v')]
    version: bool,
}

/// Setup `tracing::subscriber` to read the log level from RUST_LOG environment variable.
fn setup_tracing(debug: bool) {
    let level = if debug { "debug" } else { "info" };
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| format!("qrsync={level},tower_http={level},axum::rejection=trace").into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();
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
    let opts: Opts = argh::from_env();
    if opts.version {
        println!("qrsync v{} - {}", env!("CARGO_PKG_VERSION"), env!("CARGO_PKG_AUTHORS"));
        process::exit(0)
    }
    setup_tracing(opts.debug);
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
