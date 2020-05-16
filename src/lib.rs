#![feature(proc_macro_hygiene, decl_macro, ip)]
#![forbid(unsafe_code)]

//! ## QrSync
//! Utility to copy files over WiFi to/from mobile devices inside a terminal.
//!
//! When I built QrSync, it was only meant to send files from a terminal to a mobile device, then I
//! found the amazing [qrcp](https://github.com/claudiodangelis/qrcp) and I took some ideas from it and
//! implemented also the possibility to copy file from the mobile device to the computer running QrSync.
//!
//! ### Dependencies
//! QrSync depends on Rocket, which only builds against nightly >= 1.42.
//!
//! ### Operational modes
//! QrSync can run in two mode, depending on command line options:
//! * Send mode: this mode is selected when a file is passed to the command line. QrSync will
//! generate a QR code on the terminal and start the HTTP server in send mode.
//!     Example:
//!     ```sh
//!     ❯❯❯ qrsync my_document.pdf
//!      INFO  qrsync::http > Send mode enabled for file /home/bigo/my_document.pdf
//!      INFO  qrsync::http > Scan this QR code with a QR code reader app to open the URL http://192.168.1.11:5566/send
//!     ```
//! * Receive mode: this mode is selected if no file is passed to the command line. QrSync will
//! generate a QR code on the terminal and start the HTTP server in receive mode from the current
//! folder. A specific folder to save received files can be specified with --root-dir command line
//! option.
//!     Example:
//!     ```sh
//!     ❯❯❯ qrsync
//!      INFO  qrsync::http > Receive mode enabled inside directory /home/bigo
//!      INFO  qrsync::http > Scan this QR code with a QR code reader app to open the URL http://192.168.1.11:5566/receive
//!     ```
//!
//! ### Command line options
//! ```sh
//! USAGE:
//!     qrsync [FLAGS] [OPTIONS] [filename]
//!
//! ARGS:
//!     <filename>    File to be send to the mobile device
//!
//! FLAGS:
//!     -d, --debug           Enable QrSync debug
//!     -h, --help            Prints help information
//!     -l, --light-term      Draw QR in a terminal with light background
//!         --rocket-debug    Enable Rocket framework debug
//!     -V, --version         Prints version information
//!
//! OPTIONS:
//!     -i, --ip-address <ip-address>    IP address to bind the HTTP server to. Default to primary interface
//!     -p, --port <port>                Port to bind the HTTP server to [default: 5566]
//!     -r, --root-dir <root-dir>        Root directory to store files in receive mode
//!     -w, --workers <workers>          Number of rocket workers [default: 1]
//! ```
//!
//! ### Acknowledgement
//! * [qrcp](https://github.com/claudiodangelis/qrcp): I took many ideas from this amazing project
//! and "stole" most of the HTML Bootstrap based UI.
//! * [rocket](https://rocket.rs/): A great HTTP framework for Rust, very expandable and simple to
//! use.
//! * [qr2term](https://docs.rs/qr2term/): Terminal based QR rendering library.
//! * [clap](https://clap.rs/): Oh man, where do I start telling how much do I love Clap?

extern crate base64;
extern crate clap;
#[macro_use]
extern crate log;
extern crate pretty_env_logger;
extern crate qr2term;
#[macro_use]
extern crate rocket;
extern crate get_if_addrs;
extern crate pnet;
extern crate rocket_contrib;
extern crate rocket_multipart_form_data;

pub mod error;
pub mod http;
pub mod routes;
pub mod utils;
