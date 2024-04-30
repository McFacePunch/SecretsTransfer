// database interrface for handling both local in-memeory and remote nosql databases cleanly.
use std::fmt;
use std::{collections::HashMap, error::{self, Error}};
use uuid::Uuid;
use rusqlite::{Connection, Result};//, Error};
use std::path::Path;

use crate::config;
use crate::redis_client;



type Key = String;
type Value = String;

#[derive(Debug)]
enum DatabaseError {
    Generic(String),
}

impl std::error::Error for DatabaseError {}

impl std::error::Error for DBError {}

#[derive(Debug)]
enum DBError {
    DatabaseUnavailable,
    DatabaseError(DatabaseError),
}

impl fmt::Display for DatabaseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DatabaseError::Generic(message) => write!(f, "Database error: {}", message),
        }
    }
}

impl fmt::Display for DBError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DBError::DatabaseUnavailable => write!(f, "Database unavailable"),
            DBError::DatabaseError(err) => write!(f, "Database operation failed: {}", err),
        }
    }
}

// Trait for your database operations
trait DatabaseHandler {
    async fn store_data(self, key: &str, data: &str) -> Result<(), DatabaseError>;
    async fn retrieve_data(&self, key: &str) -> Result<&str, DatabaseError>;
}

pub struct InMemoryDatabaseHandler {
    pub map: HashMap<String, String>,
}

/* impl DatabaseHandler for InMemoryDatabaseHandler {
    async fn store_data(self, key: &str, value: &str) -> Result<(), DatabaseError> { 
        let mut map = self.map.lock().unwrap();
        self.map.insert(key.to_string(), value.to_string());
        Ok(()) 
    }

    async fn retrieve_data(&self, key: &str) -> Result<Option<String>, DatabaseError> { 
        let data = self.map.lock().unwrap().get(key).cloned();
        Ok(data)
    }
} */

impl DatabaseHandler for InMemoryDatabaseHandler {
    async fn store_data(mut self, key: &str, value: &str) -> Result<(), DatabaseError> { 
        //let mut map = self.map;
        //map.insert(key.to_string(), value.to_string());
        self.map.insert(key.to_string(), value.to_string());
        Ok(()) 
    }

    async fn retrieve_data(&self, key: &str) -> Result<&str, DatabaseError> {
        let data = self.map.get(key).unwrap().as_str();
        Ok(data)
    }
}

struct RedisDatabaseHandler {
    connection: redis::aio::MultiplexedConnection // Store the Redis client
}

impl DatabaseHandler for RedisDatabaseHandler {
    async fn store_data(self, key: &str, value: &str) -> Result<(), DatabaseError> {
        let connection = &self.connection; // TODO fix this
        redis_client::get_or_set_value_with_retries(redis_client::RedisOperation::Set, &mut connection.clone(), key, Some(value));
        Ok(()) 
    }

    async fn retrieve_data(&self, key: &str) -> Result<&str, DatabaseError> { 
        println!("Retrieving data from Redis with key: {}", key);
        // Mock implementation
        Ok("Mocked data")
    }
}

struct RusqliteDatabaseHandler {
    connection: rusqlite::Connection,
}

impl DatabaseHandler for RusqliteDatabaseHandler {
    async fn store_data(self, key: &str, value: &str) -> Result<(), DatabaseError> {
        // Insert the key-value pair into the database
        self.connection.execute("INSERT INTO users (username, password) VALUES (?, ?)", &[&key, &value]).unwrap();
        Ok(())
    }

    async fn retrieve_data(&self, key: &str) -> Result<&str, DatabaseError> {
        // Retrieve the value from the database
        let mut stmt = self.connection.prepare("SELECT password FROM users WHERE username = ?").unwrap();
        //let row = stmt.query_row([key], |row| row.get(0)).unwrap();
        //Ok(row)
        Ok("NOOP") // TODO fix
    }
}

