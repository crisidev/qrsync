//! QR code and HTTP worker handling.

use std::fs;
use std::net::IpAddr;
use std::path::PathBuf;

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
use crate::utils::{sanitize_file_name, ResultOrError};

/// Main structure implementing the workflow if sending and receving files between devices.
/// It fetches the main IP address, generates the QR code, configures and runs the Rocket worker.
pub struct QrSyncHttp {
    ip_address: String,
    port: u16,
    filename: Option<String>,
    root_dir: PathBuf,
    workers: u16,
    light_term: bool,
}

impl QrSyncHttp {
    pub fn new(
        ip_address: Option<String>,
        port: u16,
        filename: Option<String>,
        root_dir: PathBuf,
        workers: u16,
        light_term: bool,
    ) -> ResultOrError<Self> {
        let ip_address = QrSyncHttp::find_public_ip(ip_address)?;
        Ok(QrSyncHttp {
            ip_address: ip_address.to_string(),
            port,
            filename,
            root_dir,
            workers,
            light_term,
        })
    }

    /// Find the public IP by looping over all the available interfaces and finding a public
    /// routable interface with an IP address which can be reached from the outside.
    fn find_public_ip(ip_address: Option<String>) -> ResultOrError<IpAddr> {
        if ip_address.is_some() {
            return Ok(ip_address.unwrap().parse()?);
        }
        let all_interfaces = datalink::interfaces();
        let default_interface = all_interfaces
            .iter()
            .find(|e| e.is_up() && !e.is_loopback() && !e.ips.is_empty());

        match default_interface {
            Some(interface) => {
                if !interface.ips.is_empty() {
                    let ipaddr = interface.ips[0].ip();
                    debug!(
                        "Found IP address {} for interface {}",
                        ipaddr, interface.name
                    );
                    Ok(ipaddr)
                } else {
                    Err(QrSyncError::new(
                        &format!("IP list for interface {} is empty", interface.name),
                        Some("ip-discovery"),
                    ))
                }
            }
            None => Err(QrSyncError::new(
                "Unable to find default interface",
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
                fs::canonicalize(filename)?.to_string_lossy()
            );
            url = format!(
                "http://{}:{}/{}",
                self.ip_address,
                self.port,
                sanitize_file_name(filename)
            );
        } else {
            info!(
                "Receive mode enabled inside directory {}",
                fs::canonicalize(&self.root_dir)?.to_string_lossy()
            );
            url = format!("http://{}:{}/receive", self.ip_address, self.port);
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
            .address(&self.ip_address)
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
