use axum::{extract, response::Json};
use serde::{Deserialize, Serialize};
use rusqlite::{params, Connection, Error as SqliteError};
use bcrypt::{hash, verify};
use jsonwebtoken::{encode, Header, EncodingKey};  // Assuming you are using the 'jsonwebtoken' crate for JWT handling

use crate::database::{init_db, DbError, DB_PATH};

// Dummy structs to represent request and response. Modify as needed.
#[derive(Deserialize)]
pub struct StoreSecretRequest {
    secret: String,
}

#[derive(Serialize)]
pub struct StoreSecretResponse {
    message: String,
    secret_id: String,
}

#[derive(Deserialize)]
pub struct RetrieveSecretRequest {
    secret_id: String,
}

#[derive(Serialize)]
pub struct RetrieveSecretResponse {
    secret: String,
}

#[derive(Deserialize)]
pub struct SignupRequest {
    username: String,
    password: String,
}

#[derive(Serialize)]
pub struct SignupResponse {
    message: String,
}

#[derive(Deserialize)]
pub struct LoginRequest {
    username: String,
    password: String,
}

#[derive(Serialize)]
pub struct LoginResponse {
    token: String,
}

// Handler to store secret
pub async fn store_secret(extract::Json(body): extract::Json<StoreSecretRequest>) -> Json<StoreSecretResponse> {
    // Logic to store the secret goes here

    let response = StoreSecretResponse {
        message: "Secret stored successfully".into(),
        secret_id: "random_id".into(),
    };

    Json(response)
}

// Handler to retrieve secret
pub async fn retrieve_secret(extract::Json(body): extract::Json<RetrieveSecretRequest>) -> Json<RetrieveSecretResponse> {
    // Logic to retrieve the secret goes here

    let response = RetrieveSecretResponse {
        secret: "sample_secret".into(),
    };

    Json(response)
}

// Handler to signup users
pub async fn signup(extract::Json(body): extract::Json<SignupRequest>) -> Json<SignupResponse> {
    let hashed_password = hash(&body.password, 4).unwrap();

    let conn = Connection::open(DB_PATH).unwrap();
    match conn.execute(
        "INSERT INTO users (username, password) VALUES (?1, ?2)",
        params![&body.username, &hashed_password],
    ) {
        Ok(_) => Json(SignupResponse {
            message: "User signed up successfully".into(),
        }),
        Err(_) => Json(SignupResponse {
            message: "Signup failed".into(),
        }),
    }
}

// Handler to login users and return JWT
pub async fn login(extract::Json(body): extract::Json<LoginRequest>) -> Json<LoginResponse> {
    let conn = Connection::open(DB_PATH).unwrap();
    let mut stmt = conn.prepare("SELECT password FROM users WHERE username = ?1").unwrap();
    let stored_password: Result<String> = stmt.query_row(params![&body.username], |row| row.get(0));

    match stored_password {
        Ok(hashed_password) if verify(&body.password, &hashed_password).unwrap() => {
            let claims = Claims {
                sub: body.username.clone(),
                exp: (chrono::Utc::now() + chrono::Duration::hours(1)).timestamp() as usize,
            };

            let token = encode(&Header::default(), &claims, &EncodingKey::from_secret("secret_key".as_ref())).unwrap();

            Json(LoginResponse {
                token,
            })
        }
        _ => Json(LoginResponse {
            token: "Invalid credentials".into(),
        }),
    }
}

// Handler to verify JWT

