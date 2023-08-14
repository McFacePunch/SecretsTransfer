use axum::{Router, handler::{get, post}, http::Uri, response::Redirect, extract::Host};
use rustls::{NoClientAuth, ServerConfig};
use tokio_rustls::TlsAcceptor;

use std::fs::File;
use std::io::BufReader;
use std::fs;
use std::sync::Arc;
use std::net::SocketAddr;
use std::path::PathBuf;

use serde::Deserialize;

use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::fmt;
use tracing_subscriber::filter;
use tracing::{error, info};

//custom tracer
use tracer::init_tracing;

use crate::database::init_db;

mod tracer;
mod api;
mod database;

#[derive(Deserialize)]
struct Config {
    //webserver
    listen_address: String,
    http_port: u16,
    https_port: u16,

    //ssl
    cert_path: String,
    key_path: String,

    //redis
    redis_server: String,

    //database
    db_path: String,
    db_name: String,

    //debug
    debug_level: String,
    debug_log_path: String,
}

#[derive(Clone, Copy)]
struct Ports {
    http: u16,
    https: u16,
}

fn load_config() -> Config {
    let config_str = fs::read_to_string("config.json")
        .expect("Failed to read configuration file");
    serde_json::from_str(&config_str)
        .expect("Failed to deserialize configuration")
}

async fn hello_world() -> &'static str {
    "Hello, Axum!"
}

async fn configure_tls(cert_path: &str, key_path: &str) -> RustlsConfig {
    RustlsConfig::from_pem_file(
        PathBuf::from(cert_path),
        PathBuf::from(key_path),
    )
    .await
    .unwrap()
}

fn router_setup() -> Router<()> {
    Router::new()
        .route("/", get(hello_world)) 
        .route("/store_secret", post(api::store_secret))
        .route("/retrieve_secret", get(api::retrieve_secret))
        .route("/signup", post(api::signup))   // New route
        .route("/login", post(api::login))     // New route
}

async fn redirect_http_to_https(ports: Ports) {
    fn make_https(host: String, uri: Uri, ports: Ports) -> Result<Uri, BoxError> {
        let mut parts = uri.into_parts();

        parts.scheme = Some(axum::http::uri::Scheme::HTTPS);

        if parts.path_and_query.is_none() {
            parts.path_and_query = Some("/".parse().unwrap());
        }

        let https_host = host.replace(&ports.http.to_string(), &ports.https.to_string());
        parts.authority = Some(https_host.parse()?);

        Ok(Uri::from_parts(parts)?)
    }

    let redirect = move |Host(host): Host, uri: Uri| async move {
        match make_https(host, uri, ports) {
            Ok(uri) => Ok(Redirect::permanent(&uri.to_string())),
            Err(error) => {
                tracing::warn!(%error, "failed to convert URI to HTTPS");
                Err(StatusCode::BAD_REQUEST)
            }
        }
    };

    let addr = SocketAddr::from(([127, 0, 0, 1], ports.http));
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    tracing::debug!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, redirect.into_make_service())
        .await
        .unwrap();
}

#[tokio::main]
async fn main() {
    
    let config = load_config();

    init_tracing(&config.debug_log_path);

    let ports = Ports {
        http: config.http_port,
        https: config.https_port,
    };

    tracing::info!("Initializing Database...");
    if let Err(err) = api::init_db(config.db_path, config.db_name).await {
        error!("Failed to initialize the database: {}", err);
        return;
    }

    tokio::spawn(redirect_http_to_https(ports));

    let tls_config = configure_tls(&config.cert_path, &config.key_path).await;

    let router = router_setup();

    // expand the following lines to include an error for failing to attach to the port or bind to the address
    let listen_addr = format!("{}:{}", config.listen_address, config.https_port);
    let addr = match listen_addr.parse() {
        Ok(address) => address,
        Err(e) => {
            error!("Failed to parse listen address: {}. Error: {}", listen_addr, e);
            return;
        }
    };

    tracing::info!("listening on {}", addr);
    if let Err(e) = axum_server::bind_rustls(addr, tls_config)
        .serve(router.into_make_service())
        .await
    {
        error!("Failed to bind to address and start the server: {}", e);
    }

}
