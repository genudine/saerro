pub mod cors;

use core::time;
use once_cell::sync::Lazy;
use rocket::{data::Outcome, http::uri::Host, request::FromRequest, Build, Request, Rocket};
use serde::{Deserialize, Serialize};
use std::{ops::Sub, time::SystemTime};

#[macro_use]
extern crate rocket;
#[macro_use]
extern crate serde_json;

#[derive(Serialize, Deserialize)]
struct IncomingHeaders {
    host: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct Factions {
    tr: u32,
    nc: u32,
    vs: u32,
    ns: u32,
}

#[derive(Serialize, Deserialize, Debug)]
struct WorldPopulation {
    world_id: u32,
    total: u32,
    factions: Factions,
}

#[derive(Serialize, Deserialize, Debug)]
struct MultipleWorldPopulation {
    worlds: Vec<WorldPopulation>,
}

pub static REDIS_CLIENT: Lazy<redis::Client> = Lazy::new(|| {
    redis::Client::open(std::env::var("REDIS_ADDR").unwrap_or("redis://localhost:6379".to_string()))
        .unwrap()
});

fn hello(host: String) -> serde_json::Value {
    json!({
        "@": "Saerro Listening Post - PlanetSide 2 Live Population API",
        "@GitHub": "https://github.com/genudine/saerro",
        "@Disclaimer": "Genudine Dynamics is not responsible for any damages caused by this software. Use at your own risk.",
        "@Support": "#api-dev in https://discord.com/servers/planetside-2-community-251073753759481856",
        "Worlds": {
            "Connery": format!("https://{}/w/1", host),
            "Miller": format!("https://{}/w/10", host),
            "Cobalt": format!("https://{}/w/13", host),
            "Emerald": format!("https://{}/w/17", host),
            "Jaeger": format!("https://{}/w/19", host),
            "SolTech": format!("https://{}/w/40", host),
            "Genudine": format!("https://{}/w/1000", host),
            "Ceres": format!("https://{}/w/2000", host),
        },
        "All Worlds": format!("https://{}/m/?ids=1,10,13,17,19,40,1000,2000", host),
    })
}

async fn get_world_pop(world_id: String) -> WorldPopulation {
    let mut con = REDIS_CLIENT.get_connection().unwrap();

    let filter_timestamp = SystemTime::now()
        .sub(time::Duration::from_secs(60 * 15))
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let (vs, nc, tr, ns): (u32, u32, u32, u32) = redis::pipe()
        .zcount(format!("wp:{}/{}", world_id, 1), filter_timestamp, "+inf")
        .zcount(format!("wp:{}/{}", world_id, 2), filter_timestamp, "+inf")
        .zcount(format!("wp:{}/{}", world_id, 3), filter_timestamp, "+inf")
        .zcount(format!("wp:{}/{}", world_id, 4), filter_timestamp, "+inf")
        .query(&mut con)
        .unwrap();

    let total = tr + vs + nc;

    let response = WorldPopulation {
        world_id: world_id.parse().unwrap(),
        total,
        factions: Factions { tr, nc, vs, ns },
    };

    response
}

#[get("/w/<world_id>")]
async fn world_pop(world_id: String) -> serde_json::Value {
    let response = get_world_pop(world_id).await;

    json!(response)
}

#[get("/m?<ids>")]
async fn multiple_world_pop(ids: String) -> serde_json::Value {
    let mut response = MultipleWorldPopulation { worlds: vec![] };

    for id in ids.split(",") {
        response.worlds.push(get_world_pop(id.to_string()).await);
    }

    json!(response)
}

#[get("/")]
async fn index() -> serde_json::Value {
    hello("saerro.harasse.rs".to_string())
}

#[launch]
fn rocket() -> Rocket<Build> {
    rocket::build()
        .attach(cors::CORS)
        .mount("/", routes![index, world_pop, multiple_world_pop])
}
