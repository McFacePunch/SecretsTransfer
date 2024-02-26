// TODO: Add a proper license
// graceful shutdown? https://github.com/tokio-rs/axum/blob/main/examples/tls-graceful-shutdown/src/main.rs
// client tls? https://cloud.tencent.com/developer/article/1900692


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

use axum_server::tls_rustls::RustlsConfig;

use rustls::{
    server::{NoClientAuth},
    sign::CertifiedKey,
    CipherSuite, RootCertStore, SupportedCipherSuite,
};

use serde::{Deserialize, Deserializer};

use clap::Parser;

use std::{net::SocketAddr, path::PathBuf, str::FromStr, sync::Arc};
use std::fs;
use std::io;

use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use tracing_subscriber::fmt;
use tracing_subscriber::prelude::*;
use tracing_subscriber::filter::LevelFilter;

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

mod other;
use other::ascii_art;

mod api;
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

    tracing::info!("Using certificate: {:?}", cert_path_buf);
    tracing::info!("Using private key: {:?}", key_path_buf);

    let webserver = Router::new()
        .route("/", get(api::root_handler))
        .route("/status", get(api::status_handler))
        .route("/login", post(api::login_handler))
        .route("/logout", post(api::logout_handler))
        .route("/:any", get(api::not_found));
        //.route("/signup", post(api::signup))
        //.route("/retrieve_secret", get(api::retrieve_secret))
        //.route("/store_secret", post(api::store_secret));
        //.nest("/", Router::new()
        //    .route("/status", get(api::status_handler))

    let tls_config = RustlsConfig::from_pem_file(cert_path_buf, key_path_buf).await.unwrap();
        // Specify accepted ciphers
    
/*     tls_config.ciphersuites = vec![
        &rustls::ciphersuite::TLS13_AES_256_GCM_SHA384,
        &rustls::ciphersuite::TLS13_CHACHA20_POLY1305_SHA256,
        // Add more ciphers here
    ]; */

    // run https server
    let addr = SocketAddr::from(([127, 0, 0, 1], ports.https));
    tracing::info!("listening on {}", addr);
    axum_server::bind_rustls(addr, tls_config)
        .serve(webserver.into_make_service())
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



/* 


pub fn load_certs(filename: &str) -> Vec<rustls::Certificate> {
    let certfile = File::open(filename).expect("cannot open certificate file");
    let mut reader = BufReader::new(certfile);
    rustls_pemfile::certs(&mut reader)
        .unwrap()
        .iter()
        .map(|v| rustls::Certificate(v.clone()))
        .collect()
}

pub fn load_private_key(filename: &str) -> rustls::PrivateKey {
    let keyfile = File::open(filename).expect("cannot open private key file");
    let mut reader = BufReader::new(keyfile);

    loop {
        match rustls_pemfile::read_one(&mut reader).expect("cannot parse private key .pem file") {
            Some(Item::RSAKey(key)) => return rustls::PrivateKey(key),
            Some(Item::PKCS8Key(key)) => return rustls::PrivateKey(key),
            None => break,
            _ => {}
        }
    }

    panic!(
        "no keys found in {:?} (encrypted keys not supported)",
        filename
    );
}



fn make_server_config(certs: &str, key_file: &str) -> Arc<rustls::ServerConfig> {
    let roots = load_certs(certs);
    let certs = roots.clone();
    let mut client_auth_roots = RootCertStore::empty();
    for root in roots {
        client_auth_roots.add(&root).unwrap();
    }
    let client_auth = AllowAnyAuthenticatedClient::new(client_auth_roots);

    let privkey = load_private_key(key_file);
    let suites = rustls::ALL_CIPHER_SUITES.to_vec();
    let versions = rustls::ALL_VERSIONS.to_vec();

    let mut config = rustls::ServerConfig::builder()
        .with_cipher_suites(&suites)
        .with_safe_default_kx_groups()
        .with_protocol_versions(&versions)
        .expect("inconsistent cipher-suites/versions specified")
        .with_client_cert_verifier(client_auth)
        .with_single_cert_with_ocsp_and_sct(certs, privkey, vec![], vec![])
        .expect("bad certificates/private key");

    config.key_log = Arc::new(rustls::KeyLogFile::new());
    config.session_storage = rustls::server::ServerSessionMemoryCache::new(256);
    Arc::new(config)
}
 */
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