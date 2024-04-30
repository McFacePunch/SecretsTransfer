#![forbid(unsafe_code)]
// TODO: Add a proper license
// graceful shutdown? https://github.com/tokio-rs/axum/blob/main/examples/tls-graceful-shutdown/src/main.rs
// client tls? https://cloud.tencent.com/developer/article/1900692

// TODO: Consider feature flags?


// Standard library imports
use std::{
    net::SocketAddr,
    path::PathBuf,
    io,
    sync::Arc,
    sync::Mutex,
    collections::HashMap,
};

// External crate imports
//{FromRequest, Host}
use axum::{
    extract::Host, handler::HandlerWithoutStateExt, http::{StatusCode, Uri},
    middleware, response::Redirect, routing::{get, post}, BoxError, Extension, Router,
    extract::State
};
use axum_server::tls_rustls::RustlsConfig;

use clap::Parser;
use tower::layer;

use core::panic;

use tracing;
use tracing_subscriber::{
    filter::LevelFilter,
    layer::SubscriberExt, 
    prelude::*,
};
use tracing_subscriber::fmt;

use redis::aio::MultiplexedConnection;

// Local imports

mod other;
mod api;
mod custom_middleware;
mod redis_client;
mod database;

mod config;

mod tests;

#[derive(Clone)]
struct RedisState {
    pub _conn: Arc<MultiplexedConnection>, 
}

struct DatabaseLayer {
    db: Arc<Mutex<HashMap<String, i32>>>,
}


#[allow(dead_code)]
#[derive(Clone, Copy)]
struct Ports {
    http: u16,
    https: u16,
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
    let args = Args::parse();

    // Load the configuration file
    if args.config.is_empty() {
        panic!("[!] No configuration file provided");
    }

    other::ascii_art(); // eh why not?

    tracing::info!("Using configuration file: {}", args.config);
    let config: config::Config = config::load_config(&args.config).unwrap();

    setup_logging(&config);

    /* Reference Implemnation Delete Later
    // Initialize the KV database
    let kv_db = init_kv_db(&config::Config::default()).await.unwrap();

    // Create a DBStates struct
    let db_states = DBStates {
        value_store: Some(Arc::new(Mutex::new(ValueStore::InMemory(kv_db)))),
        user_db: None,
    };

    // Create a Router
    let router = Router::new().route("/", get(handler));

    // Add the DBStates to the router's extensions
    let app = router.layer(Extension(db_states));
    */

    //let mut kv_db = database::init_kv_db(&config).await.unwrap();

    tracing::debug!("Creating db_sate");
    let db_state = database::DBStates {
        //value_store: Some(Arc::new(Mutex::new(database::StorageEnum::InMemory(kv_db)))),
        value_store: database::init_kv_db(&config).await.unwrap(),
        user_db: database::init_user_db(&config).await.unwrap(),
    };

    let arc_db_state: Arc<Mutex<database::DBStates>> = Arc::new(Mutex::new(db_state));
    //let arc_db_state = Extension(Arc::new(Mutex::new(db_state)));
    //tracing::debug!("


    // uuid <--> Secrets routes
    let secrets_routes = Router::new()
        //.route("/store_secret", get(|State(state): State<MultiplexedConnection>|top_level_handler_fn!(get, api::store_secret)))
        .route("/store_secret", get(api::store_secret2))
        //.route("/retrieve_secret", get(api::retrieve_secret))
        .route("/retrieve_secret/:uuid", get(api::retrieve_secret2))
        .route("/*any", get(api::not_found))
        //.layer(Extension(db_state));// create a layer for enforced auth
        //.layer(Extension(db_state))
        .layer(Extension(arc_db_state));
        
    let webserver = Router::new()
        .route("/favicon.ico", get(api::favicon))
        .route("/", get(api::root_handler))

        // redis route nesting
        .nest("/secrets", secrets_routes)   

        // User routes
        .route("/signup", get(api::signup_get_handler).post(api::signup_post_handler))
        .route("/login", post(api::login_handler))
        .route("/logout", post(api::logout_handler))

        // Debug/Info Routes
        .route("/status", get(api::status_handler))
        .route("/headers", get(api::header_handler))
        .route("/connection", get(api::connection_handler))
        //.route("/trigger_error", get(trigger_error)) // TODO: fault injection to test middleware
        
        // catch all handles and layers
        .route("/*any", get(api::not_found))
        .layer(middleware::from_fn(custom_middleware::print_request_response))
        .layer(Extension(config.debug_requests));
        //.layer(HandleErrorLayer::new(custom_middleware::handle_error)); // TODO: wrap all error via this handler middleware


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

fn setup_logging(config: &config::Config) {
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