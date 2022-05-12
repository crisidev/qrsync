//! Rocket routes definitions.

use std::fs;
use std::path::{Path, PathBuf};
use std::str;

use axum::response::{IntoResponse, Html};
use rocket::http::ContentType;
use rocket::response::content::{Css, Html as RocketHtml, Plain};
use rocket::response::{Redirect, Responder, Result as RocketResult};
use rocket::{Data, Request, State};
use rocket_download_response::DownloadResponse;
use rocket_multipart_form_data::mime::Mime;
use rocket_multipart_form_data::{
    mime, FileField, MultipartFormData, MultipartFormDataField, MultipartFormDataOptions, Repetition,
};

use crate::error::QrSyncError;
use crate::ResultOrError;

const POST_HTML: &str = include_str!("templates/post.html");
const DONE_HTML: &str = include_str!("templates/done.html");
const ERROR_HTML: &str = include_str!("templates/error.html");
const BOOTSTRAP_CSS: &str = include_str!("templates/bootstrap.min.css");
const BOOTSTRAP_CSS_MAP: &str = include_str!("templates/bootstrap.min.css.map");

/// Request context structure, passed between Rocket handler to share state.
pub struct RequestCtx {
    file_name: Option<String>,
    root_dir: PathBuf,
}

impl RequestCtx {
    pub fn new(file_name: Option<String>, root_dir: &Path) -> Self {
        RequestCtx {
            file_name,
            root_dir: root_dir.to_path_buf(),
        }
    }

    fn download_file(&self, file_name: String) -> ResultOrError<DownloadResponse> {
        match &self.file_name {
            Some(stored_filename) => {
                let encoded_file_name = base64::decode_config(&file_name, base64::URL_SAFE_NO_PAD)?;
                let decoded_file_name = str::from_utf8(&encoded_file_name)?;
                if stored_filename == decoded_file_name {
                    let file_path = self.root_dir.join(stored_filename);
                    Ok(DownloadResponse::from_file(file_path, Some(decoded_file_name), None))
                } else {
                    tracing::error!(
                        "Requested file {} differs from served one {}",
                        decoded_file_name,
                        stored_filename
                    );
                    Err(QrSyncError::Error("QrSync is not running in send mode".into()))
                }
            }
            None => {
                tracing::error!("QrSync is not running in send mode");
                Err(QrSyncError::Error("QrSync is not running in send mode".into()))
            }
        }
    }

    /// Copy a file from a source to a destination. The file_name and content_type are used to produce
    /// nice errors.
    fn copy_file(&self, content_type: &Mime, src: &Path, dst: &Path) {
        match fs::copy(src, dst) {
            Ok(_) => tracing::info!(
                "Received file with content-type {} stored in {}",
                content_type,
                dst.display()
            ),
            Err(e) => tracing::error!(
                "Unable to store file {} to {}: {}",
                self.file_name.as_ref().unwrap_or(&"unknown-file".to_string()),
                dst.display(),
                e
            ),
        }
    }
}

/// Serve GET /<file_name> URL returning the file served from Rocket.
#[get("/<file_name>")]
pub fn get_send(file_name: String, state: State<RequestCtx>) -> Result<DownloadResponse, Redirect> {
    match state.download_file(file_name) {
        Ok(data) => Ok(data),
        Err(_) => Err(Redirect::found("/error")),
    }
}

/// Serve POST /receive URL parsing the multipart form. This way multiple files with different
/// names can be received in a single session.
#[post("/receive", format = "multipart/form-data", data = "<data>")]
pub fn post_receive(content_type: &ContentType, data: Data, state: State<RequestCtx>) -> Redirect {
    let mut options = MultipartFormDataOptions::new();
    options
        .allowed_fields
        .push(MultipartFormDataField::file("binary-files").repetition(Repetition::infinite()));
    options.allowed_fields.push(
        MultipartFormDataField::file("text-file")
            .content_type_by_string(Some(mime::TEXT_PLAIN))
            .unwrap(),
    );
    match MultipartFormData::parse(content_type, data, options) {
        Ok(multipart_form_data) => {
            let files = multipart_form_data.files.get("binary-files");
            let mut file_vec = Vec::new();
            if let Some(files) = files {
                match files {
                    FileField::Single(data) => file_vec.push(data),
                    FileField::Multiple(datas) => file_vec.extend(datas.iter()),
                }
            }
            let file = multipart_form_data.files.get("text-file");
            if let Some(file) = file {
                match file {
                    FileField::Single(data) => file_vec.push(data),
                    FileField::Multiple(datas) => file_vec.extend(datas.iter()),
                }
            }
            for file in file_vec.iter() {
                let content_type = file.content_type.as_ref().unwrap_or(&mime::TEXT_PLAIN);
                let file_name = file.file_name.as_ref();
                if let Some(file_name) = file_name {
                    if !file_name.is_empty() {
                        let file_path = state.root_dir.join(file_name);
                        state.copy_file(content_type, &file.path, &file_path);
                    }
                }
            }
            Redirect::found("/receive_done")
        }
        Err(e) => {
            tracing::error!("Unable to parse multipart form data: {}", e);
            Redirect::found("/error")
        }
    }
}

/// Serve GET /receive URL where the user can input files and text to receive.
pub(crate) async fn get_receive() -> impl IntoResponse {
    Html(POST_HTML.to_string())
}

/// Serve GET /done URL where we redirect upon success.
pub(crate) async fn get_receive_done() -> impl IntoResponse {
    Html(DONE_HTML.to_string())
}

/// Serve GET /error URL where we redirect upon errors,
pub(crate) async fn get_error() -> impl IntoResponse {
    Html(ERROR_HTML.to_string())
}

/// Serve Bootstrap minimized CSS as static file.
#[get("/static/bootstrap.min.css", format = "text/css")]
pub fn static_bootstrap_css(_state: State<RequestCtx>) -> Css<String> {
    Css(BOOTSTRAP_CSS.to_string())
}

/// Serve Bootstrap minimized CSS map as static file.
#[get("/static/bootstrap.min.css.map", format = "text/plain")]
pub fn static_bootstrap_css_map(_state: State<RequestCtx>) -> Plain<String> {
    Plain(BOOTSTRAP_CSS_MAP.to_string())
}

/// Serve a fake favicon to avoid getting errors from Rocket if the favicon is requested.
#[get("/favicon.ico", format = "image/webp")]
pub fn static_favicon(_state: State<RequestCtx>) -> Plain<String> {
    Plain("hi".to_string())
}

/// Rickroll curious cats :)
#[get("/")]
pub fn slash(_state: State<RequestCtx>) -> Redirect {
    Redirect::found("https://www.youtube.com/watch?v=oHg5SJYRHA0")
}

/// Catch all for HTTP Rocket errors.
pub fn bad_request<'r>(req: &'r Request) -> RocketResult<'r> {
    ERROR_HTML.to_string().respond_to(req)
}
