// database interrface for handling both local in-memeory and remote nosql databases cleanly.
use std::{fmt, result};
use std::sync::Arc;
use std::{collections::HashMap, error::Error};
use clap::error;
use uuid::Uuid;
use rusqlite::{Connection, Result};//, Error};
use std::path::Path;

use base64::prelude::*;

use tokio::sync::RwLock;

use crate::config;
use crate::redis_client;


type Key = String;
type Value = String;

#[derive(Debug)]
enum DatabaseError {
    Generic(String),
}

impl std::error::Error for DatabaseError {}


impl fmt::Display for DatabaseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            //DBError::DatabaseUnavailable => write!(f, "Database unavailable"),
            //åDBError::DatabaseError(err) => write!(f, "Database operation failed: {}", err),
            DatabaseError::Generic(message) => write!(f, "Database error: {}", message),
        }
    }
}

// #[derive(Clone)]
// pub enum StorageEnum {
//     InMemory(HashMap<String, String>),
//     ExternalDB(redis::aio::MultiplexedConnection),
//     //NoSQLDB(rusqlite::Connection),
//     None,
// }

#[derive(Clone)]
pub enum StorageEnum {
    InMemory(InMemoryStorage),
    Redis(RedisStorage),
    //NoSQLDB(rusqlite::Connection),
    //InMemory(Arc<RwLock<HashMap<String, String>>>),
    //ExternalDB(Arc<redis::aio::MultiplexedConnection>),
    None,
}

/* #[derive(Clone)]
pub struct DB_Object {
    pub storage: StorageEnum, 
} */
// pub struct DB_Object {
//     pub storage: Box<dyn Storage>, // Store a trait object
// }

pub trait Storage {
    async fn set(&self, key: &str, value: &str) -> Result<(), Box<dyn Error>>;
    async fn get(&self, key: &str) -> Result<Option<String>, Box<dyn Error>>;
}



#[derive(Clone)]
pub struct RedisStorage {
    pool: Arc<redis::aio::MultiplexedConnection>,
}

impl Storage for RedisStorage {
    async fn set(&self, key: &str, value: &str) -> Result<(), Box<dyn Error>> {
        let conn = self.pool.as_ref();
        //let result = redis::cmd("SET").arg(key).arg(value).query_async(&mut conn.clone()).await?;

        // base64 encode the value here?

        redis_client::get_or_set_value_with_retries(redis_client::RedisOperation::Set, &mut conn.clone(), &key, Some(&value))
        .await
        .map_err(|err| {
            // Handle Redis errors
            return err;
        });
        Ok(())
    }

    async fn get(&self, key: &str) -> Result<Option<String>, Box<dyn Error>> {
        //let mut conn = self.pool.get().await?;
        //let value: Option<String> = redis::cmd("GET").arg(key).query_async(&mut conn).await?;
        //Ok(value)
        let conn = self.pool.as_ref();

            // Attempt to parse the UUID
        let uuid = match Uuid::parse_str(&key) {
            Ok(uuid) => uuid,
            Err(err) => return Err(Box::new(DatabaseError::Generic(format!("Invalid UUID {}", err)))),
        };

        let secret: Result<std::option::Option<std::string::String>, _> = redis_client::get_or_set_value_with_retries(redis_client::RedisOperation::Get, &mut conn.clone(), &uuid.to_string(), None)
        .await
        .map_err(|err| { return DatabaseError::Generic(err.to_string())});
        
        match secret.unwrap() {
            Some(secret) => {
                //let output = format!("Secret:\n{}\n{}\n", value, uuid);
                // TODO: consider base64::URL_SAFE_NO_PAD?
                let decoded_secret = BASE64_STANDARD.decode(secret);// decode(&value); 
                
                match decoded_secret {
                    Ok(decoded) => return Ok(Some(String::from_utf8(decoded).unwrap())),
                    Err(err) => return Err(Box::new(DatabaseError::Generic(format!("Secret Decode Error {}", err)))),
                }
            },
            None => return Err(Box::new(DatabaseError::Generic(format!("Secret error?"))))
        }
    }
}