// TODO
// create the full path for the database
/*

Userdb

main
	do kvDB init
		mem OR redis and return it

	do userDB init: (can be none)
		extern OR mem OR none and return it 



*/

/* impl DatabaseHandler for InMemoryDatabaseHandler {
    async fn store_data(&self, key: &str, value: &str) -> Result<(), DatabaseError> {
        let mut map = self.map.lock().unwrap();
        map.insert(key.to_string(), value.to_string());
        Ok(())
    }

    async fn retrieve_data(&self, key: &str) -> Result<Option<String>, DatabaseError> {
        let map = self.map.lock().unwrap();
        let data = map.get(key).cloned();
        Ok(data)
    }
}


impl DatabaseHandler for RedisDatabaseHandler {
    async fn store_data(&self, key: &str, value: &str) -> Result<(), DatabaseError> {
        let connection = &self.connection;
        redis_client::get_or_set_value_with_retries(
            redis_client::RedisOperation::Set,
            &mut connection.clone(),
            key,
            Some(value),
        );
        Ok(())
    }

    async fn retrieve_data(&self, key: &str) -> Result<Option<String>, DatabaseError> {
        println!("Retrieving data from Redis with key: {}", key);
        // Mock implementation
        Ok(Some("Mocked data".to_string()))
    }
}
 */
// TODO
// create the full path for the database

#[derive(Clone)]
pub enum StorageEnum {
    InMemory(HashMap<String, String>),
    ExternalDB(redis::aio::MultiplexedConnection),
    //NoSQLDB(rusqlite::Connection),
    None,
}

#[derive(Clone)]
pub struct DBStates {
    pub value_store: StorageEnum,
    pub user_db: Option<StorageEnum>,
}

// A trait that abstracts the operations
trait KeyValueStore {
    fn insert(&self, key: String, value: i32);
    // Add other necessary methods
}

// In-memory implementation of the KeyValueStore
pub struct InMemoryStore {
    inner: HashMap<String, i32>,
}

impl KeyValueStore for InMemoryStore {
    fn insert(&self, key: String, value: i32) {
        //let mut map = &self.inner;
        //map.insert(key, value);
        tracing::error!("Inserting into InMemoryStore not implemented");
    }
}

impl KeyValueStore for RedisStore {
    fn insert(&self, key: String, value: i32) {
        // Here you would interact with Redis
    }
}

// Define a struct for the Redis store
struct RedisStore {
    client: redis::aio::MultiplexedConnection,
}

impl RedisStore {
    // Asynchronous function to create a new RedisStore
    async fn new(config: &config::Config) -> Result<Self, Box<dyn Error>> {
        let connection_string = format!("{}:{}", config.redis_server, config.redis_port);
        tracing::debug!("Connecting to Redis at: {}", connection_string);

        // Create a client
        let client = redis::Client::open(connection_string)?;

        // Asynchronously connect to Redis and get a MultiplexedConnection
        let multiplexed_conn = client.get_multiplexed_tokio_connection().await?;

        tracing::debug!("Redis enabled");
        Ok(RedisStore {
            client: multiplexed_conn,
        })
    }
}

pub async fn init_kv_db(config: &config::Config) -> Result<StorageEnum, Box<dyn Error>> {
    if config.redis_enabled {
        let connection_string = format!("{}:{}", config.redis_server, config.redis_port);
        tracing::debug!("Connecting to Redis at: {}", connection_string);
        let client = redis_client::connect_to_redis(&connection_string).await;

        tracing::debug!("Redis enabled");
        Ok(StorageEnum::ExternalDB(client))
    } else {
        // In-memory database setup
        let shared_hashmap = HashMap::new();
        Ok(StorageEnum::InMemory(shared_hashmap))
    }
}

