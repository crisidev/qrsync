//! Definition and implementation of QrSync and dependencies errors.

use std::io::Error as IoError;
use std::net::AddrParseError;
use std::str::Utf8Error;

use base64::DecodeError;
use ctrlc::Error as CtrlcError;
use hyper::Error as HyperError;
use qr2term::QrError;
use thiserror::Error;

/// Generic QrSync error structure, implementing all error types coming from dependencies.
#[derive(Error, Debug)]
pub enum QrSyncError {
    #[error("QrSync error: {0}")]
    Error(String),
    #[error("QrTerm error: {0}")]
    QrTerm(#[from] QrError),
    #[error("Address parsing error: {0}")]
    AddrParse(#[from] AddrParseError),
    #[error("I/O error: {0}")]
    Io(#[from] IoError),
    #[error("Ctrl-c error: {0}")]
    Ctrlc(#[from] CtrlcError),
    #[error("Base64 decode error: {0}")]
    Base64(#[from] DecodeError),
    #[error("UTF-8 error: {0}")]
    Utf8(#[from] Utf8Error),
    #[error("Hyper server error: {0}")]
    Hyper(#[from] HyperError),
}
