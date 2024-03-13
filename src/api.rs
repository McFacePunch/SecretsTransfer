// TODO: create a middleware to check if the user is AuthN/AuthZ'd
// TODO: implement dual token generation (lookup+crypto tokens) and secret storage in Redis 
// TODO: Encrypt secret at rest

use axum::{
    body::Body, debug_handler, 
    extract::{ConnectInfo, Extension, Form, Path, Query, Json},
    http::{header::HeaderMap, status, StatusCode},
    response::{Html, IntoResponse, Redirect, Response}
};
use redis::aio::MultiplexedConnection;
use serde::Deserialize;
use tokio::fs::read;
use uuid::Uuid;
use std::{
    convert::Infallible, path::PathBuf, 
    net::{IpAddr, SocketAddr}, fmt::Write
};

use validator::Validate; 

use base64;
//use uuid::Uuid;

use crate::redis_client;

#[derive(Deserialize, Validate)]
struct SecretData {
    #[validate(length(max = 1024, message = "Secret exceeds maximum size of 1KB"))]
    secret: String, // Base64-encoded secret
}

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

#[allow(dead_code)]
pub async fn redirect_to_login() -> Redirect {
    Redirect::temporary("login")
}

#[debug_handler]
pub async fn store_secret(Extension(redis_connection): Extension<MultiplexedConnection>) -> impl IntoResponse {
    // send uuid as key and secret as value

    let secret_uuid = uuid::Uuid::new_v4().to_string();

    //generate URL with UUID
    let base_url = "https://localhost:8443/secrets/retrieve_secret/";
    let secret_url = format!("Secret-test:\n{}{}", base_url, secret_uuid);

    crate::redis_client::set_value_with_retries(&mut redis_connection.clone(), &secret_uuid, &secret_url).await.unwrap();

    (StatusCode::OK, secret_url).into_response()
}

#[debug_handler]
pub async fn store_secret_post(
    Extension(redis_connection): Extension<MultiplexedConnection>,
    Json(payload): Json<SecretData>,
) -> impl IntoResponse {
    // Get the secret from the JSON payload and encode it
    let secret_string = base64::encode(&payload.secret);

    // Generate UUID
    let secret_uuid = uuid::Uuid::new_v4().to_string();

    // Store in Redis
    crate::redis_client::set_value_with_retries(&mut redis_connection.clone(), &secret_uuid, &secret_string)
    .await
    .map_err(|err| {
        // Handle Redis errors
        StatusCode::INTERNAL_SERVER_ERROR
    });

    // Generate Response URL (adjust if needed)
    let base_url = "https://localhost:8443/secrets/retrieve_secret/";
    let secret_url = format!("Secret-test:\n{}{}", base_url, secret_uuid);

    (StatusCode::CREATED, secret_url).into_response()
}


#[debug_handler]
/*pub async fn retrieve_secret(
    Extension(redis_connection): Extension<MultiplexedConnection>,
    Path(uuid): Path<String>,
    //Query(params): Query<HashMap<String, String>>,
    ) -> impl IntoResponse {

    let output = format!("Secret:\n{}\n{}", value,uuid);
    (StatusCode::OK, output).into_response()  //temp
}*/
pub async fn retrieve_secret(
    Extension(redis_connection): Extension<MultiplexedConnection>,
    Path(uuid_str): Path<String>, // Extract the UUID string 
) -> impl IntoResponse {

    // Attempt to parse the UUID
    let uuid = match Uuid::parse_str(&uuid_str) {
        Ok(uuid) => uuid,
        Err(_) => return (StatusCode::BAD_REQUEST, "Invalid UUID format").into_response(),
    };

    // Look up value in Redis
    let value = redis_client::get_value_with_retries(&mut redis_connection.clone(), &uuid.to_string()).await;
    match value {
        Ok(value) => {
            let output = format!("Secret:\n{}\n{}\n", value, uuid);
            
            // Decode Base64
            // TODO: improve with base64::URL_SAFE_NO_PAD and stop using deprecated function
            let decoded_secret = base64::decode(&value); 
            
            match decoded_secret {
                Ok(decoded) => (StatusCode::OK, decoded).into_response(),
                Err(_) => (StatusCode::BAD_REQUEST, "Invalid Base64").into_response(),
            }
        },
        Err(status) => (StatusCode::NOT_FOUND, "Secret not found").into_response()
    }
}

// Catch all handlers

pub async fn not_found() -> impl IntoResponse {
    (StatusCode::NOT_FOUND, "404, Not Found").into_response()
}


// Debug stuff
// expand?
pub async fn connection_handler(ConnectInfo(remote_addr): ConnectInfo<SocketAddr>) -> impl IntoResponse {
    // Extract client's IP Address
    let client_ip = match remote_addr.ip() {
        IpAddr::V4(ip) => ip.to_string(),
        IpAddr::V6(ip) => ip.to_string(),
    };

    format!("Hello from client: {}", client_ip)
}

// Headermap example for debugging
// https://docs.rs/axum/latest/axum/extract/index.html#the-order-of-extractors
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

/* // TODO: implement this to do fault injection and test error handling middleware
pub async fn trigger_error() -> Result<impl IntoResponse, std::convert::Infallible> {
    Err("This is a forced error".to_string())
}*/
