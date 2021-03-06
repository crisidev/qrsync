//! Definition and implementation of QrSync and dependencies errors.

use std::fmt;
use std::io::Error as IoError;
use std::net::AddrParseError;
use std::str::Utf8Error;

use base64::DecodeError;
use ctrlc::Error as CtrlcError;
use qr2term::QrError;
use rocket::config::ConfigError;
use rocket_multipart_form_data::mime::FromStrError;
use rocket_multipart_form_data::MultipartFormDataError;

/// Generic QrSync error structure, implementing all error types coming from dependencies.
#[derive(Debug, PartialEq)]
pub struct QrSyncError {
    kind: String,
    message: String,
}

impl QrSyncError {
    pub fn new(message: &str, kind: Option<&str>) -> QrSyncError {
        let kind_value = kind.unwrap_or("qrsync");
        QrSyncError {
            kind: String::from(kind_value),
            message: message.to_string(),
        }
    }
}

impl fmt::Display for QrSyncError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}: {}", self.kind, self.message)
    }
}

impl From<ConfigError> for QrSyncError {
    fn from(error: ConfigError) -> Self {
        QrSyncError {
            kind: String::from("rocket-config"),
            message: error.to_string(),
        }
    }
}

impl From<QrError> for QrSyncError {
    fn from(error: QrError) -> Self {
        QrSyncError {
            kind: String::from("qr-term"),
            message: error.to_string(),
        }
    }
}

impl From<AddrParseError> for QrSyncError {
    fn from(error: AddrParseError) -> Self {
        QrSyncError {
            kind: String::from("qr-term"),
            message: error.to_string(),
        }
    }
}

impl From<IoError> for QrSyncError {
    fn from(error: IoError) -> Self {
        QrSyncError {
            kind: String::from("io"),
            message: error.to_string(),
        }
    }
}

impl From<FromStrError> for QrSyncError {
    fn from(error: FromStrError) -> Self {
        QrSyncError {
            kind: String::from("multipart-form"),
            message: error.to_string(),
        }
    }
}

impl From<MultipartFormDataError> for QrSyncError {
    fn from(error: MultipartFormDataError) -> Self {
        QrSyncError {
            kind: String::from("multipart-form"),
            message: error.to_string(),
        }
    }
}

impl From<CtrlcError> for QrSyncError {
    fn from(error: CtrlcError) -> Self {
        QrSyncError {
            kind: String::from("ctrlc"),
            message: error.to_string(),
        }
    }
}

impl From<DecodeError> for QrSyncError {
    fn from(error: DecodeError) -> Self {
        QrSyncError {
            kind: String::from("base64"),
            message: error.to_string(),
        }
    }
}

impl From<Utf8Error> for QrSyncError {
    fn from(error: Utf8Error) -> Self {
        QrSyncError {
            kind: String::from("utf-8"),
            message: error.to_string(),
        }
    }
}
