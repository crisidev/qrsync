//! Rocket routes definitions.

use std::path::{Path, PathBuf};

use rocket::http::ContentType;
use rocket::response::content::{Css, Html, Plain};
use rocket::response::{NamedFile, Redirect};
use rocket::response::{Responder, Result as RocketResult};
use rocket::{Data, Request, State};
use rocket_multipart_form_data::{
    mime, FileField, MultipartFormData, MultipartFormDataField, MultipartFormDataOptions,
    Repetition,
};

use crate::utils::{copy_file, sanitize_file_name};

const POST_HTML: &str = include_str!("templates/post.html");
const DONE_HTML: &str = include_str!("templates/done.html");
const ERROR_HTML: &str = include_str!("templates/error.html");
const BOOTSTRAP_CSS: &str = include_str!("templates/bootstrap.min.css");
const BOOTSTRAP_CSS_MAP: &str = include_str!("templates/bootstrap.min.css.map");

/// Request context structure, passed between Rocket handler to share state.
pub struct RequestCtx {
    filename: Option<String>,
    root_dir: PathBuf,
}

impl RequestCtx {
    pub fn new(filename: Option<String>, root_dir: &PathBuf) -> Self {
        RequestCtx {
            filename,
            root_dir: root_dir.to_path_buf(),
        }
    }
}

/// Serve GET /<file_name> URL returning the file served from Rocket.
#[get("/<file_name>")]
pub fn get_send(file_name: String, state: State<RequestCtx>) -> Result<NamedFile, Redirect> {
    match state.filename.as_ref() {
        Some(filename) => {
            if file_name == sanitize_file_name(filename) {
                Ok(NamedFile::open(Path::new(filename)).unwrap())
            } else {
                Err(Redirect::found("/error"))
            }
        }
        None => Err(Redirect::found("/error")),
    }
}

/// Serve GET /receive URL where the user can input files and text to receive.
#[get("/receive")]
pub fn get_receive(_state: State<RequestCtx>) -> Html<String> {
    Html(POST_HTML.to_string())
}

/// Serve GET /done URL where we redirect upon success.
#[get("/receive_done")]
pub fn get_done(_state: State<RequestCtx>) -> Html<String> {
    Html(DONE_HTML.to_string())
}

/// Serve GET /error URL where we redirect upon errors,
#[get("/error")]
pub fn get_error(_state: State<RequestCtx>) -> Html<String> {
    Html(ERROR_HTML.to_string())
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
                        copy_file(file_name, content_type, &file.path, &file_path);
                    }
                }
            }
            Redirect::found("/receive_done")
        }
        Err(e) => {
            error!("Unable to parse multipart form data: {}", e);
            Redirect::found("/error")
        }
    }
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
