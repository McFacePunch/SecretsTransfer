use std::time::Duration;
use redis::{Client, RedisError};
use redis::aio::MultiplexedConnection;

extern crate redis;

pub async fn connect_to_redis(redis_url: &str) -> MultiplexedConnection {
    let client = Client::open(redis_url);
    //let mut con: MultiplexedConnection = client.get_multiplexed_async_connection().await;
    let mut con: MultiplexedConnection = match client {
        Ok(client) => client.get_multiplexed_async_connection().await.unwrap(),
        Err(e) => {
            tracing::debug!("Exiting due to Redis error:\n{}", e);
            panic!("Exiting due to Redis error:\n{}", e);
        }
    };

    // do ping to test connection
    let value: String = redis::cmd("PING").query_async(&mut con).await.unwrap();
    tracing::debug!("Redis PING response: {:?}", value);
    
    // examples
    //con.set(key, value).await?;
    //let value = con.get(key).await?;

    con
}

pub async fn get_value_from_redis(con: &mut MultiplexedConnection, key: &str) -> Result<String, RedisError> {
    let value: String = redis::cmd("GET").arg(key).query_async(con).await?;
    Ok(value)
}

pub async fn set_value_in_redis(con: &mut MultiplexedConnection, key: &str, value: &str) -> Result<(), RedisError> {
    redis::cmd("SET")
        .arg(key)
        .arg(value)
        .query_async(con)
        .await?;
    Ok(())
}

pub enum RedisOperation {
    Get,
    Set,
}

pub async fn get_or_set_value_with_retries(
    operation: RedisOperation,
    con: &mut MultiplexedConnection,
    key: &str, 
    value: Option<&str>, 
    ) -> Result<Option<String>, RedisError> { 

    let retry_delay = Duration::from_secs(1);
    let max_retries = 3;
    let mut retries = 0;
    
    while retries < max_retries {
        match operation {
            RedisOperation::Set => { 
                match set_value_in_redis(con, key, value.unwrap_or("")).await {
                    Ok(()) => return Ok(None), // Return None for Set
                    Err(err) => {
                        retries += 1;
                        eprintln!("Redis SET error {}\nRetrying... ({} attempts left)",err , max_retries - retries);
                        tokio::time::sleep(retry_delay).await;
                    }
                }
            }, 
            RedisOperation::Get => {
                match get_value_from_redis(con, key).await { 
                    Ok(result) => return Ok(Some(result)),
                    Err(err) => {
                        retries += 1;
                        eprintln!("Redis GET error {}\nRetrying... ({} attempts left)",err , max_retries - retries);
                        tokio::time::sleep(retry_delay).await;
                    }
                } 
            }
        }
    }
    Err(RedisError::from((redis::ErrorKind::ClientError, "Max retries exceeded")))
}

// TODO Improve this?
pub async fn get_value_with_retries(
    con: &mut MultiplexedConnection,
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