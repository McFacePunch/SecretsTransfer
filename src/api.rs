// TODO: create a middleware to check if the user is AuthN/AuthZ'd
// TODO: implement dual token generation (lookup+crypto tokens) and secret storage in Redis 
// TODO: Encrypt secret at rest
use std::{
    borrow::Borrow, convert::Infallible, error::Error, fmt::Write, net::{IpAddr, SocketAddr}, path::PathBuf, sync::{Arc, Mutex}
};

use axum::{
    body::Body, //debug_handler, //todo use handler
    extract::{ConnectInfo, Extension, Form, Path, Multipart},
    http::{header, header::HeaderMap, StatusCode},
    response::{Html, IntoResponse, Redirect, Response}
};

use askama::Template;
use serde::{Serialize, Deserialize};
use validator::Validate;
use std::str;


use crate::{database::Storage, frontend};
use crate::database;//::{Storage, StorageEnum};

// TODO put the mb counter into config so its easy to adjust. Should work up and down, like using 0.5
const MAX_FILE_SIZE: usize = 5 * 1024 * 1024; // 5MB limit

#[derive(Deserialize, Validate)]
pub struct SecretData {
    #[validate(length(max = 10240, message = "Secret exceeds maximum size of 10KB"))]
    secret: String,
}

#[derive(Serialize, Deserialize)]
struct FileMetadata {
    filename: String,
    data: Vec<u8>, // Store the file data as a binary vector
}

#[derive(Deserialize,Serialize,Validate)]
pub struct SignupForm {
    #[validate(email)]
    pub email: String,
    
    #[validate(length(min = 12))]
    pub password: String,
}

pub async fn status_handler() -> impl IntoResponse {
    (StatusCode::OK, "Service is running").into_response()
}

// TODO implement the login page
pub async fn login_get_handler() -> impl IntoResponse {
    (StatusCode::OK, "Login Page").into_response()
}

// TODO implement this
pub async fn logout_handler() -> impl IntoResponse {
    (StatusCode::OK, "Logout Page").into_response()
}

// TODO make this askama based
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

// TODO Implement this
pub async fn signup_post_handler(Form(signup_data): Form<SignupForm>,
Extension(_db): Extension<Arc<Mutex<database::StorageEnum>>>
) -> StatusCode {
    // 1. Validate the email and password (format, uniqueness, etc.)
    // 2. Hash the password securely (never store plain text passwords)
    // 3. Store the user data in a database

    let form = SignupForm { email: signup_data.email, password: signup_data.password};
    match form.validate() {
        Ok(_) => {
            tracing::info!("Received a valid signup request for user: {}", form.email);

            // TODO
            // if user exists bail with error

            // else create
            // return 201 or redirect to login?
            //redirect_to_login().await;
            return StatusCode::ACCEPTED;
            //return Redirect::temporary("login")
        }
        Err(e) => {
            tracing::debug!("Received a bad signup request for user: {}\n{}", form.email, e);
            return StatusCode::BAD_REQUEST;
        }
    }
} 

async fn get_value(storage: &database::StorageEnum, key: &str) -> Result<Option<String>, Box<dyn Error>> {
    match storage {
        database::StorageEnum::InMemory(map) => map.get(key).await,
        database::StorageEnum::Redis(pool) => pool.get(key).await,
        //StorageEnum::NoSQLDB(conn) => conn.execute("SELECT value FROM table WHERE key = ?", [key]),
        database::StorageEnum::None => Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, "No database available"))),
    }
}

async fn set_value(storage: &database::StorageEnum, key: &str, value: &str) -> Result<(), Box<dyn Error>> {
    match storage {
        database::StorageEnum::InMemory(map) => map.set(key, value).await,
        database::StorageEnum::Redis(pool) => pool.set(key, value).await,
        //StorageEnum::NoSQLDB(conn) => conn.execute("INSERT INTO table (key, value) VALUES (?, ?)", [key, value]),
        database::StorageEnum::None => Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, "No database available"))),
    }
}

pub async fn test_store_secret_post(
    Extension(db): Extension<database::StorageEnum>,
    Form(secret_data): Form<SecretData>, // Extract form data
) -> impl IntoResponse {
    let secret_uuid = database::get_uuid();

    let base_url = "https://localhost:8443/secrets/retrieve_secret/";
    let secret_url = format!("{}{}", base_url, secret_uuid);

    let out = set_value(&db, &secret_uuid, &secret_data.secret).await;

    match out {
        Ok(()) => {
            tracing::info!("Stored secret {}", secret_data.secret); // TOOD debug remove
            let url = Some(secret_url); // Success message
            (StatusCode::OK, Html(frontend::SecretFormTemplate { 
                title: "Secret Form".to_string(),
                login_enabled: false,
                result: url }.render().unwrap())).into_response()
            //(StatusCode::OK, Html(frontend::OldSecretFormTemplate { }.render().unwrap())).into_response()
        }
        Err(e) => {
            tracing::error!("Error storing secret: {}", e);
            //let result = Some(format!("Error storing secret: {}", e)); // Error message
            (StatusCode::INTERNAL_SERVER_ERROR, Html(frontend::SecretFormTemplate { 
                title: "Secret Form".to_string(),
                login_enabled: false,
                result: None }.render().unwrap())).into_response()
            //(StatusCode::INTERNAL_SERVER_ERROR, Html(frontend::OldSecretFormTemplate { }.render().unwrap())).into_response()
        }
    }
}

