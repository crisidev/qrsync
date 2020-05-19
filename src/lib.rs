#![feature(proc_macro_hygiene, decl_macro, ip)]
#![forbid(unsafe_code)]

//! ## QrSync
//! Utility to copy files over WiFi to/from mobile devices inside a terminal.
//!
//! When I built QrSync, it was only meant to send files from a terminal to a mobile device, then I
//! found the amazing [qrcp](https://github.com/claudiodangelis/qrcp) and I took some ideas from it and
//! implemented also the possibility to copy file from the mobile device to the computer running QrSync.
//!
//! ### Acknowledgement
//! * [qrcp](https://github.com/claudiodangelis/qrcp): I took many ideas from this amazing project
//! and "stole" most of the HTML Bootstrap based UI.
//! * [rocket](https://rocket.rs/): A great HTTP framework for Rust, very expandable and simple to
//! use.
//! * [qr2term](https://docs.rs/qr2term/): Terminal based QR rendering library.
//! * [clap](https://clap.rs/): Oh man, where do I start telling how much do I love Clap?
//!
//! See Github project [README](https://github.com/crisidev/qrsync/blob/master/README.md) for more
//! info.

extern crate base64;
extern crate clap;
extern crate ctrlc;
#[macro_use]
extern crate log;
#[cfg(target_family = "unix")]
extern crate pnet;
extern crate pretty_env_logger;
extern crate qr2term;
#[macro_use]
extern crate rocket;
extern crate rocket_contrib;
extern crate rocket_multipart_form_data;

pub mod error;
pub mod http;
pub mod routes;
pub mod utils;
