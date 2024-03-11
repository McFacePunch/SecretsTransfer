//use axum::{handler::get, handler::post, Router, response::IntoResponse, http::StatusCode, routing::fallthrough};

// TODO: create a middleware to check if the user is AuthN/AuthZ'd
// TODO: implement dual token generation (lookup+crypto tokens) and secret storage in Redis 
// TODOL Encrypt secret at rest

use axum::{
    body::Body,
    http::StatusCode,
    response::{IntoResponse, Redirect, Response},
};
use axum::http::header::HeaderMap;
use axum::extract::ConnectInfo;
use axum::extract::Form;
use axum::response::Html;
use axum::extract::Extension;

use redis::{Commands, Connection, RedisError};

use serde::Deserialize;

use tokio::fs::read;

//use http_body_util::BodyExt;

use std::{convert::Infallible, path::PathBuf};
use std::net::{IpAddr, SocketAddr};
use std::fmt::Write; 

use uuid::Uuid;

//use crate::redis_client;
//use redis::aio::Connection;
extern crate redis;

//mod redis_client;


#[derive(Deserialize)]
pub struct SignupForm {
    email: String,
    password: String,
}

pub async fn favicon() -> Result<Response<Body>, Infallible> {
    // TODO: add this to the config
    // TODO check for file existence and error?
    let favicon_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("./images/favicon_io/favicon.ico");
    let bytes = match read(favicon_path).await {
        Ok(bytes) => bytes,
        Err(_) => return Ok(Response::builder().status(StatusCode::NOT_FOUND).body(Body::empty()).unwrap()),
    };

    let response = Response::builder()
        .header("Content-Type", "image/x-icon")
        .body(Body::from(bytes))
        .unwrap();

    Ok(response)
}

pub async fn root_handler() -> impl IntoResponse {
    (StatusCode::OK, "Hellow World!").into_response()
}

pub async fn status_handler() -> impl IntoResponse {
    (StatusCode::OK, "Service is running").into_response()
}

pub async fn login_handler() -> impl IntoResponse {
    (StatusCode::OK, "Login Page").into_response()
}

pub async fn logout_handler() -> impl IntoResponse {
    (StatusCode::OK, "Logout Page").into_response()
}

pub async fn redirect_to_login() -> Redirect {
    Redirect::temporary("login")
}

pub async fn create_secret_url(Extension(redis_connection): Extension<Connection>) -> impl IntoResponse {
    // Use the connection 'conn' to store secrets in Redis
    //let conn = &redis_connection;

    let secret_uuid = uuid::Uuid::new_v4().to_string();
    // send uuid as key and secret as value

    //generate URL with UUID
    let base_url = "http://localhost:8443/secret/";
    let secret_url = format!("{}{}", base_url, secret_uuid);

    //redis_client::set_value_with_retries(redis_connection, secret_uuid, "secret").await.unwrap();

    //(StatusCode::OK, secret_url).into_response()
   (StatusCode::CREATED, "Secret created successfully")  //temp
}

// pub async fn create_secret_url(Extension(shared_conn): Extension<Arc<Connection>>) -> String {
pub async fn retrieve_secret_url(Extension(redis_connection): Extension<Connection>) -> impl IntoResponse {
    //let mut value = redis_client::get_value_with_retries(redis_connection, "test_key").await.unwrap();
    (StatusCode::CREATED, "Secret created found")  //temp
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

pub async fn signup_get_handler() -> impl IntoResponse {
    let html =r#"
        <!DOCTYPE html>
        <html>
        <head>
            <title>Signup Page</title>
        </head>
        <body>
            <h1>Signup</h1>
            <form id="signup-form" method="POST">
                <label for="email">Email:</label><br>
                <input type="email" id="email" name="email" required><br><br>
                <label for="password">Password:</label><br>
                <input type="password" id="password" name="password" required><br><br>
                <button type="submit">Sign Up</button>
            </form>

            <script>
                const signupForm = document.getElementById('signup-form');
                signupForm.addEventListener('submit', async (event) => {
                    event.preventDefault(); // Prevent default form submission

                    const formData = new FormData(signupForm);
                    const response = await fetch('/signup', {
                        method: 'POST',
                        body: formData
                        headers: {
                            'Content-Type': 'application/x-www-form-urlencoded' 
                        }
                    });

                    if (response.ok) {
                        alert('Signup successful!'); 
                        // Optionally redirect to another page
                    } else {
                        alert('Signup failed. Please try again.');
                    }
                });
            </script>-
        </body>
        </html>
    "#;

    //(StatusCode::OK, Html(html)).into_response();
    Html(html) // returns a 200 as well
}

pub async fn signup_post_handler(Form(signup_data): Form<SignupForm>) -> StatusCode {
    // 1. Validate the email and password (format, uniqueness, etc.)
    // 2. Hash the password securely (never store plain text passwords)
    // 3. Store the user data in a database
    tracing::info!("Received signup request for user: {}:{}", signup_data.email, signup_data.password);

    StatusCode::CREATED
} 

pub async fn not_found() -> impl IntoResponse {
    (StatusCode::NOT_FOUND, "404, Not Found").into_response()
}

/* // TODO: implement this to do fault injection and test error handling middleware
pub async fn trigger_error() -> Result<impl IntoResponse, std::convert::Infallible> {
    Err("This is a forced error".to_string())
}*/
