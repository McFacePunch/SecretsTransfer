//use axum::{handler::get, handler::post, Router, response::IntoResponse, http::StatusCode, routing::fallthrough};

// TODO: create a middleware to check if the user is AuthN/AuthZ'd
// TODO: implement dual token generation (lookup+crypto tokens) and secret storage in Redis 
// TODOL Encrypt secret at rest

use axum::{
    body::{Body, Bytes},
    extract::Host,
    extract::Request,
    routing::{get, post},
    http::StatusCode,
    response::{IntoResponse, Redirect, Response},
    Router,
    BoxError,
};
use axum::http::header::HeaderMap;
use axum::extract::Query;
use axum::extract::ConnectInfo;

use axum::Error;
//use std::error::Error;

use http_body_util::BodyExt;


use std::net::{IpAddr, SocketAddr};
use std::fmt::Write; 


pub async fn root_handler() -> impl IntoResponse {
    (StatusCode::OK, "Hellow World!").into_response()
}

pub async fn status_handler() -> impl IntoResponse {
    (StatusCode::OK, "Service is running").into_response()
}

#[allow(dead_code)]
pub async fn login_handler() -> impl IntoResponse {
    (StatusCode::OK, "Login Page").into_response()
}

#[allow(dead_code)]
pub async fn logout_handler() -> impl IntoResponse {
    (StatusCode::OK, "Logout Page").into_response()
}

pub async fn redirect_to_login() -> Redirect {
    Redirect::temporary("login")
}

pub async fn create_secret_url() -> impl IntoResponse {
    (StatusCode::OK, "Secret URL").into_response()
}

//expanded connection info example
pub async fn connection_handler(ConnectInfo(remote_addr): ConnectInfo<SocketAddr>) -> impl IntoResponse {
    // Extract client's IP Address
    let client_ip = match remote_addr.ip() {
        IpAddr::V4(ip) => ip.to_string(),
        IpAddr::V6(ip) => ip.to_string(),
    };

    format!("Hello from client: {}", client_ip)
}

// headermap example
#[allow(dead_code)]
pub async fn header_handler(headers: HeaderMap) -> impl IntoResponse {
    // Access a specific header
    //if let Some(content_type) = headers.get("content-type") {
        //format!("Content-Type: {}", content_type.to_str().unwrap_or("unknown"))
    //} else {format!"No Content-Type header found".to_string();}
    let mut output = String::new();

    for (key, value) in headers.iter() {
        let _ = writeln!(&mut output, "{}: {}", key, value.to_str().unwrap_or("invalid value")); 
    }

    output
}

pub async fn not_found() -> impl IntoResponse {
    (StatusCode::NOT_FOUND, "404, Not Found").into_response()
}

/* // TODO: implement this to do fault injection and test error handling middleware
pub async fn trigger_error() -> Result<impl IntoResponse, std::convert::Infallible> {
    Err("This is a forced error".to_string())
}*/


/* fn log_request(request: &Request<Body>) {
    let method = request.method();
    let uri = request.uri();
    let headers = request.headers();

    tracing::info!(
        method = ?method,
        uri = ?uri,
        headers = ?headers,
        "Received a request"
    );
} */
