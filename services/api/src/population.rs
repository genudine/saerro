use core::time;
use std::{ops::Sub, time::SystemTime};

use rocket_db_pools::{deadpool_redis::redis::pipe, Connection};
use serde::{Deserialize, Serialize};

use crate::redispool::RedisPool;

#[derive(Serialize, Deserialize, Debug)]
pub struct Factions {
    tr: u32,
    nc: u32,
    vs: u32,
    ns: u32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct WorldPopulation {
    world_id: u32,
    total: u32,
    factions: Factions,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct MultipleWorldPopulation {
    worlds: Vec<WorldPopulation>,
}

#[get("/w/<world_id>")]
pub async fn get_world_pop(world_id: String, mut con: Connection<RedisPool>) -> serde_json::Value {
    let filter_timestamp = SystemTime::now()
        .sub(time::Duration::from_secs(60 * 15))
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let (vs, nc, tr, ns): (u32, u32, u32, u32) = pipe()
        .zcount(format!("wp:{}/{}", world_id, 1), filter_timestamp, "+inf")
        .zcount(format!("wp:{}/{}", world_id, 2), filter_timestamp, "+inf")
        .zcount(format!("wp:{}/{}", world_id, 3), filter_timestamp, "+inf")
        .zcount(format!("wp:{}/{}", world_id, 4), filter_timestamp, "+inf")
        .query_async(&mut *con)
        .await
        .unwrap();

    let total = tr + vs + nc;

    let response = WorldPopulation {
        world_id: world_id.parse().unwrap(),
        total,
        factions: Factions { tr, nc, vs, ns },
    };

    json!(response)
}
