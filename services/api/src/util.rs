use redis::{aio::MultiplexedConnection, AsyncCommands, FromRedisValue};
use std::{
    ops::Sub,
    time::{Duration, SystemTime},
};

pub async fn zcount<RV: FromRedisValue>(mut con: MultiplexedConnection, key: String) -> RV {
    let filter_timestamp = SystemTime::now()
        .sub(Duration::from_secs(60 * 15))
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    con.zcount::<String, u64, &'static str, RV>(key, filter_timestamp, "+inf")
        .await
        .unwrap()
}
