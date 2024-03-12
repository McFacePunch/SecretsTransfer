
use std::time::Duration;
use redis::{Client, RedisError};
use redis::AsyncCommands;

extern crate redis;


pub async fn connect_to_redis(redis_url: &str) -> Result<Client, RedisError> {
    let client = Client::open(redis_url)?;
    let mut con: redis::aio::MultiplexedConnection = client.get_multiplexed_async_connection().await?;

    // Test Connectivity (asynchronous)
    let _: () = con.get("test_key").await?; 

    Ok(client) 
}

// Function to retrieve a value from Redis
pub async fn get_value_from_redis(con: &mut redis::aio::MultiplexedConnection, key: &str) -> Result<String, RedisError> {
    let value: String = redis::cmd("GET").arg(key).query_async(con).await?;
    Ok(value)
}

// Improve this with one function to retry both get and set
pub async fn get_value_with_retries(
    con: &mut redis::aio::MultiplexedConnection,
    key: &str,
) -> Result<String, RedisError> {
    let retry_delay = Duration::from_secs(1);
    let max_retries = 3;
    let mut retries = 0;

    while retries < max_retries {
        match get_value_from_redis(con, key).await {
            Ok(value) => return Ok(value),
            Err(err) => {
                retries += 1;
                eprintln!("Redis GET error {}\nRetrying... ({} attempts left)",err , max_retries - retries);
                tokio::time::sleep(retry_delay).await;
            }
        }
    }
    Err(RedisError::from((redis::ErrorKind::ClientError, "Max retries exceeded")))
}




pub async fn set_value_in_redis(con: &mut redis::aio::MultiplexedConnection, key: &str, value: &str) -> Result<(), RedisError> {
    redis::cmd("SET")
        .arg(key)
        .arg(value)
        .query_async(con)
        .await?;

    Ok(())
}

pub async fn set_value_with_retries(
    con: &mut redis::aio::MultiplexedConnection,
    key: &str, 
    value: &str, 
) -> Result<(), RedisError> {
    let retry_delay = Duration::from_secs(1);
    let max_retries = 3;
    let mut retries = 0;
    
    while retries < max_retries {
        match set_value_in_redis(con, key, value).await {
            Ok(()) => return Ok(()),
            Err(err) => {
                retries += 1;
                eprintln!("Redis GET error {}\nRetrying... ({} attempts left)",err , max_retries - retries);
                tokio::time::sleep(retry_delay).await;
            }
        }
    }
    Err(RedisError::from((redis::ErrorKind::ClientError, "Max retries exceeded")))
}


/* pub fn redis_GET_string(client: redis::Client, value: String) -> redis::RedisResult<()> {
    // Connect to Redis
    match redis::cmd("GET").arg("my_key").query(&mut con) {
        Ok(val) => println!("The value is: {}", val),
        Err(err) => {
            if err.kind() == redis::ErrorKind::ConnectionError {
                redis::ErrorKind::TryAgain;
                // Specific handling for connection errors
                eprintln!("Connection lost: {}", err);
                // ... retry logic ...
            } else {
                eprintln!("Other error: {}", err);
            }
        }
    }

    Ok(())
} */