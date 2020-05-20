//! QR code and HTTP worker handling.

use std::fs;
use std::net::IpAddr;
use std::path::PathBuf;

#[cfg(target_family = "unix")]
use pnet::datalink;
use qr2term::{qr, render};
use rocket::config::{Config, Environment};
use rocket::Catcher;

use crate::error::QrSyncError;
use crate::routes::{
    bad_request, static_rocket_route_info_for_get_done, static_rocket_route_info_for_get_error,
    static_rocket_route_info_for_get_receive, static_rocket_route_info_for_get_send,
    static_rocket_route_info_for_post_receive, static_rocket_route_info_for_slash,
    static_rocket_route_info_for_static_bootstrap_css,
    static_rocket_route_info_for_static_bootstrap_css_map,
    static_rocket_route_info_for_static_favicon, RequestCtx,
};
use crate::ResultOrError;

/// Main structure implementing the workflow if sending and receving files between devices.
/// It fetches the main IP address, generates the QR code, configures and runs the Rocket worker.
#[allow(dead_code)]
pub struct QrSyncHttp {
    ip_address: IpAddr,
    port: u16,
    filename: Option<String>,
    root_dir: PathBuf,
    workers: u16,
    light_term: bool,
    ipv6: bool,
}

impl QrSyncHttp {
    pub fn new(
        ip_address: Option<String>,
        port: u16,
        filename: Option<String>,
        root_dir: PathBuf,
        workers: u16,
        light_term: bool,
        ipv6: bool,
    ) -> ResultOrError<Self> {
        let mut qrsync = QrSyncHttp {
            ip_address: "127.0.0.1".parse()?,
            port,
            filename,
            root_dir,
            workers,
            light_term,
            ipv6,
        };
        qrsync.find_public_ip(ip_address)?;
        Ok(qrsync)
    }

    /// Find the public IP by looping over all the available interfaces and finding a public
    /// routable interface with an IP address which can be reached from the outside.
    /// This method currently works only on *nix.
    #[cfg(target_family = "unix")]
    fn find_public_ip(&mut self, ip_address: Option<String>) -> ResultOrError<()> {
        if ip_address.is_some() {
            self.ip_address = ip_address.unwrap().parse()?;
            return Ok(());
        }
        let all_interfaces = datalink::interfaces();
        let default_interface = all_interfaces
            .iter()
            .find(|e| e.is_up() && !e.is_loopback() && !e.ips.is_empty());

        match default_interface {
            Some(interface) => {
                for ip in interface.ips.iter() {
                    if self.ipv6 {
                        if ip.is_ipv6() && ip.ip().is_global() {
                            self.ip_address = ip.ip();
                            break;
                        }
                    } else if ip.is_ipv4() {
                        self.ip_address = ip.ip();
                        break;
                    }
                }
                if !self.ip_address.is_loopback() {
                    debug!(
                        "Found IP address {} for interface {}",
                        self.ip_address, interface.name
                    );
                    Ok(())
                } else {
                    Err(QrSyncError::new(
                        "Unable to find a valid IP address to bind with. See --ip-address option to specify the IP address to use", 
                        Some("ip-discovery")
                    ))
                }
            },
            None => Err(QrSyncError::new(
                "Unable to find default interface. See --ip-address option to specify the IP address to use",
                Some("ip-discovery"),
            ))
        }
    }

    /// To have IP address autodiscovery on windows, the pnet crate have many dependencies, so we
    /// make things easier for now by requiring the --ip-address command line option on this
    /// platform.
    /// This method currently works only on windows.
    #[cfg(target_family = "windows")]
    fn find_public_ip(&mut self, ip_address: Option<String>) -> ResultOrError<()> {
        match ip_address {
            Some(ip_address) => {
                self.ip_address = ip_address.parse()?;
                Ok(())
            }
            None => Err(QrSyncError::new(
                "On windows the command-line option --ip-address is mandatory",
                Some("ip-discovery"),
            )),
        }
    }

    /// Print the QR code to stdout on the terminal and generates white based QRs on dark terminals
    /// and black based QRs on light terminals.
    fn print_qr_code(&self, data: &str) -> ResultOrError<()> {
        let mut matrix = qr::Qr::from(data)?.to_matrix();
        if self.light_term {
            matrix.surround(2, render::QrDark);
        } else {
            matrix.surround(2, render::QrLight);
        }
        render::Renderer::default().print_stdout(&matrix);
        Ok(())
    }

    /// Generates the QR code based on the mode QrSync is started, giving the user a different URL
    /// in case we are expecting the mobile device to send to receive the file.
    fn generate_qr_code(&self) -> ResultOrError<()> {
        let url: String;
        if self.filename.is_some() {
            let filename = self.filename.as_ref().unwrap();
            info!(
                "Send mode enabled for file {}",
                fs::canonicalize(filename)?.display()
            );
            url = format!(
                "http://{}:{}/{}",
                self.ip_address.to_string(),
                self.port,
                base64::encode_config(filename, base64::URL_SAFE_NO_PAD)
            );
        } else {
            info!(
                "Receive mode enabled inside directory {}",
                fs::canonicalize(&self.root_dir)?.display()
            );
            url = format!(
                "http://{}:{}/receive",
                self.ip_address.to_string(),
                self.port
            );
        };
        info!(
            "Scan this QR code with a QR code reader app to open the URL {}",
            url
        );
        self.print_qr_code(&url)?;
        Ok(())
    }

    /// Build a list of Rocket::Catcher for any HTTP error Rocket supports to allow presenting a
    /// nice page to the user.
    fn build_error_catchers(&self) -> Vec<Catcher> {
        let codes = vec![
            400, 401, 402, 403, 404, 405, 406, 407, 408, 409, 410, 411, 412, 413, 414, 415, 416,
            417, 418, 421, 426, 428, 429, 431, 451, 500, 501, 503, 511,
        ];
        let mut catchers: Vec<Catcher> = Vec::new();
        for code in codes.iter() {
            catchers.push(Catcher::new(*code, bad_request));
        }
        catchers
    }

    /// Configure rocket, print the QR code and run the HTTP worker.
    pub fn run(&self) -> ResultOrError<()> {
        let config = Config::build(Environment::Production)
            .address(&self.ip_address.to_string())
            .port(self.port)
            .workers(self.workers)
            .finalize()?;
        let app = rocket::custom(config);
        self.generate_qr_code()?;
        let error = app
            .mount(
                "/",
                routes![
                    slash,
                    get_error,
                    get_send,
                    get_receive,
                    get_done,
                    post_receive,
                    static_bootstrap_css,
                    static_bootstrap_css_map,
                    static_favicon,
                ],
            )
            .register(self.build_error_catchers())
            .manage(RequestCtx::new(self.filename.clone(), &self.root_dir))
            .launch();

        Err(QrSyncError::new(
            &format!("Launch failed! Error: {}", error),
            Some("rocket-launch"),
        ))
    }
}
