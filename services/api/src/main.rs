pub mod cors;
pub mod graphql;
pub mod redispool;

use redispool::RedisPool;
use rocket::fairing::AdHoc;
use rocket::response::content::RawHtml;
use rocket::response::status;
use rocket::{error, Build, Rocket};
use rocket_db_pools::deadpool_redis::redis::{cmd, pipe};
use rocket_db_pools::{Connection, Database};

#[macro_use]
extern crate rocket;
#[macro_use]
extern crate serde_json;

#[get("/")]
async fn index() -> RawHtml<String> {
    RawHtml(include_str!("html/index.html").to_string())
}

#[get("/health")]
async fn health(
    mut con: Connection<RedisPool>,
) -> Result<serde_json::Value, status::Custom<serde_json::Value>> {
    let (ping, pc, ps4us, ps4eu): (String, bool, bool, bool) = pipe()
        .cmd("PING")
        .get("heartbeat:pc")
        .get("heartbeat:ps4us")
        .get("heartbeat:ps4eu")
        .query_async(&mut *con)
        .await
        .unwrap_or_default();

    if ping != "PONG" {
        return Err(status::Custom(
            rocket::http::Status::ServiceUnavailable,
            json!({
                "status": "error",
                "message": "Redis is not responding",
            }),
        ));
    }

    Ok(json!({
        "status": if ping == "PONG" && pc && ps4us && ps4eu { "ok" } else { "degraded" },
        "redis": ping == "PONG",
        "pc": if pc { "primary" } else { "backup/down" },
        "ps4us": if ps4us { "primary" } else { "backup/down" },
        "ps4eu": if ps4eu { "primary" } else { "backup/down" },
    }))
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
        .manage(graphql::schema())
        .mount("/", routes![index, health,])
        .mount(
            "/graphql",
            routes![
                graphql::graphiql,
                graphql::playground,
                graphql::playground2,
                graphql::get_graphql,
                graphql::post_graphql
            ],
        )
}
