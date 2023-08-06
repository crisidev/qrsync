//! Definition and implementation of QrSync and dependencies errors.

use std::io::Error as IoError;
use std::net::AddrParseError;
use std::str::Utf8Error;

use axum::extract::multipart::MultipartError;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use base64::DecodeError;
use ctrlc::Error as CtrlcError;
use hyper::Error as HyperError;
use qr2term::QrError;
use thiserror::Error;

const ERROR_HTML: &str = include_str!("templates/error-custom.html");

/// Generic QrSync error structure, implementing all error types coming from dependencies.
#[derive(Error, Debug)]
pub enum QrSyncError {
    /// QrSync error.
    #[error("QrSync error: {0}")]
    Error(String),
    /// QrTerm error.
    #[error("QrTerm error: {0}")]
    QrTerm(#[from] QrError),
    /// Address parsing error.
    #[error("Address parsing error: {0}")]
    AddrParse(#[from] AddrParseError),
    /// I/O error.
    #[error("I/O error: {0}")]
    Io(#[from] IoError),
    /// Ctrl-c error.
    #[error("Ctrl-c error: {0}")]
    Ctrlc(#[from] CtrlcError),
    /// Base64 decode error.
    #[error("Base64 decode error: {0}")]
    Base64(#[from] DecodeError),
    /// UTF-8 error.
    #[error("UTF-8 error: {0}")]
    Utf8(#[from] Utf8Error),
    /// Hyper server error.
    #[error("Hyper server error: {0}")]
    Hyper(#[from] HyperError),
    /// Multipart form error.
    #[error("Multipart form error: {0}")]
    Multipart(#[from] MultipartError),
}

impl IntoResponse for QrSyncError {
    fn into_response(self) -> Response {
        let body = ERROR_HTML.replace("###ERRORMESSAGE###", &self.to_string());
        (StatusCode::INTERNAL_SERVER_ERROR, body).into_response()
    }
}
