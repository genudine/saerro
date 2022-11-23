use rocket_db_pools::{deadpool_redis, Database};

#[derive(Database)]
#[database("redis")]
pub struct RedisPool(deadpool_redis::Pool);
