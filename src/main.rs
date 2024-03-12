// TODO: Add a proper license
// graceful shutdown? https://github.com/tokio-rs/axum/blob/main/examples/tls-graceful-shutdown/src/main.rs
// client tls? https://cloud.tencent.com/developer/article/1900692

// TODO: Consider feature flags?


// Standard library imports
use std::{
    net::SocketAddr,
    path::PathBuf,
    fs,
    io,
    sync::Arc,
};

// External crate imports
use axum::{
    extract::Host, handler::HandlerWithoutStateExt, http::{StatusCode, Uri},
    middleware, response::Redirect, routing::get, BoxError, Extension, Router
};
use axum_server::tls_rustls::RustlsConfig;

use clap::Parser;

use core::panic;

use serde::Deserialize;

use tracing;
use tracing_subscriber::{
    filter::LevelFilter,
    //fmt::{self, format},
    layer::SubscriberExt, 
    prelude::*,
};
use tracing_subscriber::fmt;


use redis::Client;

// Local imports

mod other;
mod api;
mod custom_middleware;
mod redis_client;

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
    redis_port: u16,

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

/* #[derive(Clone)]
struct RedisState{
   pub connection: Client,
} */
struct RedisState {
    pub connection: Arc<Client>, 
}

#[derive(Clone, Default)]
struct DatabaseState {
    //pub connection: T,
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

    // Load the configuration file
    if args.config.is_empty() {
        panic!("[!] No configuration file provided");
    }

    other::ascii_art(); // eh why not?

    tracing::info!("Using configuration file: {}", args.config);
    let config = load_config(&args.config).unwrap();
    //println!("Config loaded:\n{:?}", config);

    setup_logging(&config);

    // setup redis
    let connection_string = format!("{}:{}/0", config.redis_server, config.redis_port);
    tracing::debug!("Redis server: {}", connection_string);

    let shared_redis_client = redis_client::connect_to_redis(&connection_string).await;
    match shared_redis_client {
        Ok(client) => {
            tracing::debug!("Connected to Redis successfully!");
        }
        Err(e) => {
            //tracing::error!("Could not connect to Redis: {}", e);
            panic!("Exiting due to Redis error:\n{}", e);

        }
    }
    
    // TODO setup database

    // uuid <--> Secrets routes
    let client = redis_client::connect_to_redis(&config.redis_server).await;
    let shared_state = match client {
        Ok(client) => RedisState { connection: Arc::new(client) },
        Err(e) => panic!("Could not connect to Redis: {}", e),
    };
    #[derive(Clone)]
    struct RedisState {
        pub connection: Arc<Client>, 
    }

    let secrets_routes = Router::new()
        .route("/create_secret_url", get(|| async { /* Logic here */ }))
        //.route("/create_secret_url", get(api::create_secret_url))
        .route("/retrieve_secret_url", get(|| async { /* Logic here */ }))
        //.route("/retrieve_secret_url", get(api::retrieve_secret_url))
        //.layer(Extension(shared_redis_connection)) // create a layer for auth always flag
        .with_state(shared_state.clone());
        //.with_state(shared_redis_connection.clone());

    let webserver = Router::new()
        .route("/favicon.ico", get(api::favicon))
        .route("/", get(api::root_handler))

        // redis routes
        .nest("/secrets", secrets_routes)   

        .route("/signup", get(api::signup_get_handler).post(api::signup_post_handler))
        //.route("/login", post(api::login_handler))
        //.route("/logout", post(api::logout_handler))

        // Debug Routes
        // fault injection to test middleware later
        .route("/status", get(api::status_handler))
        .route("/headers", get(api::header_handler))
        .route("/connection", get(api::connection_handler))
        //.route("/trigger_error", get(trigger_error))
        
        // catch all handles and layers
        .route("/*any", get(api::not_found))//;
        .layer(middleware::from_fn(custom_middleware::print_request_response))//;
        .layer(Extension(config.debug_requests));
        //.layer(Extension(redis_connection));
        //.layer(HandleErrorLayer::new(custom_middleware::handle_error));


    let ports = Ports { //todo move to two args vs this setup?
        http: config.http_port,
        https: config.https_port,
    };
    
    // optional: spawn a second server to redirect http requests to this server
    if config.http_redirection {
        tracing::info!("Spawning HTTP redirection server on port {}", ports.http);
        tokio::spawn(redirect_http_to_https(ports));
    }

    let cert_path_buf = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(config.cert_path);
    let key_path_buf = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(config.key_path);
    tracing::info!("Using certificate at: {:?}", cert_path_buf);
    tracing::info!("Using private key at: {:?}", key_path_buf);

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