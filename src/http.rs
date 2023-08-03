//! QR code and HTTP worker handling.

use std::fs;
use std::path::PathBuf;
use std::sync::Arc;

use axum::routing::{get, post};
use axum::{extract::DefaultBodyLimit, Extension, Router};
use base64::{engine::general_purpose, Engine as _};
#[cfg(target_family = "unix")]
use pnet::datalink;
use qr2term::matrix::Matrix;
use qr2term::qr::Qr;
use qr2term::render::{Color, QrDark, QrLight, Renderer};
use tower::ServiceBuilder;
use tower_http::limit::RequestBodyLimitLayer;
use tower_http::trace::TraceLayer;

use crate::routes::*;
use crate::{QrSyncError, QrSyncResult};

/// Main structure implementing the workflow if sending and receving files between devices.
/// It fetches the main IP address, generates the QR code, configures and runs the Rocket worker.
#[derive(Debug)]
pub struct QrSyncHttp {
    ip_address: Option<String>,
    port: u16,
    filename: Option<String>,
    root_dir: PathBuf,
    light_term: bool,
    ipv6: bool,
}

impl QrSyncHttp {
    /// Create a new instance of QrSyncHttp from command line arguments.
    pub fn new(
        ip_address: Option<String>,
        port: u16,
        filename: Option<String>,
        root_dir: PathBuf,
        light_term: bool,
        ipv6: bool,
    ) -> Self {
        QrSyncHttp {
            ip_address,
            port,
            filename,
            root_dir,
            light_term,
            ipv6,
        }
    }

    /// Find the public IP by looping over all the available interfaces and finding a public
    /// routable interface with an IP address which can be reached from the outside.
    /// This method currently works only on *nix.
    #[cfg(target_family = "unix")]
    fn find_public_ip(&self) -> QrSyncResult<String> {
        if self.ip_address.is_some() {
            return Ok(self.ip_address.as_ref().unwrap().to_string());
        }
        let all_interfaces = datalink::interfaces();
        let default_interface = all_interfaces
            .iter()
            .find(|e| e.is_up() && !e.is_loopback() && !e.ips.is_empty());

        let mut ip_address = "127.0.0.1".parse()?;
        match default_interface {
            Some(interface) => {
                for ip in interface.ips.iter() {
                    if self.ipv6 {
                        if ip.is_ipv6() {
                            ip_address = ip.ip();
                            break;
                        }
                    } else if ip.is_ipv4() {
                        ip_address = ip.ip();
                        break;
                    }
                }
                if !ip_address.is_loopback() {
                    tracing::debug!("Found IP address {} for interface {}", ip_address, interface.name);
                    Ok(ip_address.to_string())
                } else {
                    Err(QrSyncError::Error(
                        "Unable to find a valid IP address to bind with. See --ip-address option to specify the IP address to use".into() 
                    ))
                }
            }
            None => Err(QrSyncError::Error(
                "Unable to find default interface. See --ip-address option to specify the IP address to use".into(),
            )),
        }
    }

    /// To have IP address autodiscovery on windows, the pnet crate have many dependencies, so we
    /// make things easier for now by requiring the --ip-address command line option on this
    /// platform.
    /// This method currently works only on windows.
    #[cfg(target_family = "windows")]
    fn find_public_ip(&self) -> QrSyncResult<String> {
        match &self.ip_address {
            Some(ip_address) => Ok(ip_address.to_string()),
            None => Err(QrSyncError::Error(
                "On windows the command-line option --ip-address is mandatory".into(),
            )),
        }
    }

    /// Generates the QR code based on the mode QrSync is started, giving the user a different URL
    /// in case we are expecting the mobile device to send to receive the file.
    fn generate_qr_code_url(&self, ip_address: &str) -> QrSyncResult<String> {
        let url = match self.filename.as_ref() {
            Some(filename) => {
                tracing::info!("Send mode enabled for file {}", fs::canonicalize(filename)?.display());
                format!(
                    "http://{}:{}/{}",
                    ip_address,
                    self.port,
                    general_purpose::URL_SAFE_NO_PAD.encode(filename)
                )
            }
            None => {
                tracing::info!(
                    "Receive mode enabled inside directory {}",
                    fs::canonicalize(&self.root_dir)?.display()
                );
                format!("http://{}:{}/receive", ip_address, self.port)
            }
        };
        tracing::info!("Scan this QR code with a QR code reader app to open the URL {}", url);
        Ok(url)
    }

    /// Print the QR code to stdout on the terminal and generates white based QRs on dark terminals
    /// and black based QRs on light terminals.
    fn generate_qr_code_matrix(&self, data: &str) -> QrSyncResult<Matrix<Color>> {
        let mut matrix = Qr::from(data)?.to_matrix();
        if self.light_term {
            matrix.surround(2, QrDark);
        } else {
            matrix.surround(2, QrLight);
        }
        Ok(matrix)
    }

    fn print_qr_code(&self, ip_address: &str) -> QrSyncResult<()> {
        let url = self.generate_qr_code_url(ip_address)?;
        let qr = self.generate_qr_code_matrix(&url)?;
        Renderer::default().print_stdout(&qr);
        Ok(())
    }