pub async fn init_user_db(config: &config::Config) -> Result<Option<StorageEnum>, Box<dyn Error>>{
    if config.db_persist {
        // TODO implement this
        if config.db_remote {
            panic!("Remote NoSQL database setup not yet implemented")  // Placeholder error
            //Ok(()?) // Placeholder error
        } else {
            let db_path = Path::new(&config.db_path); 

            // Schema provisioning (if needed)
            let conn = Connection::open(db_path)?;
            conn.execute("CREATE TABLE IF NOT EXISTS users (id INTEGER PRIMARY KEY, name TEXT)", [])?;

            //Ok(Some(StorageEnum::NoSQLDB(conn))) //TODO fix
            panic!("NoSQLDB not yet implemented")
        }
    } else {
        // In-memory database setup
        //let shared_hashmap: HashMap<Key, Value> = HashMap::new();
        let shared_hashmap = StorageEnum::InMemory( HashMap::new());
        Ok(Some(shared_hashmap))
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
// database interrface for handling both local in-memeory and remote nosql databases cleanly.
use std::fmt;
use std::{collections::HashMap, error::{self, Error}};
use uuid::Uuid;
use rusqlite::{Connection, Result};//, Error};
use std::sync::{Arc, Mutex};

use redis::aio::MultiplexedConnection;

use redis::Client;

use std::path::Path;

use crate::config;
use crate::redis_client;



type Key = String;
type Value = String;

type SharedHashMap = Arc<Mutex<HashMap<Key, Value>>>;

#[derive(Debug)]
enum DatabaseError {
    Generic(String),
}

impl std::error::Error for DatabaseError {}

impl std::error::Error for DBError {}

#[derive(Debug)]
enum DBError {
    DatabaseUnavailable,
    DatabaseError(DatabaseError),
}

impl fmt::Display for DatabaseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DatabaseError::Generic(message) => write!(f, "Database error: {}", message),
        }
    }
}

impl fmt::Display for DBError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DBError::DatabaseUnavailable => write!(f, "Database unavailable"),
            DBError::DatabaseError(err) => write!(f, "Database operation failed: {}", err),
        }
    }
}

// Trait for your database operations
trait DatabaseHandler {
    async fn store_data(&self, key: &str, data: &str) -> Result<(), DatabaseError>;
    async fn retrieve_data(&self, key: &str) -> Result<Option<String>, DatabaseError>;
}
// Implementations for an InMemory database handler
pub struct InMemoryDatabaseHandler {
    map: Mutex<HashMap<String, String>>,
}

impl DatabaseHandler for InMemoryDatabaseHandler {
    async fn store_data(&self, key: &str, value: &str) -> Result<(), DatabaseError> { 
        let mut map = self.map.lock().unwrap();
        map.insert(key.to_string(), value.to_string());
        Ok(()) 
    }

    async fn retrieve_data(&self, key: &str) -> Result<Option<String>, DatabaseError> { 
        let map = self.map.lock().unwrap();
        let data = map.get(key).cloned();
        Ok(data)
    }
}

struct RedisDatabaseHandler {
    connection: MultiplexedConnection // Store the Redis client
}
impl DatabaseHandler for RedisDatabaseHandler {
    async fn store_data(&self, key: &str, value: &str) -> Result<(), DatabaseError> {
        let connection = &self.connection;
        redis_client::get_or_set_value_with_retries(redis_client::RedisOperation::Set, &mut connection.clone(), key, Some(value));
        Ok(()) 
    }

    async fn retrieve_data(&self, _key: &str) -> Result<Option<String>, DatabaseError> { 
        println!("Retrieving data from Redis with key: {}", _key);
        // Mock implementation
        Ok(Some("Mocked data".to_string()))
    }
}






// TODO
// create the full path for the database
/*

Userdb

main
	do kvDB init
		mem OR redis and return it

	do userDB init: (can be none)
		extern OR mem OR none and return it 



*/

