//use axum::{handler::get, handler::post, Router, response::IntoResponse, http::StatusCode, routing::fallthrough};
use axum::{
    extract::Host,
    routing::{get, post},
    http::StatusCode,
    response::{IntoResponse, Redirect},
    Router,
    BoxError,
};

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

pub async fn not_found() -> impl IntoResponse {
    (StatusCode::NOT_FOUND, "404, Not Found").into_response()
}

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



/* use axum::{extract, Json, Router, routing::post, response::IntoResponse};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;
use std::collections::HashMap;
use jsonwebtoken::{encode, EncodingKey, Header, decode, DecodingKey, Validation};
use axum::http::StatusCode;
use axum::response::Json as AxumJson;

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: String,
    exp: usize,
}

#[derive(Debug, Deserialize)]
struct LoginRequest {
    username: String,
    password: String,
}

#[derive(Debug, Serialize)]
struct LoginResponse {
    token: String,
}

type Db = Arc<Mutex<HashMap<String, String>>>;

async fn login(
    Json(payload): Json<LoginRequest>,
    db: extract::Extension<Db>,
) -> impl IntoResponse {
    let message = if let Some(password) = db.lock().await.get(&payload.username) {
        if password == &payload.password {
            let claims = Claims {
                sub: payload.username.clone(),
                exp: (chrono::Utc::now() + chrono::Duration::hours(2)).timestamp() as usize, //make duration configurable
            };
            let token = encode(&Header::default(), &claims, &EncodingKey::from_secret("secret".as_ref())).unwrap();
            return AxumJson(LoginResponse { token }).into_response();
        } else {
            "Invalid password".into()
        }
    } else {
        "Invalid username".into()
    };
    (StatusCode::UNAUTHORIZED, message).into_response()
}

async fn protected_route(
    extract::Extension(db): extract::Extension<Db>,
    extract::Header(header): extract::Header<Option<String>>,
) -> impl IntoResponse {
    let token = match header {
        Some(token) => token,
        None => return (StatusCode::UNAUTHORIZED, "No token provided").into_response(),
    };

    let token_data = decode::<Claims>(&token, &DecodingKey::from_secret("secret".as_ref()), &Validation::default());
    match token_data {
        Ok(data) => {
            let username = data.claims.sub;
            if db.lock().await.contains_key(&username) {
                (StatusCode::OK, "You are authenticated").into_response()
            } else {
                (StatusCode::UNAUTHORIZED, "Invalid token").into_response()
            }
        }
        Err(_) => (StatusCode::UNAUTHORIZED, "Invalid token").into_response(),
    }
}

pub fn router() -> Router {
    let db = Db::default();
    Router::new()
        .route("/login", post(login))
        .route("/protected", post(protected_route))
        .layer(axum::AddExtensionLayer::new(db))
} */