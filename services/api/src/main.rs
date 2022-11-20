use core::time;
use std::{ops::Sub, time::SystemTime};

use once_cell::sync::Lazy;
use salvo::cors::Cors;
use salvo::prelude::*;
use serde::{Deserialize, Serialize};

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

#[handler]
async fn info(req: &mut Request, res: &mut Response) {
    let headers: IncomingHeaders = req.parse_headers().unwrap();
    let json = json!({
        "@": "Saerro Listening Post - PlanetSide 2 Live Population API",
        "@GitHub": "https://github.com/genudine/saerro",
        "@Disclaimer": "Genudine Dynamics is not responsible for any damages caused by this software. Use at your own risk.",
        "@Support": "#api-dev in https://discord.com/servers/planetside-2-community-251073753759481856",
        "Worlds": {
            "Connery": format!("https://{}/w/1", headers.host),
            "Miller": format!("https://{}/w/10", headers.host),
            "Cobalt": format!("https://{}/w/13", headers.host),
            "Emerald": format!("https://{}/w/17", headers.host),
            "Jaeger": format!("https://{}/w/19", headers.host),
            "SolTech": format!("https://{}/w/40", headers.host),
            "Genudine": format!("https://{}/w/1000", headers.host),
            "Ceres": format!("https://{}/w/2000", headers.host),
        },
        "All Worlds": format!("https://{}/m/?ids=1,10,13,17,19,40,1000,2000", headers.host),
    });

    res.render(serde_json::to_string_pretty(&json).unwrap());
}

#[handler]
async fn get_world(req: &mut Request, res: &mut Response) {
    let world_id: String = req.param("worldID").unwrap();
    let response = get_world_pop(world_id).await;

    res.render(Json(response));
}

#[handler]
async fn get_world_multi(req: &mut Request, res: &mut Response) {
    let world_ids_raw = req.query::<String>("ids").unwrap();
    let world_ids: Vec<&str> = world_ids_raw.split(",").collect();

    let mut response = MultipleWorldPopulation { worlds: Vec::new() };

    for world_id in world_ids {
        response
            .worlds
            .push(get_world_pop(world_id.to_string()).await);
    }

    res.render(Json(response));
}

async fn get_world_pop(world_id: String) -> WorldPopulation {
    let mut con = REDIS_CLIENT.get_connection().unwrap();

    let filter_timestamp = SystemTime::now()
        .sub(time::Duration::from_secs(60 * 15))
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let (vs, nc, tr): (u32, u32, u32) = redis::pipe()
        .zcount(format!("{}/{}", world_id, 1), filter_timestamp, "+inf")
        .zcount(format!("{}/{}", world_id, 2), filter_timestamp, "+inf")
        .zcount(format!("{}/{}", world_id, 3), filter_timestamp, "+inf")
        .query(&mut con)
        .unwrap();

    let total = tr + vs + nc;

    let response = WorldPopulation {
        world_id: world_id.parse().unwrap(),
        total,
        factions: Factions { tr, nc, vs },
    };

    response
}

#[tokio::main]
async fn main() {
    let port = ::std::env::var("PORT").unwrap_or("7878".to_string());
    let addr = format!("127.0.0.1:{}", port);

    let cors_handler = Cors::builder()
        .allow_any_origin()
        .allow_method("GET")
        .build();

    println!("Listening on http://localhost:{}", port);

    let router = Router::new()
        .hoop(cors_handler)
        .push(Router::with_path("/").get(info))
        .push(Router::with_path("/w/<worldID>").get(get_world))
        .push(Router::with_path("/m/").get(get_world_multi));
    Server::new(TcpListener::bind(&addr)).serve(router).await;
}