    /// Configure Axum, print the QR code and run the HTTP worker.
    pub async fn run(&self) -> QrSyncResult<()> {
        let app = Router::new()
            .route("/", get(slash))
            .route("/receive", get(get_receive))
            .route("/receive_done", get(get_receive_done))
            .route("/error", get(get_error))
            .route("/static/bootstrap.min.css", get(static_bootstrap_css))
            .route("/static/bootstrap.min.css.map", get(static_bootstrap_css_map))
            .route("/favicon.ico", get(static_favicon))
            .route("/:file_name", get(get_send))
            .route("/receive", post(post_receive))
            .layer(DefaultBodyLimit::disable())
            .layer(RequestBodyLimitLayer::new(250 * 1024 * 1024 * 1024 /* 250Gb */))
            .fallback(bad_request);
        let state = Arc::new(State::new(self.filename.clone(), &self.root_dir));
        let app = app.layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(Extension(state)),
        );
        let ip_address = self.find_public_ip()?;
        self.print_qr_code(&ip_address)?;
        let address = format!("{}:{}", ip_address, self.port).parse()?;
        let server = hyper::Server::bind(&address).serve(app.into_make_service());

        if let Err(err) = server.await {
            tracing::error!("Server error: {}", err);
        }
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use pretty_assertions::{assert_eq, assert_ne};

    #[test]
    fn test_find_public_ip_passing_ip_address() {
        let ip_address = "10.0.0.1";
        let http = QrSyncHttp::new(
            Some(ip_address.to_string()),
            12345,
            Some("a-file".to_string()),
            PathBuf::from("a-dir"),
            false,
            false,
        );
        assert_eq!(http.find_public_ip().unwrap(), ip_address.to_string());
    }

    #[test]
    fn test_find_public_ip_passing_autodetect() {
        let http = QrSyncHttp::new(
            None,
            12345,
            Some("a-file".to_string()),
            PathBuf::from("a-dir"),
            false,
            false,
        );
        assert_ne!(http.find_public_ip().unwrap(), "127.0.0.1".to_string());
    }

    #[test]
    fn test_generate_qr_code_url_send_mode() {
        let ip_address = "10.0.0.1";
        let file_name = "a-file";
        let http = QrSyncHttp::new(
            Some(ip_address.to_string()),
            12345,
            Some(file_name.to_string()),
            PathBuf::from("a-dir"),
            false,
            false,
        );
        let url = http.generate_qr_code_url(ip_address).unwrap();
        assert_eq!(
            format!(
                "http://{}:12345/{}",
                ip_address,
                general_purpose::URL_SAFE_NO_PAD.encode(file_name)
            ),
            url
        );
    }

    #[test]
    fn test_generate_qr_code_url_receive_mode() {
        let ip_address = "10.0.0.1";
        let http = QrSyncHttp::new(
            Some(ip_address.to_string()),
            12345,
            None,
            PathBuf::from("a-dir"),
            false,
            false,
        );
        let url = http.generate_qr_code_url(ip_address).unwrap();
        assert_eq!(format!("http://{}:12345/receive", ip_address,), url);
    }

    #[test]
    fn test_generate_qr_code_matrix_dark() {
        let ip_address = "10.0.0.1";
        let http = QrSyncHttp::new(
            Some(ip_address.to_string()),
            12345,
            None,
            PathBuf::from("a-dir"),
            false,
            false,
        );
        let url = http.generate_qr_code_url(ip_address).unwrap();
        let qr = http.generate_qr_code_matrix(&url).unwrap();
        assert_eq!(qr.pixels().len(), 1089);
        let light_pixels = qr.pixels().iter().filter(|&n| *n == QrLight).count();
        let dark_pixels = qr.pixels().iter().filter(|&n| *n == QrDark).count();
        assert_eq!(light_pixels, 667);
        assert_eq!(dark_pixels, 422);
    }

    #[test]
    fn test_generate_qr_code_matrix_light() {
        let ip_address = "10.0.0.1";
        let http = QrSyncHttp::new(
            Some(ip_address.to_string()),
            12345,
            None,
            PathBuf::from("a-dir"),
            true,
            false,
        );
        let url = http.generate_qr_code_url(ip_address).unwrap();
        let qr = http.generate_qr_code_matrix(&url).unwrap();
        assert_eq!(qr.pixels().len(), 1089);
        let light_pixels = qr.pixels().iter().filter(|&n| *n == QrLight).count();
        let dark_pixels = qr.pixels().iter().filter(|&n| *n == QrDark).count();
        assert_eq!(light_pixels, 419);
        assert_eq!(dark_pixels, 670);
    }

    #[test]
    fn test_print_qr_code() {
        let ip_address = "10.0.0.1";
        let http = QrSyncHttp::new(
            Some(ip_address.to_string()),
            12345,
            None,
            PathBuf::from("a-dir"),
            false,
            false,
        );
        assert_eq!(http.print_qr_code(ip_address).is_ok(), true);
    }
}