/* impl DatabaseHandler for InMemoryDatabaseHandler {
    async fn store_data(&self, key: &str, value: &str) -> Result<(), DatabaseError> {
        let mut map = self.map.lock().unwrap();
        map.insert(key.to_string(), value.to_string());
        Ok(())
    }

    async fn retrieve_data(&self, key: &str) -> Result<Option<String>, DatabaseError> {
        let map = self.map.lock().unwrap();
        let data = map.get(key).cloned();
        Ok(data)
    }
}


impl DatabaseHandler for RedisDatabaseHandler {
    async fn store_data(&self, key: &str, value: &str) -> Result<(), DatabaseError> {
        let connection = &self.connection;
        redis_client::get_or_set_value_with_retries(
            redis_client::RedisOperation::Set,
            &mut connection.clone(),
            key,
            Some(value),
        );
        Ok(())
    }

    async fn retrieve_data(&self, key: &str) -> Result<Option<String>, DatabaseError> {
        println!("Retrieving data from Redis with key: {}", key);
        // Mock implementation
        Ok(Some("Mocked data".to_string()))
    }
}
 */
// TODO
// create the full path for the database

pub enum ValueStore<T> {
    InMemory(Arc<Mutex<HashMap<String, String>>>),
    ExternalDB(Option<String, T>),
}
,
}

#[derive(Clone)]
pub struct DBStates<T> {
    pub value_store: Option<Arc<Mutex<ValueStore<String>>>>,
    pub user_db: Option<Arc<ValueStore<T>>>,
}

// A trait that abstracts the operations
trait KeyValueStore {
    fn insert(&self, key: String, value: i32);
    // Add other necessary methods
}

// In-memory implementation of the KeyValueStore
struct InMemoryStore {
    inner: Mutex<HashMap<String, i32>>,
}

impl KeyValueStore for InMemoryStore {
    fn insert(&self, key: String, value: i32) {
        let mut map = self.inner.lock().unwrap();
        map.insert(key, value);
    }
}

impl KeyValueStore for RedisStore {
    fn insert(&self, key: String, value: i32) {
        // Here you would interact with Redis
    }
}

// Define a struct for the Redis store
struct RedisStore {
    client: MultiplexedConnection,
}

impl RedisStore {
    // Asynchronous function to create a new RedisStore
    async fn new(config: &config::Config) -> Result<Self, Box<dyn Error>> {
        let connection_string = format!("{}:{}", config.redis_server, config.redis_port);
        tracing::debug!("Connecting to Redis at: {}", connection_string);

        // Create a client
        let client = redis::Client::open(connection_string)?;

        // Asynchronously connect to Redis and get a MultiplexedConnection
        let multiplexed_conn = client.get_multiplexed_tokio_connection().await?;

        tracing::debug!("Redis enabled");
        Ok(RedisStore {
            client: multiplexed_conn,
        })
    }
}

pub async fn init_kv_db(config: &config::Config) -> Result<Arc<dyn KeyValueStore>, Box<dyn Error>> {
    if config.redis_enabled {
        let connection_string = format!("{}:{}", config.redis_server, config.redis_port);
        tracing::debug!("Connecting to Redis at: {}", connection_string);
        let client = redis_client::connect_to_redis(&connection_string).await;

        tracing::debug!("Redis enabled");
        Ok(Arc::new(RedisStore { client }))
    } else {
        // In-memory database setup
        let shared_hashmap = Arc::new(InMemoryStore { inner: Mutex::new(HashMap::new()) });
        Ok(shared_hashmap)
    }
}

pub fn get_uuid() -> String {
    let uuidf = Uuid::new_v4(); // TODO: turn into uuid7 later for indexing?
    let uuidr = Uuid::new_v4();

    let custom_uid = format!("{}-{}", uuidf, uuidr);
    tracing::debug!("Generated UUID: {}", custom_uid);

    custom_uid
}


























struct RusqliteKeyValueStore {
    conn: Connection,
}

impl KeyValueStore for RusqliteKeyValueStore {
    fn insert(&self, key: String, value: i32) {
        // Insert the key-value pair into the database
        self.conn.execute("INSERT INTO users (username, password) VALUES (?, ?)", &[&key, &value]).unwrap();
    }

