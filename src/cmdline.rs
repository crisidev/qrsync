//! Command line parsing implementation.

use clap::Clap;

/// Clap derived command line options.
#[derive(Clap, Debug)]
#[clap(author, about, version)]
pub struct Opts {
    /// File to be send to the mobile device.
    pub filename: Option<String>,
    /// Root directory to store files in receive mode.
    #[clap(short, long)]
    pub root_dir: Option<String>,
    /// Enable QrSync debug.
    #[clap(short, long)]
    pub debug: bool,
    /// Enable Rocket framework debug.
    #[clap(long)]
    pub rocket_debug: bool,
    /// Port to bind the HTTP server to.
    #[clap(short, long, default_value = "5566")]
    pub port: u16,
    /// IP address to bind the HTTP server to. Default to primary interface.
    #[clap(short, long)]
    pub ip_address: Option<String>,
    /// Number of rocket workers.
    #[clap(short, long, default_value = "1")]
    pub workers: u16,
    /// Draw QR in a terminal with light background.
    #[clap(short, long)]
    pub light_term: bool,
}

/// Parse command line options and return a Clap structure.
pub fn parse_command_line() -> Opts {
    Opts::parse()
}
