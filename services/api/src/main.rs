pub mod classes;
pub mod cors;
pub mod population;
pub mod redispool;
pub mod vehicles;

use redispool::RedisPool;
use rocket::fairing::AdHoc;
use rocket::{error, Build, Rocket};
use rocket_db_pools::deadpool_redis::redis::{cmd, pipe};
use rocket_db_pools::{Connection, Database};

use serde::{Deserialize, Serialize};

#[macro_use]
extern crate rocket;
#[macro_use]
extern crate serde_json;

#[derive(Serialize, Deserialize)]
struct IncomingHeaders {
    host: String,
}

fn hello_world(host: String, world_id: &str) -> serde_json::Value {
    json!({
        "population": format!("https://{}/w/{}", host, world_id),
        "vehicles": format!("https://{}/w/{}/vehicles", host, world_id),
        "classes": format!("https://{}/w/{}/classes", host, world_id),
    })
}

fn hello(host: String) -> serde_json::Value {
    json!({
        "@": "Saerro Listening Post - PlanetSide 2 Live Population API",
        "@GitHub": "https://github.com/genudine/saerro",
        "@Disclaimer": "Genudine Dynamics is not responsible for any damages caused by this software. Use at your own risk.",
        "@Support": "#api-dev in https://discord.com/servers/planetside-2-community-251073753759481856",
        "Worlds": {
            "Connery": hello_world(host.clone(), "1"),
            "Miller": hello_world(host.clone(), "10"),
            "Cobalt": hello_world(host.clone(), "13"),
            "Emerald": hello_world(host.clone(), "17"),
            "Jaeger": hello_world(host.clone(), "19"),
            "SolTech": hello_world(host.clone(), "40"),
            "Genudine": hello_world(host.clone(), "1000"),
            "Ceres": hello_world(host.clone(), "2000"),
        },
        // "All World Population": format!("https://{}/m/?ids=1,10,13,17,19,40,1000,2000", host),
    })
}

#[get("/")]
fn index() -> serde_json::Value {
    hello("saerro.harasse.rs".to_string())
}

#[get("/health")]
async fn health(mut con: Connection<RedisPool>) -> serde_json::Value {
    let (ping, pc, ps4us, ps4eu): (String, bool, bool, bool) = pipe()
        .cmd("PING")
        .get("heartbeat:pc")
        .get("heartbeat:ps4us")
        .get("heartbeat:ps4eu")
        .query_async(&mut *con)
        .await
        .unwrap_or_default();

    json!({
        "status": if ping == "PONG" && pc && ps4us && ps4eu { "ok" } else { "degraded" },
        "redis": ping == "PONG",
        "pc": if pc { "primary" } else { "backup/down" },
        "ps4us": if ps4us { "primary" } else { "backup/down" },
        "ps4eu": if ps4eu { "primary" } else { "backup/down" },
    })
}

#[launch]
fn rocket() -> Rocket<Build> {
    let figment = rocket::Config::figment().merge((
        "databases.redis.url",
        format!(
            "redis://{}:{}",
            std::env::var("REDIS_HOST").unwrap_or("localhost".to_string()),
            std::env::var("REDIS_PORT").unwrap_or("6379".to_string()),
        ),
    ));

    rocket::build()
        .configure(figment)
        .attach(cors::CORS)
        .attach(RedisPool::init())
        .attach(AdHoc::on_ignite("Redis Check", |rocket| async move {
            if let Some(pool) = RedisPool::fetch(&rocket) {
                let mut con = pool.get().await.unwrap();
                let _: () = cmd("PING").query_async(&mut con).await.unwrap();
            } else {
                error!("Redis connection failed");
            }
            rocket
        }))
        .mount(
            "/",
            routes![
                index,
                health,
                population::get_world_pop,
                vehicles::get_vehicles,
                classes::get_classes,
            ],
        )
}