    // Add other necessary methods
}

/* pub fn init_user_db<T>(config: &config::Config) -> Result<ValueStore<T>, Box<dyn Error>>{
//-> Result<Arc<Mutex<HashMap<String, i32>>>, Box<dyn Error>> { // Adjusted return type
    if config.db_persist {
        // TODO implement this
        if config.db_remote {
            panic!("Remote NoSQL database setup not yet implemented")  // Placeholder error
            //Ok(()?) // Placeholder error
        } else {
            let db_path = Path::new(&config.db_path); 

            // Schema provisioning (if needed)
            let conn = Connection::open(db_path)?;
            conn.execute("CREATE TABLE IF NOT EXISTS users (id INTEGER PRIMARY KEY, name TEXT)", [])?;

            Ok(ValueStore::ExternalDB(Arc::new(Mutex::new(conn)))) 
        }
    } else {
        // In-memory database setup
        //let db = HashMap::new();
        let shared_hashmap = Arc::new(Mutex::new(HashMap::new()));
        //Ok(shared_hashmap)
        Ok(ValueStore::InMemory(shared_hashmap))
    }
} */

// Update the `init_user_db` function to return the new type
pub fn init_user_db<T>(config: &config::Config) -> Result<ValueStore<T>, Box<dyn Error>>{
    if config.db_persist {
        // TODO implement this
        if config.db_remote {
            panic!("Remote NoSQL database setup not yet implemented")  // Placeholder error
            //Ok(()?) // Placeholder error
        } else {
            let db_path = Path::new(&config.db_path); 

            // Schema provisioning (if needed)
            let conn = Connection::open(db_path)?;
            conn.execute("CREATE TABLE IF NOT EXISTS users (id INTEGER PRIMARY KEY, name TEXT)", [])?;

            Ok(ValueStore::ExternalDB(conn)) 
        }
    } else {
        // In-memory database setup
        //let db = HashMap::new();
        let shared_hashmap = Arc::new(Mutex::new(HashMap::new()));
        //Ok(shared_hashmap)
        Ok(ValueStore::InMemory(shared_hashmap))
    }
}

pub async fn old_init_kv_db(config: &config::Config) -> Result<Arc<dyn KeyValueStore>, Box<dyn Error>> {
    if config.redis_enabled {
        let connection_string = format!("{}:{}", config.redis_server, config.redis_port);
        tracing::debug!("Connecting to Redis at: {}", connection_string);
        let client = redis_client::connect_to_redis(&connection_string).await;

        tracing::debug!("Redis enabled");
        Ok(Arc::new(RedisStore { client }))
    } else {
        // In-memory database setup
        let shared_hashmap = Arc::new(InMemoryStore { inner: Mutex::new(HashMap::new()) });
        Ok(shared_hashmap)
    }
}

pub fn old_get_uuid() -> String {
    let uuidf = Uuid::new_v4(); // TODO: turn into uuid7 later for indexing?
    let uuidr = Uuid::new_v4();

    let custom_uid = format!("{}-{}", uuidf, uuidr);
    tracing::debug!("Generated UUID: {}", custom_uid);

    custom_uid
}

/* use rusqlite::{Connection, Error as SqliteError};
use std::convert::From;

#[derive(Debug)]
pub enum DbError {
    InitError(String),
    SqliteError(SqliteError),
}

impl From<SqliteError> for DbError {
    fn from(err: SqliteError) -> Self {
        DbError::SqliteError(err)
    }
}

pub const DB_PATH: &str = "path_to_your_database.db";

pub fn init_db() -> Result<(), DbError> {
    let conn = Connection::open(DB_PATH).map_err(|e| DbError::InitError(format!("Unable to open the database: {}", e)))?;
    conn.execute(
        "CREATE TABLE IF NOT EXISTS users (
             username TEXT PRIMARY KEY,
             password TEXT NOT NULL
         )",
        [],
    ).map_err(DbError::from)?;

    Ok(())
}
 */


*/