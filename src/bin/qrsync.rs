extern crate clap;
#[macro_use]
extern crate log;
extern crate qrsync;

use std::env;
use std::path::Path;
use std::process;

use qrsync::http::QrSyncHttp;
use qrsync::ResultOrError;
use clap::Clap;
use log::LevelFilter;

/// Clap derived command line options.
#[derive(Clap, Debug)]
#[clap(author, about, version)]
struct Opts {
    /// File to be send to the mobile device.
    filename: Option<String>,
    /// Root directory to store files in receive mode.
    #[clap(short = 'r', long = "root-dir")]
    root_dir: Option<String>,
    /// Enable QrSync debug.
    #[clap(short = 'd', long = "debug")]
    debug: bool,
    /// Enable Rocket framework debug.
    #[clap(short = 'D', long = "rocket-debug")]
    rocket_debug: bool,
    /// Port to bind the HTTP server to.
    #[clap(short = 'p', long = "port", default_value = "5566")]
    port: u16,
    /// IP address to bind the HTTP server to. Default to primary interface.
    #[clap(short = 'i', long = "ip-address")]
    ip_address: Option<String>,
    /// Number of rocket workers.
    #[clap(short = 'w', long = "workers", default_value = "1")]
    workers: u16,
    /// Draw QR in a terminal with light background.
    #[clap(short = 'l', long = "light-term")]
    light_term: bool,
    /// Prefer IPv6 over IPv4.
    #[clap(short = '6', long = "ipv6")]
    ipv6: bool,
}

/// Setup logging, with different level configurations for QrSync and the rest of libraries used.
fn setup_logging(debug: bool, rocket_debug: bool) {
    let app_level = if debug {
        LevelFilter::Debug
    } else {
        LevelFilter::Info
    };
    let rocket_level = if rocket_debug {
        LevelFilter::Debug
    } else {
        LevelFilter::Error
    };
    pretty_env_logger::formatted_builder()
        .filter(Some("qrsync"), app_level)
        .filter(None, rocket_level)
        .init();
    debug!(
        "QrSync log level: {}, Rocket log level: {}",
        app_level, rocket_level
    );
}

/// Register signal handlers for SIGTERM, SIGINT and SIGQUIT
fn register_signal_handlers() -> ResultOrError<()> {
    ctrlc::set_handler(move || {
        warn!("Shutting down QrSync server");
        process::exit(0);
    })?;
    Ok(())
}

/// Parse command line flags, configure logging, register signal handlers and run QrSync.
fn run() -> ResultOrError<()> {
    let opts = Opts::parse();
    setup_logging(opts.debug, opts.rocket_debug);
    debug!("Command line options are {:#?}", opts);
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
        opts.workers,
        opts.light_term,
        opts.ipv6,
    );
    http.run()?;
    Ok(())
}

/// The main!
fn main() -> ! {
    match run() {
        Ok(_) => {
            info!("QrSync run successfully");
            process::exit(0);
        }
        Err(e) => {
            error!("Error running QrSync: {}", e);
            process::exit(1);
        }
    }
}