pub async fn test_retrieve_secret_get(
    Extension(db): Extension<database::StorageEnum>,
    Path(secret_uuid): Path<String>, 
) -> impl IntoResponse {

    let out = get_value(&db, &secret_uuid).await;
    match out {

        Ok(Some(secret_url)) => {
            tracing::debug!("Retrieved secret: {}", secret_url);
            (StatusCode::OK, secret_url).into_response()
        },
        Ok(None) => {
            tracing::debug!("Secret not found");
            (StatusCode::NOT_FOUND, "Secret not found").into_response()
        },
        Err(e) => {
            tracing::error!("Error retrieving secret: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR).into_response()
        }
    }
}

pub async fn file_upload_secret(
    Extension(db): Extension<database::StorageEnum>,
    mut multipart: Multipart, 
) -> Result<Response<Body>, (StatusCode, String)> {
    let mut file_content = Vec::new();
    let mut original_filename = String::new();
    let mut total_size = 0usize;

    // Process the multipart form and read the file
    while let Some(mut field) = multipart.next_field().await.map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            format!("Failed to process multipart field: {}", e),
        )
    })? {
        if field.name() == Some("file") {
            // Capture original filename
            original_filename = field.file_name().unwrap_or_default().to_string();

            // Stream and check file size while reading it
            while let Some(chunk) = field.chunk().await.map_err(|e| {
                (
                    StatusCode::BAD_REQUEST,
                    format!("Failed to read file chunk: {}", e),
                )
            })? {
                total_size += chunk.len();
                if total_size > MAX_FILE_SIZE {
                    return Err((
                        StatusCode::PAYLOAD_TOO_LARGE,
                        "File exceeds the maximum size of 5MB".to_string(),
                    ));
                }
                // Accumulate file data
                file_content.extend_from_slice(&chunk);
            }
        }
    }

    if file_content.is_empty() {
        return Err((StatusCode::BAD_REQUEST, "No file uploaded.".to_string()));
    }

    //let encoded_file_content = encode(&file_content);

    // Generate the UUID and append it to the original filename
    let secret_uuid = database::get_uuid();
    let full_filename = format!("{}_{}", secret_uuid, original_filename);

    // Store both the file data and the full filename in Redis
    let file_metadata = FileMetadata {
        filename: full_filename.clone(),
        data: file_content,
    };

    let serialized_data = serde_json::to_string(&file_metadata).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Error serializing file metadata: {}", e),
        )
    })?;

    // Store the serialized data in Redis
    match set_value(&db, &secret_uuid, serialized_data.as_str()).await {
        Ok(()) => {
            let base_url = "https://localhost:8443/secrets/download_file";
            let secret_url = format!("\nSecret Stored:\n{}\n{}", base_url, secret_uuid);
            tracing::info!("Received file");
            Ok((StatusCode::OK, secret_url).into_response())
        }
        Err(e) => {
            tracing::info!("Error in upload file");
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Error storing secret: {}", e),
            ))
        }
    }
}

pub async fn file_download_secret(
    Extension(db): Extension<database::StorageEnum>,
    Form(secret_data): Form<SecretData>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    // Retrieve the serialized file metadata from Redis
    let serialized_data = get_value(&db, &secret_data.secret).await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Error retrieving secret: {}", e),
        )
    })?;

    // Ensure the content is present
    let serialized_data = serialized_data.ok_or((
        StatusCode::NOT_FOUND,
        "Secret not found".to_string(),
    ))?;

    // Deserialize the data back into FileMetadata
    let file_metadata: FileMetadata = serde_json::from_str(&serialized_data).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Error deserializing file metadata: {}", e),
        )
    })?;

    // Extract the original filename by stripping the UUID from the full filename
    let original_filename = file_metadata
        .filename
        .splitn(2, '_') // Split by the first occurrence of '_'
        .nth(1) // Get the part after the UUID
        .unwrap_or("unknown_filename");

    // Decode the file data if it was stored as base64
    let decoded_content = file_metadata.data;

    // Build the response with the correct headers for file download
    Ok(Response::builder()
        .header("Content-Type", "application/octet-stream")
        .header("Content-Disposition", format!("attachment; filename=\"{}\"", original_filename))
        .body(Body::from(decoded_content))
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Error creating response: {}", e),
            )
        })?)
}


//////////////////
// Wrappers and Utilities
//////////////////
#[allow(dead_code)]
pub async fn redirect_to_login() -> Redirect {
    Redirect::temporary("login")
}

pub async fn not_found() -> impl IntoResponse {
    (StatusCode::NOT_FOUND, "404, Not Found").into_response()
}

// TODO expand debug stuff?
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
