// TODO: create a middleware to check if the user is AuthN/AuthZ'd
// TODO: implement dual token generation (lookup+crypto tokens) and secret storage in Redis 
// TODO: Encrypt secret at rest
use std::{
    convert::Infallible, path::PathBuf, 
    net::{IpAddr, SocketAddr}, fmt::Write,
    collections::HashMap,
    sync::{Arc, Mutex},
};

use axum::{
    body::Body, debug_handler, 
    extract::{ConnectInfo, Extension, Form, Path, Query, Json},
    http::{header::HeaderMap, status, StatusCode},
    response::{Html, IntoResponse, Redirect, Response}
};
use serde_json::map;
use tracing_subscriber::registry::Data;

use crate::{database, redis_client, redis_client::RedisOperation};
use redis::aio::MultiplexedConnection;

use serde::Deserialize;
use serde::Serialize;

use tokio::fs::read;
use uuid::Uuid;
use validator::Validate; 
use base64;

use crate::config;

#[derive(Deserialize, Validate)]
struct SecretData {
    #[validate(length(max = 10240, message = "Secret exceeds maximum size of 10KB"))]
    secret: String,
}

#[derive(Deserialize,Serialize,Validate)]
pub struct SignupForm {
    #[validate(email)]
    pub email: String,
    
    #[validate(length(min = 12))]
    pub password: String,
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

    let form = SignupForm { email: signup_data.email, password: signup_data.password};
    match form.validate() {
        Ok(_) => {
            tracing::info!("Received a valid signup request for user: {}", form.email);

            // if user exists bail with error
            // else create
            // return 201 or redirect to login?
            //redirect_to_login().await;
            return StatusCode::ACCEPTED;
            //return Redirect::temporary("login")
        }
        Err(e) => {
            tracing::debug!("Received a bad signup request for user: {}", form.email);
            return StatusCode::BAD_REQUEST;
        }
    }
} 






pub async fn store_secret(Extension(db): Extension<Arc<Mutex<database::DBStates>>>) -> impl IntoResponse {
    let secret_uuid = database::get_uuid();

    let base_url = "https://localhost:8443/secrets/retrieve_secret/";
    let secret_url = format!("{}{}", base_url, secret_uuid);

    {
        let db = &mut db.lock().unwrap();

        match &mut db.value_store {
            database::StorageEnum::InMemory(map) => {
                // Insert directly into the map without cloning.
                map.insert(secret_uuid.clone(), secret_url.clone());
                tracing::info!("Stored secret with UUID: {}", secret_uuid);
                for (key, value) in map.iter() {
                    tracing::info!("KEY VALUE !!!!!!!!!!!!!!! {}: {}", key, value);
                }
            },

            database::StorageEnum::ExternalDB(ref mut redis_connection) => {
                redis_client::get_or_set_value_with_retries(
                    RedisOperation::Set, 
                    redis_connection, 
                    &secret_uuid, 
                    Some(&secret_url)
                ).await.unwrap();
                tracing::debug!("NO OP ExternalDB")
            },

            database::StorageEnum::None => {
                tracing::debug!("No storage defined.");
            }
        }
    } // Lock is automatically released here

    (StatusCode::OK, secret_url).into_response()
}


/* 
pub async fn store_secret(
    Extension(db): Extension<Arc<Mutex<database::DBStates>>>,
) -> impl IntoResponse {
    let secret_uuid = database::get_uuid();

    let base_url = "https://localhost:8443/secrets/retrieve_secret/";
    let secret_url = format!("{}{}", base_url, secret_uuid);

    {
        let mut db = db.lock().unwrap(); // Acquire the lock on the database state.
        let mut hashtable = hashtable.lock().unwrap().value_store;

        match &mut db.value_store {
            database::StorageEnum::InMemory(map) => {
                // Insert directly into the map without cloning.
                map.insert(secret_uuid.clone(), secret_url.clone());
                tracing::info!("Stored secret with UUID: {}", secret_uuid);
                for (key, value) in map.iter() {
                    tracing::info!("KEY VALUE !!!!!!!!!!!!!!! {}: {}", key, value);
                }
            },
            database::StorageEnum::ExternalDB(redis_connection) => {
                // Assuming `redis_client::get_or_set_value_with_retries` is an async function you have defined.
                redis_client::get_or_set_value_with_retries(
                    RedisOperation::Set, 
                    redis_connection, 
                    &secret_uuid, 
                    Some(&secret_url)
                ).await.unwrap();
            },
            database::StorageEnum::None => {
                tracing::debug!("No storage defined.");
            }
        }
    } // Lock is automatically released here

    (StatusCode::OK, secret_url).into_response()
}
 */


pub async fn retrieve_secret(
    Path(uuid): Path<String>, 
    Extension(db): Extension<Arc<Mutex<database::DBStates>>>,
) -> impl IntoResponse {
    {
        let db = &mut db.lock().unwrap();

        match &mut db.value_store {
            database::StorageEnum::InMemory(mapp) => {
                let map = mapp.clone();
                let output = map.get(&uuid).unwrap_or(&"HT Secret not found".to_string()).to_string(); 
                
                (StatusCode::OK, output.to_string()).into_response()
            }
            database::StorageEnum::ExternalDB(ref mut redis_connection) => {
                redis_client::get_value_with_retries(
                    &mut redis_connection.clone(), 
                    &uuid).await.map_or_else(
                    |status| (StatusCode::NOT_FOUND, "EDB Secret not found").into_response(),
                    |secret_url| (StatusCode::OK, secret_url).into_response(),
                )
            }
            database::StorageEnum::None => (StatusCode::NOT_FOUND, "NO OP").into_response(),
        }
    }
}


















