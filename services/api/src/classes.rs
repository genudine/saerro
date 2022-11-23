use core::time;
use rocket_db_pools::deadpool_redis::redis::{pipe, AsyncCommands};
use rocket_db_pools::Connection;
use serde::{Deserialize, Serialize};
use std::ops::Sub;
use std::time::SystemTime;

use crate::redispool::RedisPool;

#[derive(Serialize, Deserialize, Debug)]
struct ClassCounts {
    world_id: String,
    classes: Classes,
}

#[derive(Serialize, Deserialize, Debug)]
struct Classes {
    light_assault: u32,
    engineer: u32,
    combat_medic: u32,
    heavy_assault: u32,
    infiltrator: u32,
    max: u32,
}

#[get("/w/<world_id>/classes")]
pub async fn get_classes(world_id: String, mut con: Connection<RedisPool>) -> serde_json::Value {
    let cache_key = format!("cache:classes:{}", world_id);

    match con.get::<String, String>(cache_key.clone()).await {
        Ok(cached) => {
            return serde_json::from_str(&cached).unwrap();
        }
        Err(_) => {}
    }

    let filter_timestamp = SystemTime::now()
        .sub(time::Duration::from_secs(60 * 15))
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    // I hate this but it's fast???
    // The type only allows 12 at a time.
    let (light_assault, engineer, combat_medic, heavy_assault, infiltrator, max): (
        u32,
        u32,
        u32,
        u32,
        u32,
        u32,
    ) = pipe()
        .zcount(
            format!("c:{}/{}", world_id, "light_assault"),
            filter_timestamp,
            "+inf",
        )
        .zcount(
            format!("c:{}/{}", world_id, "engineer"),
            filter_timestamp,
            "+inf",
        )
        .zcount(
            format!("c:{}/{}", world_id, "combat_medic"),
            filter_timestamp,
            "+inf",
        )
        .zcount(
            format!("c:{}/{}", world_id, "heavy_assault"),
            filter_timestamp,
            "+inf",
        )
        .zcount(
            format!("c:{}/{}", world_id, "infiltrator"),
            filter_timestamp,
            "+inf",
        )
        .zcount(
            format!("c:{}/{}", world_id, "max"),
            filter_timestamp,
            "+inf",
        )
        .query_async(&mut *con)
        .await
        .unwrap();

    let response = ClassCounts {
        world_id,
        classes: Classes {
            light_assault,
            engineer,
            combat_medic,
            heavy_assault,
            infiltrator,
            max,
        },
    };

    let out = json!(response);

    con.set_ex::<String, String, ()>(cache_key, out.to_string(), 5)
        .await
        .unwrap();

    out
}
