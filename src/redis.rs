use redis::Commands;


/*pub fn check_redis_connection() -> redis::RedisResult<()> {
    // Connect to Redis
    let client = redis::Client::open("redis://127.0.0.1/")?;
    let mut con = client.get_connection()?;

    // Check the connection by running the PING command
    let _: () = con.ping()?;

    Ok(())
}*/
