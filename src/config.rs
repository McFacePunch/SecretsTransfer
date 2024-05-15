use serde::Deserialize;
use std::fs;

macro_rules! pub_struct {
    ($name:ident {$($field:ident: $t:ty,)*}) => {
        #[derive(Deserialize,Debug,Clone)]
        pub struct $name {
            $(pub $field: $t),*
        }
    }
} // pub_struct!(DBConfig { value: bool });

pub_struct!( Config {
    //webserver
    listen_address: String,
    http_port: u16,
    https_port: u16,

    http_redirection: bool,

    //ssl
    cert_path: String,
    key_path: String,
    
    //redis
    redis_enabled: bool,
    redis_server: String,
    redis_port: u16,

    //user database
    users_enabled: bool,
    db_persist: bool,
    db_remote: bool,
    db_host: String,
    db_port: u16,
    db_path: String,
    db_name: String,

    //debug
    debug_level: String,
    debug_requests: bool,
    debug_log_path: String,
    }
);

pub_struct!( DBConfig {
    db_persist: bool,
    db_remote: bool,
    db_host: String,
    db_port: u16,
    db_path: String,
    db_name: String,
    }
);

pub fn load_config(config_file_path: &str) -> Result<Config, Box<dyn std::error::Error>> {
    let file_content = fs::read_to_string(config_file_path)?;
    let config: Config = serde_json::from_str(&file_content)?;
    Ok(config)
}
