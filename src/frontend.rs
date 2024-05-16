use std::sync::Arc;

use crate::config;

use askama::Template;
//use askama::Html as AskamaHtml;

use axum::{
    body::Bytes, http::{header, StatusCode, Uri}, response::{Html, IntoResponse}, Extension
};

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



/* 
PoC code for router setup

// Axum route setup (combined)
let app = Router::new()
    .route(
        "/secrets/",
        post(test_store_secret_post)
            .get(|| async { (StatusCode::OK, Html(SecretFormTemplate {}.render().unwrap())) }), // Serve the HTML form
    )
    .layer(TraceLayer::new_for_http()); // Optional tracing for debugging
 */