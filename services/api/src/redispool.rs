use rocket_db_pools::{deadpool_redis, Database};

#[derive(Database, Clone)]
#[database("redis")]
pub struct RedisPool(deadpool_redis::Pool);
