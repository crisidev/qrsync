//! Shared utility functionalities.

use log::LevelFilter;
use rocket_multipart_form_data::mime::Mime;
use signal_hook::{iterator::Signals, SIGINT, SIGQUIT, SIGTERM};
use std::fs;
use std::path::Path;
use std::{process, thread};

use crate::error::QrSyncError;

/// Handy type handling Result and Errors.
pub type ResultOrError<T> = Result<T, QrSyncError>;

/// Setup logging, with different level configurations for QrSync and the rest of libraries used.
pub fn setup_logging(debug: bool, rocket_debug: bool) {
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
pub fn register_signal_handlers() -> ResultOrError<()>{
    let signals = if cfg!(windows) {
        Signals::new(&[SIGTERM, SIGINT])?
    } else {
        Signals::new(&[SIGTERM, SIGINT, SIGQUIT])?
    };
    thread::spawn(move || {
        for sig in signals.forever() {
            warn!("Received signal {:#?}. Shutting down QrSync server", sig);
            process::exit(0);
        }
    });
    debug!("Registered signal handlers for SIGTERM, SIGINT, SIGQUIT");
    Ok(())
}

/// Copy a file from a source to a destination. The file_name and content_type are used to produce
/// nice errors.
pub fn copy_file(file_name: &str, content_type: &Mime, src: &Path, dst: &Path) {
    match fs::copy(src, dst) {
        Ok(_) => info!(
            "Received file with content-type {} stored in {}",
            content_type,
            dst.to_string_lossy()
        ),
        Err(e) => error!(
            "Unable to store file {} to {}: {}",
            file_name,
            dst.to_string_lossy(),
            e
        ),
    }
}
