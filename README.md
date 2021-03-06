## QrSync
[![Travis](https://img.shields.io/travis/crisidev/qrsync?style=for-the-badge)](https://travis-ci.org/github/crisidev/qrsync)
[![Crates.io](https://img.shields.io/crates/v/qrsync?style=for-the-badge)](https://crates.io/crates/qrsync)
[![Docs.rs](https://img.shields.io/badge/docs.rs-rustdoc-green?style=for-the-badge)](https://docs.rs/crate/qrsync)
[![Crates.io](https://img.shields.io/crates/d/qrsync?style=for-the-badge)](https://crates.io/crates/qrsync)
[![License](https://img.shields.io/badge/license-MIT-blue?style=for-the-badge)](https://github.com/crisidev/qrsync/blob/master/LICENSE)

Utility to copy files over WiFi to/from mobile devices inside a terminal. 

- [Install](#install)
- [Rust version](#rust-version)
- [Platforms support](#platforms-support)
- [Operational modes](#operational-modes)
- [IPv6 support](#ipv6-support)
- [Command line options](#command-line-options)
- [Acknowledgement](#acknowledgement)
- [License](#license)

When I built QrSync, it was only meant to send files from a terminal to a mobile device, then I
found the amazing [qrcp](https://github.com/claudiodangelis/qrcp) and I took some ideas from it and 
implemented also the possibility to copy file from the mobile device to the computer running QrSync.

### Install
Travis-CI releases [binaries](https://github.com/crisidev/qrsync/releases) for various architectures when a new tag is pushed:
* x84-64 Linux GNU
* x86-64 Linux Musl
* x86-64 Darwin
* x86-64 Windows
* aarch64 Linux GNU
* aarch64 Linux Musl
* arm Linux GNU
* armv7 Linux GNU

Alternatively you can install the latest tag directly from [crates.io](https://crates.io/crates/qrsync):
```sh
❯❯❯ cargo install qrsync
```

### Rust version
QrSync depends on Rocket, which only builds against nightly >= 1.42.

### Platforms support
QrSync has been tested on Linux and MacOSX. 

It currently also build against Windows, but it has not being tested. On \*nix it uses [pnet](https://github.com/libpnet/libpnet) to auto discover the primary interface and its IP address and bind against it. Pnet have a some complex dependencies to build against Windows (see [here](https://github.com/libpnet/libpnet#windows) for more info), so on this platform QrSync makes the `--ip-address` command-line option mandatory and `pnet` is not built at all. 

### Operational modes
QrSync can run in two mode, depending on command line options:
* **Send mode:** this mode is selected when a file is passed to the command line. QrSync will
generate a QR code on the terminal and start the HTTP server in send mode.
    Example:
    ```sh
    ❯❯❯ qrsync my_document.pdf
     INFO  qrsync::http > Send mode enabled for file /home/bigo/my_document.pdf
     INFO  qrsync::http > Scan this QR code with a QR code reader app to open the URL http://192.168.1.11:5566/Q2FyZ28udG9tbA
    ```
* **Receive mode:** this mode is selected if no file is passed to the command line. QrSync will
generate a QR code on the terminal and start the HTTP server in receive mode from the current
folder. A specific folder to save received files can be specified with --root-dir command line
option.
    Example:
    ```sh
    ❯❯❯ qrsync
     INFO  qrsync::http > Receive mode enabled inside directory /home/bigo
     INFO  qrsync::http > Scan this QR code with a QR code reader app to open the URL http://192.168.1.11:5566/receive
    ```

### IPv6 support
QrSync tries to guess which interface to use and which address to bind on the selected interface. In case you want to use IPv6, ensure you have a valid non link-local address and specify `--ipv6` command line argument. Remember, the IP address can be always overridden using `--ip-address` command line argument.

### Command line options
```sh
USAGE:
    qrsync [FLAGS] [OPTIONS] [filename]

ARGS:
    <filename>    File to be send to the mobile device

FLAGS:
    -d, --debug           Enable QrSync debug
    -h, --help            Prints help information
    -6, --ipv6            Prefer IPv6 over IPv4
    -l, --light-term      Draw QR in a terminal with light background
    -D, --rocket-debug    Enable Rocket framework debug
    -V, --version         Prints version information

OPTIONS:
    -i, --ip-address <ip-address>    IP address to bind the HTTP server to. Default to primary interface
    -p, --port <port>                Port to bind the HTTP server to [default: 5566]
    -r, --root-dir <root-dir>        Root directory to store files in receive mode
    -w, --workers <workers>          Number of rocket workers [default: 1]
```

### Acknowledgement
* [qrcp](https://github.com/claudiodangelis/qrcp): I took many ideas from this amazing project
and "stole" most of the HTML Bootstrap based UI.
* [rocket](https://rocket.rs/): A great HTTP framework for Rust, very expandable and simple to
use.
* [qr2term](https://docs.rs/qr2term/): Terminal based QR rendering library.
* [clap](https://clap.rs/): Oh man, where do I start telling how much do I love Clap?

### License
See [LICENSE](https://github.com/crisidev/qrsync/blob/master/LICENSE) file.
