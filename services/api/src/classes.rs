use crate::redispool::RedisPool;
use core::time;
use rocket_db_pools::deadpool_redis::redis::{pipe, AsyncCommands};
use rocket_db_pools::Connection;
use serde::{Deserialize, Serialize};
use std::ops::Sub;
use std::time::SystemTime;

#[derive(Serialize, Deserialize, Debug)]
struct ClassCounts {
    world_id: String,
    classes: Classes,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Classes {
    light_assault: i32,
    engineer: i32,
    combat_medic: i32,
    heavy_assault: i32,
    infiltrator: i32,
    max: i32,
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

    let classes = fetch_classes(world_id.clone(), &mut con).await;

    // I hate this but it's fast???
    // The type only allows 12 at a time.

    let response = ClassCounts { world_id, classes };

    let out = json!(response);

    con.set_ex::<String, String, ()>(cache_key, out.to_string(), 5)
        .await
        .unwrap();

    out
}

pub async fn fetch_classes(world_id: String, con: &mut Connection<RedisPool>) -> Classes {
    let filter_timestamp = SystemTime::now()
        .sub(time::Duration::from_secs(60 * 15))
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let (light_assault, engineer, combat_medic, heavy_assault, infiltrator, max): (
        i32,
        i32,
        i32,
        i32,
        i32,
        i32,
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
        .query_async(&mut **con)
        .await
        .unwrap();

    Classes {
        light_assault,
        engineer,
        combat_medic,
        heavy_assault,
        infiltrator,
        max,
    }
}
