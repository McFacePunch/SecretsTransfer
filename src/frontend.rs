use std::sync::Arc;

use askama::Template;
use axum::{
    body::Bytes, http::{header, StatusCode, Uri}, response::{Html, IntoResponse}, Extension
};

pub async fn styles_handler(uri: Uri) -> impl IntoResponse {
    let path = uri.path();
    
    let (content_type, css) = if path == "/static/style.css" {
        ("text/css", include_bytes!("../templates/tailwind.min.css").as_ref())
    } else if path == "/static/all.min.css" {
        ("text/css", include_bytes!("../templates/all.min.css").as_ref())
    } else if path == "/webfonts/fa-solid-900.ttf" {
        ("font/ttf", include_bytes!("../templates/fa-solid-900.ttf").as_ref())
    } else if path == "/webfonts/fa-solid-900.woff2" {
        ("font/woff2", include_bytes!("../templates/fa-solid-900.woff2").as_ref())
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
    pub login_enabled: bool,
}

// #[derive(Template)]
// #[template(path = "default_page_template.html")]
// struct MainTemplate {
//     login_enabled: bool,
//     content: String,
// }


#[derive(Template)]
#[template(path = "default_page_template.html")]
struct DefaultTemplate<'a> {
    login_enabled: bool,
    content: &'a str,
    strs: &'a str,
}


#[derive(Template)]
#[template(path = "password_generator.html")]//, escape = "none")]
struct PasswordGeneratorTemplate{
    login_enabled: bool,
    content: String,
}

#[derive(Template)]
#[template(path = "secret_form.html")] 
pub struct SecretFormTemplate {
    pub result: Option<String>,
}
/* 
#[derive(Template)]
#[template(path = "about.html")]
pub struct AboutTemplate {
    common_fields: CommonFields
}
 
impl AboutTemplate {
    pub fn new(common_fields: CommonFields) -> Self {
        Self { 
            common_fields
        }
    }
} */
 
/* pub const PATH: &str = "/about";
 
pub async fn route(session: Session) -> AboutTemplate {
    let locale = session.get::<String>("locale").await.unwrap().unwrap_or("en".to_string());
    let title:String = rust_i18n::t!("navigation.about").to_string();
    AboutTemplate::new(CommonFields::new(PATH, &title, locale))
} */

use crate::config;

pub async fn root_page_handler(
    Extension(ref config): Extension<Arc<config::Config>>
) -> impl IntoResponse {
    let html = HomePage { login_enabled: config.users_enabled }
        .render()
        .unwrap();
    Html(html)
}

pub async fn password_page_handler(
    Extension(ref config): Extension<Arc<config::Config>>
) -> impl IntoResponse {
    //let passgen = PasswordGeneratorTemplate.render().unwrap();
    let template = DefaultTemplate { 
        login_enabled:   config.users_enabled, 
        content:         include_str!("../templates/password_generator.html"),
        strs:            include_str!("../templates/password_generator.html"),
    };
    Html(template.render().unwrap())
}


pub async fn secret_form_handler() -> Html<String> {
    Html(SecretFormTemplate { result: None}.render().unwrap()) // Provide an empty result for initial load
}

/* // Modified route handler to handle the POST request
pub async fn test_store_secret_post(
    Extension(db): Extension<database::StorageEnum>,
    Form(secret_data): Form<SecretData>, // Extract form data
) -> impl IntoResponse {
    let secret_uuid = database::get_uuid();

    let base_url = "https://localhost:8443/secrets/retrieve_secret/";
    let secret_url = format!("{}{}", base_url, secret_uuid);

    let out = set_value(&db, &secret_uuid, &secret_url).await;

    match out {
        Ok(()) => {
            tracing::debug!("Secret Stored!: {}", secret_url);
            (StatusCode::OK, secret_url).into_response()
        }
        Err(e) => {
            tracing::error!("Error storing secret: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR).into_response()
        }
    }
} */


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