#[derive(Clone)]
pub struct InMemoryStorage {
    map: Arc<RwLock<HashMap<String, String>>>,
}

impl Storage for InMemoryStorage {
    async fn set(&self, key: &str, value: &str) -> Result<(), Box<dyn Error>> {
        let mut map = self.map.write().await;

        // No need for the extra `match` here
        if map.insert(key.to_string(), value.to_string()).is_some() {
            Err(Box::new(DatabaseError::Generic("Key not found".to_string() ))) // Specific error type
        } else {
            Ok(()) 
        } 
    }

    async fn get(&self, key: &str) -> Result<Option<String>, Box<dyn Error>> {
        let map = self.map.read().await;
        //Ok(map.get(key).cloned()).ok_or(DatabaseError::Generic("Key not found".to_string()));
        let value = map.get(key).ok_or(DatabaseError::Generic("Key not found".to_string() ));
        Ok(Some(value.unwrap().to_string()))
        //return Ok(Some(map.get(key).ok_or(DatabaseError::Generic("Key not found".to_string())).unwrap()))
    }

}





pub async fn init_kv_db(config: &config::Config) -> Result<StorageEnum, Box<dyn Error>> {
    if config.redis_enabled {
        let connection_string = format!("{}:{}", config.redis_server, config.redis_port);
        tracing::debug!("Connecting to Redis at: {}", connection_string);
        let client = redis_client::connect_to_redis(&connection_string).await;

        tracing::debug!("Redis enabled");
        let out = RedisStorage { pool: Arc::new(client) };

        Ok(StorageEnum::Redis(out))
    } else {
        // In-memory database setup

        tracing::debug!("Hashtable enabled");
        //let shared_hashmap = Arc::new(RwLock::new(HashMap::new()));
        let shared_hashmap = Arc::new(RwLock::new(HashMap::new()));
        let out = InMemoryStorage { map: shared_hashmap };
        Ok(StorageEnum::InMemory(out))
    }
}

pub async fn init_user_db(config: &config::Config) -> Result<StorageEnum, Box<dyn Error>>{
    if config.db_persist {
        // TODO implement this
        if config.db_remote {
            panic!("Remote NoSQL database setup not yet implemented")  // Placeholder error
            //Ok(()?) // Placeholder error
        } else {
            //let db_path = Path::new(&config.db_path); 

            // Schema provisioning (if needed)
            //let conn = Connection::open(db_path)?;
            //conn.execute("CREATE TABLE IF NOT EXISTS users (id INTEGER PRIMARY KEY, name TEXT)", [])?;

            //Ok(Some(StorageEnum::NoSQLDB(conn))) //TODO fix
            panic!("NoSQLDB not yet implemented")
        }
    } else {
        // In-memory database setup
        //let shared_hashmap: HashMap<Key, Value> = HashMap::new();
        //let map = Arc::new(RwLock::new(HashMap::new())); // Wrap in Arc<RwLock>
        let shared_hashmap = Arc::new(RwLock::new(HashMap::new()));
        let out = InMemoryStorage { map: shared_hashmap };
        Ok(StorageEnum::InMemory(out))
    }
}


pub fn get_uuid() -> String {
    let uuidf = Uuid::new_v4(); // TODO: turn into uuid7 later for indexing?
    let uuidr = Uuid::new_v4();

    let custom_uid = format!("{}-{}", uuidf, uuidr);
    tracing::debug!("Generated UUID: {}", custom_uid);

    custom_uid
}

/* 
pub fn generate_password(){
    //Temp passwords for people who dont want to make one but want to encrypt their secrets
    use rand::Rng;
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ\
                            abcdefghijklmnopqrstuvwxyz\
                            0123456789)(*&^%$#@!~";
    const PASSWORD_LEN: usize = 30;
    let mut rng = rand::thread_rng();

    let password: String = (0..PASSWORD_LEN)
        .map(|_| {
            let idx = rng.gen_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect();

    println!("{:?}", password);

} */
