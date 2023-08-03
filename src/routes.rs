//! Axum routes definitions.

use std::path::{Path, PathBuf};
use std::str;
use std::sync::Arc;

use axum::body::{Bytes, Full};
use axum::extract::{Extension, Multipart, Path as AxumPath};
use axum::http::{header, StatusCode};
use axum::response::{Html, IntoResponse, Redirect, Response};
use base64::{engine::general_purpose, Engine as _};
use tokio::fs::File;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

use crate::error::QrSyncError;
use crate::QrSyncResult;

const POST_HTML: &str = include_str!("templates/post.html");
const DONE_HTML: &str = include_str!("templates/done.html");
const ERROR_HTML: &str = include_str!("templates/error.html");
const BOOTSTRAP_CSS: &str = include_str!("templates/bootstrap.min.css");
const BOOTSTRAP_CSS_MAP: &str = include_str!("templates/bootstrap.min.css.map");

/// Request context structure, passed between Axum handlers to share state.
pub(crate) struct State {
    file_name: Option<String>,
    root_dir: PathBuf,
}

impl State {
    pub(crate) fn new(file_name: Option<String>, root_dir: &Path) -> Self {
        State {
            file_name,
            root_dir: root_dir.to_path_buf(),
        }
    }

    async fn download_file(&self, file_name: &str) -> QrSyncResult<Vec<u8>> {
        match self.file_name.as_ref() {
            Some(stored_filename) => {
                let encoded_file_name = general_purpose::URL_SAFE_NO_PAD.decode(&file_name)?;
                let decoded_file_name = str::from_utf8(&encoded_file_name)?;
                if stored_filename == decoded_file_name {
                    let file_path = self.root_dir.join(stored_filename);
                    let mut buffer = Vec::new();
                    let mut f = File::open(file_path).await?;
                    f.read_to_end(&mut buffer).await?;
                    Ok(buffer)
                } else {
                    tracing::error!(
                        "Requested file {} differs from served one {}",
                        decoded_file_name,
                        stored_filename
                    );
                    Err(QrSyncError::Error("Requested file differs from served one".into()))
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
    async fn copy_file(&self, content_type: &str, src: Bytes, dst: &Path) {
        match File::create(dst).await {
            Ok(mut f) => match f.write_all(&src).await {
                Ok(_) => tracing::info!(
                    "Received file with content-type {} stored in {}",
                    content_type,
                    dst.display()
                ),
                Err(e) => tracing::error!("Unable to store file {:?} to {}: {}", self.file_name, dst.display(), e),
            },
            Err(e) => tracing::error!("Unable to store file {:?} to {}: {}", self.file_name, dst.display(), e),
        }
    }
}

pub(crate) async fn get_send(AxumPath(file_name): AxumPath<String>, state: Extension<Arc<State>>) -> impl IntoResponse {
    match state.download_file(&file_name).await {
        Ok(data) => {
            let decoded_file_name = general_purpose::URL_SAFE_NO_PAD
                .decode(&file_name)
                .map_err(|e| {
                    let e: QrSyncError = e.into();
                    e.into_response()
                })
                .unwrap();
            let decoded_file_name = str::from_utf8(&decoded_file_name)
                .map_err(|e| {
                    let e: QrSyncError = e.into();
                    e.into_response()
                })
                .unwrap();
            Ok(Response::builder()
                .status(StatusCode::OK)
                .header(
                    header::CONTENT_DISPOSITION,
                    format!("attachment; filename=\"{}\"", decoded_file_name),
                )
                .body(Full::from(data))
                .unwrap())
        }
        Err(_) => Err(Redirect::to("/error")),
    }
}

/// Serve POST /receive URL parsing the multipart form. This way multiple files with different
/// names can be received in a single session.
pub(crate) async fn post_receive(state: Extension<Arc<State>>, mut multipart: Multipart) -> impl IntoResponse {
    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| {
            let e: QrSyncError = e.into();
            e.into_response()
        })
        .unwrap()
    {
        let content_type = field.content_type().unwrap_or("text/plain").to_string();
        if let Some(file_name) = field.file_name() {
            if !file_name.is_empty() {
                let file_path = state.root_dir.join(file_name);
                state
                    .copy_file(
                        &content_type,
                        field
                            .bytes()
                            .await
                            .map_err(|e| {
                                let e: QrSyncError = e.into();
                                e.into_response()
                            })
                            .unwrap(),
                        &file_path,
                    )
                    .await;
            }
        }
    }
    Redirect::to("/receive_done")
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
pub(crate) async fn static_bootstrap_css() -> impl IntoResponse {
    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "text/css")
        .body(Full::from(BOOTSTRAP_CSS.to_string()))
        .unwrap()
}

/// Serve Bootstrap minimized CSS map as static file.
pub(crate) async fn static_bootstrap_css_map() -> impl IntoResponse {
    BOOTSTRAP_CSS_MAP.to_string()
}

/// Serve a fake favicon to avoid getting errors if the favicon is requested.
pub(crate) async fn static_favicon() -> impl IntoResponse {
    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "image/webp")
        .body(Full::from("hi".to_string()))
        .unwrap()
}

/// Rickroll curious cats :)
pub(crate) async fn slash() -> impl IntoResponse {
    Redirect::permanent("https://www.youtube.com/watch?v=oHg5SJYRHA0")
}

/// Catch all for HTTP errors.
pub(crate) async fn bad_request() -> impl IntoResponse {
    (StatusCode::IM_A_TEAPOT, Html(ERROR_HTML.to_string()))
}
