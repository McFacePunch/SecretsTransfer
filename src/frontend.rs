use std::sync::Arc;
use std::convert::Infallible;

use crate::{api, config};

use askama::Template;
//use askama::Html as AskamaHtml;

use axum::{
    body::Bytes, http::{header, StatusCode, Uri}, response::{Html, IntoResponse}, Extension
};
use axum::{
    body::Body, //debug_handler, //todo use handler
    response::Response
};

pub async fn favicon() -> Result<Response<Body>, Infallible> {
    // TODO: add this path to the config?
    let favicon_bytes = include_bytes!("../images/favicon_io/favicon.ico");

    // Serve the embedded bytes directly
    let response = Response::builder()
        .header("Content-Type", "image/x-icon")
        .body(Body::from(favicon_bytes.as_ref()))
        .unwrap();

    Ok(response)
}

pub async fn styles_handler(uri: Uri) -> impl IntoResponse {
    let path = uri.path();
    
    let (content_type, css) = if path == "/static/style.css" {
        ("text/css", include_bytes!("../templates/static/tailwind.min.css").as_ref())
    
    } else if path == "/static/all.min.css" {
        ("text/css", include_bytes!("../templates/static/all.min.css").as_ref())
   
    } else if path == "/webfonts/fa-solid-900.ttf" {
        ("font/ttf", include_bytes!("../templates/static/fa-solid-900.ttf").as_ref())
   
    } else if path == "/webfonts/fa-solid-900.woff2" {
        ("font/woff2", include_bytes!("../templates/static/fa-solid-900.woff2").as_ref())
   
    } else {
        return (
            StatusCode::NOT_FOUND, // todo, use standard 404 or update standard 404 for uniformity
            [
                (header::CONTENT_TYPE, "text/plain"),
            ],
            Bytes::from_static(b"Not Found"),
        );
    };

    (
        StatusCode::OK,
        [
            (header::CONTENT_TYPE, content_type),
            //(header::CACHE_CONTROL, "public, max-age=31536000"), // Cache for 1 year
        ],
        Bytes::from_static(css),
    )
}

pub async fn image_handler(uri: Uri) -> impl IntoResponse {
    let path = uri.path();

    let (content_type, image_bytes) = match path {
        // In case more images are needed
        "/images/hero.png" => ("image/png", include_bytes!("../templates/static/images/hero-background.png").as_ref()),
        //"/images/hero.jpg" => ("image/jpeg", include_bytes!("../templates/static/images/hero.jpg").as_ref()),
        //"/images/icon.svg" => ("image/svg+xml", include_bytes!("../templates/static/images/icon.svg").as_ref()),

        _ => return (StatusCode::NOT_FOUND, "404, Not Found").into_response(),  // TODO use the api::not_found to be uniform in the future
    };

    (
        StatusCode::OK,
        [
            (header::CONTENT_TYPE, content_type),
            //(header::CACHE_CONTROL, "public, max-age=31536000"), // TODO set other headers like cache
        ],
        Bytes::from_static(image_bytes),
    ).into_response()
}


#[derive(Template)]
#[template(path = "index.html")] 
pub struct HomePage {
    title: String,
    login_enabled: bool,
}

pub async fn root_page_handler(
    Extension(ref config): Extension<Arc<config::Config>>
) -> impl IntoResponse {
    let html = HomePage { 
        title: "Home".to_string(),
        login_enabled: config.users_enabled }
        .render()
        .unwrap();
    Html(html)
}


#[derive(Template)]
#[template(path = "password_generator.html")]
struct PasswordGeneratorTemplate{
    title: String,
    login_enabled: bool,
}

pub async fn password_handler(
    Extension(ref config): Extension<Arc<config::Config>>
) -> impl IntoResponse {
    let template = PasswordGeneratorTemplate { 
        title:           "Password Generator".to_string(),
        login_enabled:   config.users_enabled, 
    };
    Html(template.render().unwrap())
}


#[derive(Template)]
#[template(path = "secret_form.html")] 
pub struct SecretFormTemplate {
    pub title: String,
    pub login_enabled: bool,
    pub result: Option<String>,
}

pub async fn secret_form_handler(
    Extension(ref config): Extension<Arc<config::Config>>,
) -> impl IntoResponse {
    Html(SecretFormTemplate { 
        title:           "Secrets".to_string(),
        login_enabled:   config.users_enabled,
        result: None}.render().unwrap()) // Provide an empty result for initial load
}


#[derive(Template)]
#[template(path = "about.html")] 
pub struct AboutTemplate {
    pub title: String,
    pub login_enabled: bool,
}

pub async fn about_handler(
    Extension(ref config): Extension<Arc<config::Config>>,
) -> impl IntoResponse {
    Html(AboutTemplate { 
        title:           "About".to_string(),
        login_enabled:   config.users_enabled,
        }.render().unwrap())
}


#[derive(Template)]
#[template(path = "upload.html")] 
pub struct UploadTemplate {
    pub title: String,
    pub login_enabled: bool,
}

pub async fn upload_handler(
    Extension(ref config): Extension<Arc<config::Config>>,
) -> impl IntoResponse {
    Html(UploadTemplate { 
        title:           "Upload".to_string(),
        login_enabled:   config.users_enabled,
        }.render().unwrap())
}


#[derive(Template)]
#[template(path = "download.html")] 
pub struct DownloadTemplate {
    pub title: String,
    pub login_enabled: bool,
}

pub async fn download_handler(
    Extension(ref config): Extension<Arc<config::Config>>,
) -> impl IntoResponse {
    Html(DownloadTemplate { 
        title:           "Download".to_string(),
        login_enabled:   config.users_enabled,
        }.render().unwrap())
}
