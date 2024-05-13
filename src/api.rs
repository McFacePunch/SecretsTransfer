// TODO: create a middleware to check if the user is AuthN/AuthZ'd
// TODO: implement dual token generation (lookup+crypto tokens) and secret storage in Redis 
// TODO: Encrypt secret at rest
use std::{
    convert::Infallible, path::PathBuf, 
    net::{IpAddr, SocketAddr}, fmt::Write,
    sync::{Arc, Mutex},
    error::Error,
};

use axum::{
    body::Body, //debug_handler, //todo use handler
    extract::{ConnectInfo, Extension, Form, Path},
    http::{header::HeaderMap, StatusCode},
    response::{Html, IntoResponse, Redirect, Response}
};

use crate::database;

use serde::Deserialize;
use serde::Serialize;

use tokio::fs::read;
use validator::Validate; 

use crate::database::{Storage, StorageEnum};

#[derive(Deserialize, Validate)]
pub struct SecretData {
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
    // TODO: add this to the config?
    // TODO: check for file existence and error?
    // TODO:cache the file to prevent disk reads
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

// TODO implement the login page
pub async fn login_get_handler() -> impl IntoResponse {
    (StatusCode::OK, "Login Page").into_response()
}

// TODO implement this
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

async fn get_value(storage: &StorageEnum, key: &str) -> Result<Option<String>, Box<dyn Error>> {
    match storage {
        StorageEnum::InMemory(map) => map.get(key).await,
        StorageEnum::Redis(pool) => pool.get(key).await,
        //StorageEnum::NoSQLDB(conn) => conn.execute("SELECT value FROM table WHERE key = ?", [key]),
        StorageEnum::None => Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, "No database available"))),
    }
}

async fn set_value(storage: &StorageEnum, key: &str, value: &str) -> Result<(), Box<dyn Error>> {
    match storage {
        StorageEnum::InMemory(map) => map.set(key, value).await,
        StorageEnum::Redis(pool) => pool.set(key, value).await,
        //StorageEnum::NoSQLDB(conn) => conn.execute("INSERT INTO table (key, value) VALUES (?, ?)", [key, value]),
        StorageEnum::None => Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, "No database available"))),
    }
}

use crate::frontend;

use askama::Template;

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
            tracing::info!("Stored secret {}", secret_data.secret);
            let url = Some(secret_url); // Success message
            (StatusCode::OK, Html(frontend::SecretFormTemplate { result: url }.render().unwrap())).into_response()
            //(StatusCode::OK, Html(frontend::OldSecretFormTemplate { }.render().unwrap())).into_response()
        }
        Err(e) => {
            tracing::error!("Error storing secret: {}", e);
            //let result = Some(format!("Error storing secret: {}", e)); // Error message
            (StatusCode::INTERNAL_SERVER_ERROR, Html(frontend::SecretFormTemplate { result: None }.render().unwrap())).into_response()
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

/* 
//pub async fn store_secret2(Extension(hashtable): Extension<Arc<Mutex<database::DBStates>>>) -> impl IntoResponse {
pub async fn store_secret2(Extension(mut value_store): Extension<Arc<database::DB_Object<database::StorageEnum>>>) -> impl IntoResponse {
    let secret_uuid = database::get_uuid();

    let base_url = "https://localhost:8443/secrets/retrieve_secret/";
    let secret_url = format!("{}{}", base_url, secret_uuid);


    match value_store.as_ref() {
         database::StorageEnum::InMemory(map) => {
             let mut map_write_guard = map.write().await; // Acquire write lock
             map_write_guard.insert(secret_uuid.clone(), secret_url.clone());
         }
         database::StorageEnum::ExternalDB(redis_pool) => {
             let mut redis_conn = redis_pool.get().await.unwrap(); // Get connection from pool
             redis_client::get_or_set_value_with_retries(
                 RedisOperation::Set,
                 &mut redis_conn,
                 &secret_uuid,
                 Some(&secret_url),
             )
             .await
             .unwrap();
         }
         database::StorageEnum::None => {
             tracing::debug!("No storage defined.");
         }
     }


    match value_store.as_ref() {//.as_ref() {
        database::StorageEnum::InMemory(ref map) => {
            // Insert directly into the map without cloning.
            &map.insert(secret_uuid.clone(), secret_uuid.clone());
        },
        // TODO fix this and the weird redis_connection issue
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

    (StatusCode::OK, secret_url).into_response()
} */


/* 
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
    }
} */























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