pub async fn store_secret2(Extension(hashtable): Extension<Arc<Mutex<database::DBStates>>>) -> impl IntoResponse {
    let secret_uuid = database::get_uuid();

    let base_url = "https://localhost:8443/secrets/retrieve_secret/";
    let secret_url = format!("{}{}", base_url, secret_uuid);

    {
        let hashtable = &mut hashtable.lock().unwrap();

        match &mut hashtable.value_store {
            database::StorageEnum::InMemory(map) => {
                // Insert directly into the map without cloning.
                map.insert(secret_uuid.clone(), secret_uuid.clone());
                
                tracing::info!("Stored secret with UUID: {}", secret_uuid);
                for (key, value) in map.iter() {
                    tracing::info!("KEY VALUE !!!!!!!!!!!!!!! {}: {}", key, value);
                }
            },
            // TODO fix this and the weird redis_connection issue
            database::StorageEnum::ExternalDB(ref mut redis_connection) => {
                // Assuming `redis_client::get_or_set_value_with_retries` is an async function you have defined.
                // redis_client::get_or_set_value_with_retries(
                //     RedisOperation::Set, 
                //     &mut redis_connection, 
                //     &secret_uuid, 
                //     Some(&secret_url)
                // ).await.unwrap();
                tracing::debug!("NO OP ExternalDB")
            },
            database::StorageEnum::None => {
                tracing::debug!("No storage defined.");
            }
        }
    } // Lock is automatically released here

    (StatusCode::OK, secret_url).into_response()
}

pub async fn retrieve_secret2(
    Path(uuid): Path<String>, 
    Extension(db): Extension<Arc<Mutex<database::DBStates>>>,
) -> impl IntoResponse {
    let db = &mut db.lock().unwrap().value_store;

    //let output = d
    match db {
        database::StorageEnum::InMemory(map) => {
            let output = &mut map.get(&uuid).unwrap();

            //db.get(&uuid).unwrap_or(&"HT Secret not found".to_string()).to_string();
                    
            (StatusCode::OK, output.to_string()).into_response()
            }

        database::StorageEnum::ExternalDB(ref mut redis_connection) => {
            (StatusCode::NOT_FOUND).into_response()
        }
        database::StorageEnum::None => (StatusCode::NOT_FOUND).into_response(),

        // match&mut db.value_store {
        //     database::StorageEnum::InMemory(map) => {
        //         let output = map.get(&uuid).unwrap_or(&"HT Secret not found".to_string()).to_string();
                
        //         (StatusCode::OK, output.to_string()).into_response()
        //     }
        //     database::StorageEnum::ExternalDB(redis_connection) => {
        //         redis_client::get_value_with_retries(&mut redis_connection.clone(), &uuid).await.map_or_else(
        //             |status| (StatusCode::NOT_FOUND, "EDB Secret not found").into_response(),
        //             |secret_url| (StatusCode::OK, secret_url).into_response(),
        //         )
        //     }
        //     database::StorageEnum::None => (StatusCode::NOT_FOUND, "NO OP").into_response(),
        // }
    }
}






















//////////////////
// HashTable GET/SET
//////////////////
pub async fn hashtable_store_secret(
    Extension(db): Extension<Arc<Mutex<HashMap<String, String>>>>
) -> impl IntoResponse {
    let secret_uuid = database::get_uuid();
    let base_url = "https://localhost:8443/secrets/retrieve_secret/";
    let secret_url = format!("Secret-test:\n{}{}", base_url, secret_uuid);

    //lock, write and unlock
    let mut db = db.lock().unwrap();
    db.insert("key".to_string(), "oogy boogy".to_string());
    drop(db);
    //

    (StatusCode::OK, secret_url).into_response()
}

async fn hashtable_retrieve_secret(
    Extension(db): Extension<Arc<Mutex<HashMap<String, i32>>>>
) -> impl IntoResponse {
    // Acquire the lock
    let mut db = db.lock().unwrap();

    // Modify your hashmap
    db.insert("test-key".to_string(), 42); 

    drop(db); // Release the lock

    StatusCode::OK // Change the status code if needed
}


//////////////////
// Redis/Valkey GET/SET
//////////////////
#[debug_handler]
async fn redis_store_secret(Extension(redis_connection): Extension<MultiplexedConnection>) -> impl IntoResponse {
    // send uuid as key and secret as value

    let secret_uuid = uuid::Uuid::new_v4().to_string();

    //generate URL with UUID
    let base_url = "https://localhost:8443/secrets/retrieve_secret/"; // TODO get HOST value dymanically
    let secret_url = format!("Secret-test:\n{}{}", base_url, secret_uuid);

    redis_client::get_or_set_value_with_retries(RedisOperation::Set, &mut redis_connection.clone(), &secret_uuid, Some(&secret_url)).await.unwrap();

    (StatusCode::OK, secret_url).into_response()
}

#[debug_handler]
async fn redis_store_secret_post(
    Extension(redis_connection): Extension<MultiplexedConnection>,
    Json(payload): Json<SecretData>,
) -> impl IntoResponse {
    // validate constraints of secret, <=10k, etc 
    // Get the secret from the JSON payload and encode it
    let secret_string = base64::encode(&payload.secret);

    // Generate UUID
    let secret_uuid = uuid::Uuid::new_v4().to_string();

    // Store in Redis
    redis_client::get_or_set_value_with_retries(RedisOperation::Set, &mut redis_connection.clone(), &secret_uuid, Some(&secret_string))
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
async fn redis_retrieve_secret(
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
