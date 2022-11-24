use core::time;
use rocket_db_pools::deadpool_redis::redis::{pipe, AsyncCommands};
use rocket_db_pools::Connection;
use serde::{Deserialize, Serialize};
use std::ops::Sub;
use std::time::SystemTime;

use crate::redispool::RedisPool;

#[derive(Serialize, Deserialize, Debug)]
struct VehiclesCounts {
    world_id: String,
    total: u32,
    vehicles: Vehicles,
}

#[derive(Serialize, Deserialize, Debug)]
struct Vehicles {
    flash: u32,
    sunderer: u32,
    lightning: u32,
    scythe: u32,
    vanguard: u32,
    prowler: u32,
    reaver: u32,
    mosquito: u32,
    galaxy: u32,
    valkyrie: u32,
    liberator: u32,
    ant: u32,
    harasser: u32,
    dervish: u32,
    chimera: u32,
    javelin: u32,
    corsair: u32,
    magrider: u32,
}

#[get("/w/<world_id>/vehicles")]
pub async fn get_vehicles(world_id: String, mut con: Connection<RedisPool>) -> serde_json::Value {
    let cache_key = format!("cache:vehicles:{}", world_id);

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
    let (
        flash,
        sunderer,
        lightning,
        scythe,
        vanguard,
        prowler,
        reaver,
        mosquito,
        galaxy,
        valkyrie,
        liberator,
        ant
    ): (u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, u32) = pipe()
        .zcount(
            format!("v:{}/{}", world_id, "flash"),
            filter_timestamp,
            "+inf",
        )
        .zcount(
            format!("v:{}/{}", world_id, "sunderer"),
            filter_timestamp,
            "+inf",
        )
        .zcount(
            format!("v:{}/{}", world_id, "lightning"),
            filter_timestamp,
            "+inf",
        )
        .zcount(
            format!("v:{}/{}", world_id, "scythe"),
            filter_timestamp,
            "+inf",
        )
        .zcount(
            format!("v:{}/{}", world_id, "vanguard"),
            filter_timestamp,
            "+inf",
        )
        .zcount(
            format!("v:{}/{}", world_id, "prowler"),
            filter_timestamp,
            "+inf",
        )
        .zcount(
            format!("v:{}/{}", world_id, "reaver"),
            filter_timestamp,
            "+inf",
        )
        .zcount(
            format!("v:{}/{}", world_id, "mosquito"),
            filter_timestamp,
            "+inf",
        )
        .zcount(
            format!("v:{}/{}", world_id, "galaxy"),
            filter_timestamp,
            "+inf",
        )
        .zcount(
            format!("v:{}/{}", world_id, "valkyrie"),
            filter_timestamp,
            "+inf",
        )
        .zcount(
            format!("v:{}/{}", world_id, "liberator"),
            filter_timestamp,
            "+inf",
        )
        .zcount(
            format!("v:{}/{}", world_id, "ant"),
            filter_timestamp,
            "+inf",
        )
        .query_async(&mut *con)
        .await
        .unwrap();

    let (harasser, dervish, chimera, javelin, corsair, magrider): (u32, u32, u32, u32, u32, u32) = pipe()
        .zcount(
            format!("v:{}/{}", world_id, "harasser"),
            filter_timestamp,
            "+inf",
        )
        .zcount(
            format!("v:{}/{}", world_id, "dervish"),
            filter_timestamp,
            "+inf",
        )
        .zcount(
            format!("v:{}/{}", world_id, "chimera"),
            filter_timestamp,
            "+inf",
        )
        .zcount(
            format!("v:{}/{}", world_id, "javelin"),
            filter_timestamp,
            "+inf",
        )
        .zcount(
            format!("v:{}/{}", world_id, "corsair"),
            filter_timestamp,
            "+inf",
        )
        .zcount(
            format!("v:{}/{}", world_id, "magrider"),
            filter_timestamp,
            "+inf",
        )
        .query_async(&mut *con)
        .await
        .unwrap();

    let total = flash
        + sunderer
        + lightning
        + scythe
        + vanguard
        + prowler
        + reaver
        + mosquito
        + galaxy
        + valkyrie
        + liberator
        + ant
        + harasser
        + dervish
        + chimera
        + javelin
        + corsair
        + magrider;

    let response = VehiclesCounts {
        world_id,
        total,
        vehicles: Vehicles {
            flash,
            sunderer,
            lightning,
            scythe,
            vanguard,
            prowler,
            reaver,
            mosquito,
            galaxy,
            valkyrie,
            liberator,
            ant,
            harasser,
            dervish,
            chimera,
            javelin,
            corsair,
            magrider
        },
    };

    let out = json!(response);

    con.set_ex::<String, String, ()>(cache_key, out.to_string(), 5)
        .await
        .unwrap();

    out
}
