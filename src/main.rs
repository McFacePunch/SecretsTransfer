// TODO: Add a proper license
// graceful shutdown? https://github.com/tokio-rs/axum/blob/main/examples/tls-graceful-shutdown/src/main.rs
// client tls? https://cloud.tencent.com/developer/article/1900692

// TODO: Consider feature flags?


#![allow(unused_imports)]

use axum::{
    extract::Host,
    handler::HandlerWithoutStateExt,
    http::{StatusCode, Uri},
    response::Redirect,
    routing::get,
    routing::post,
    BoxError, Router,
};
use axum::{ response::IntoResponse,};
use axum::middleware::from_fn;
use axum::middleware;
use axum::error_handling::HandleErrorLayer;
use axum::Extension;

use axum_server::tls_rustls::RustlsConfig;

use rustls::{
    server::{NoClientAuth},
    sign::CertifiedKey,
    CipherSuite, RootCertStore, SupportedCipherSuite,
};

use tower::ServiceBuilder;
use tokio::time::error;
//use tower_http::{add_extension::AddExtensionLayer, trace::TraceLayer};

use serde::{Deserialize, Deserializer};

use clap::Parser;

use std::{net::SocketAddr, net::IpAddr, path::PathBuf, str::FromStr, sync::Arc};
use std::fs;
use std::io;
use std::time::Duration;

use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use tracing_subscriber::fmt;
use tracing_subscriber::prelude::*;
use tracing_subscriber::filter::LevelFilter;
use tracing::{debug, error, info};

use std::convert::TryFrom;
use std::fs::File;
use std::io::BufReader;

use tokio::net::{TcpStream};
use tokio_rustls::{
    TlsAcceptor,
    TlsConnector,
    rustls::{self},
    client::TlsStream as ClientTlsStream,
};
use tokio::net::TcpListener; 

mod other;
use other::ascii_art;

mod api;
mod custom_middleware;
//use api; //::{login_handler, logout_handler, status_handler, not_found};

//use axum::prelude::*;
//use axum::middleware::Logger;

#[allow(dead_code)]
#[derive(Deserialize,Debug)]
struct Config {
    //webserver
    listen_address: String,
    http_port: u16,
    https_port: u16,

    http_redirection: bool,

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
    debug_requests: bool,
    debug_log_path: String,
}

#[allow(dead_code)]
#[derive(Clone, Copy)]
struct Ports {
    http: u16,
    https: u16,
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path to the configuration file
    #[arg(short, long)]
    config: String
}

#[tokio::main]
async fn main() {
    let args = Args::parse(); // Parse command line arguments

    ascii_art(); // eh why not?

    // Load the configuration file
    if args.config.is_empty() {
        panic!("[!] No configuration file provided");
    }

    tracing::info!("Using configuration file: {}", args.config);
    let config = load_config(&args.config).unwrap();
    //println!("Config loaded:\n{:?}", config);

    setup_logging(&config);

    //remove this
    tracing::debug!("Redis server: {}", config.redis_server);
    tracing::debug!("Database path: {}", config.db_path);
    

    // configure the ports used by the server, passed to http handler
    let ports = Ports { //todo move to two args vs this setup
        http: config.http_port,
        https: config.https_port,
    };
    
    // optional: spawn a second server to redirect http requests to this server
    if config.http_redirection {
        tracing::info!("Spawning HTTP redirection server on port {}", ports.http);
        tokio::spawn(redirect_http_to_https(ports));
    }

    // configure certificate and private key used by http(s) server
    let cert_path_buf = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(config.cert_path);
    let key_path_buf = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(config.key_path);

    tracing::info!("Using certificate at: {:?}", cert_path_buf);
    tracing::info!("Using private key at: {:?}", key_path_buf);

    let webserver = Router::new()
        //.layer(Extension(config.debug_requests))
        //.layer(middleware::from_fn(custom_middleware::print_request_response))
        .route("/", get(api::root_handler))
        .route("/status", get(api::status_handler))
        .route("/create_secret_url", get(api::create_secret_url) )
        //.route("/retrieve_secret_url", get(api::retrieve_secret_url) )
        //.route("/secret/:id", get(api::retrieve_secret) ) // todo use more CRUD
        .route("/headers", get(api::header_handler))
        .route("/connection", get(api::connection_handler))

        //.route("/login", post(api::login_handler))
        //.route("/logout", post(api::logout_handler))
        .route("/:any", get(api::not_found))//;
        //
        //.layer(Extension(config.debug_requests))
        .layer(middleware::from_fn(custom_middleware::print_request_response))//;
        .layer(Extension(config.debug_requests));

        //.route("/signup", post(api::signup))
        //.route("/retrieve_secret", get(api::retrieve_secret))
        //.route("/store_secret", post(api::store_secret));
        //.nest("/", Router::new()
        //    .route("/status", get(api::status_handler))

    let tls_config = RustlsConfig::from_pem_file(cert_path_buf, key_path_buf).await.unwrap();

    // run https server
    let addr = SocketAddr::from(([127, 0, 0, 1], ports.https));
    tracing::info!("listening on {}", addr);
    axum_server::bind_rustls(addr, tls_config)
        .serve(webserver.into_make_service_with_connect_info::<SocketAddr>())//.into_make_service())
        .await
        .unwrap();
}

fn load_config(config_file_path: &str) -> Result<Config, Box<dyn std::error::Error>> {
    let file_content = fs::read_to_string(config_file_path)?;
    let config: Config = serde_json::from_str(&file_content)?;
    Ok(config)
}

fn setup_logging(config: &Config) {
    // Parse the log level from the config
    let log_level = match config.debug_level.to_lowercase().as_str() {
        "error" => LevelFilter::ERROR,
        "warn" => LevelFilter::WARN,
        "info" => LevelFilter::INFO,
        "debug" => LevelFilter::DEBUG,
        "trace" => LevelFilter::TRACE,
        _ => panic!("Invalid log level"),
    };

    // TODO: support filtering by module
    // exmaple: let filter_layer = tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "SecretsTransfer=debug".into());

    // Initialize the logger
    let file_layer = fmt::layer()
        .with_writer(std::fs::OpenOptions::new()
            .append(true)
            .create(true)
            .open(config.debug_log_path.as_str())
            .unwrap())
        .with_filter(log_level);

    let stdout_layer = fmt::Layer::new()
        .with_writer(io::stdout)
        .with_filter(log_level);

    tracing_subscriber::registry()
        .with(file_layer)
        .with(stdout_layer)
        .init();

    tracing::info!("Logging initialized");
}

// Redirect HTTP to HTTPS
#[allow(dead_code)]
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