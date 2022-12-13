use async_once::AsyncOnce;
use axum::{routing::get, Json, Router};
use futures::{pin_mut, FutureExt};
use futures_util::StreamExt;
use lazy_static::lazy_static;
use serde::Deserialize;
use serde_aux::prelude::*;
use serde_json::json;
use sqlx::{postgres::PgPoolOptions, query};
use std::{env, net::SocketAddr};
use tokio::task::JoinSet;
use tokio_tungstenite::{connect_async, tungstenite::Message};

mod translators;

lazy_static! {
    static ref WS_ADDR: String = env::var("WS_ADDR").unwrap_or_default();
    static ref PG: AsyncOnce<sqlx::PgPool> = AsyncOnce::new(async {
        let db_url = std::env::var("DATABASE_URL")
            .unwrap_or("postgres://saerrouser:saerro321@localhost:5432/data".to_string());
        PgPoolOptions::new().connect(&db_url).await.unwrap()
    });
}

async fn send_init(tx: futures::channel::mpsc::UnboundedSender<Message>) {
    let worlds_raw = env::var("WORLDS").unwrap_or("all".to_string());
    let worlds: Vec<&str> = worlds_raw.split(',').collect();

    // Send setup message
    let setup_msg = json!({
        "action": "subscribe",
        "worlds": worlds,
        "eventNames": ["Death", "VehicleDestroy"],
        "characters": ["all"],
        "logicalAndCharactersWithWorlds": true,
        "service": "event",
    });

    tx.unbounded_send(Message::text(setup_msg.to_string()))
        .unwrap();

    println!("[ws] Sent setup message");
}

#[derive(Clone)]
struct PopEvent {
    world_id: i32,
    team_id: i32,
    character_id: String,
    zone_id: i32,
}

struct VehicleEvent {
    world_id: i32,
    vehicle_id: String,
    character_id: String,
    zone_id: i32,
    team_id: i32,
}

struct ClassEvent {
    world_id: i32,
    character_id: String,
    loadout_id: String,
    zone_id: i32,
    team_id: i32,
}

struct AnalyticsEvent {
    world_id: i32,
    event_name: String,
}

async fn track_pop(pop_event: PopEvent) {
    // println!("[ws/track_pop]");
    let pool = PG.get().await;

    let PopEvent {
        world_id,
        team_id,
        character_id,
        zone_id,
    } = pop_event;

    query("INSERT INTO players (time, character_id, world_id, faction_id, zone_id) VALUES (now(), $1, $2, $3, $4);")
        .bind(character_id)
        .bind(world_id)
        .bind(team_id)
        .bind(zone_id)
        .execute(pool)
        .await
        .unwrap();
}

async fn track_vehicle(vehicle_event: VehicleEvent) {
    // println!("[ws/track_vehicle]");
    let pool = PG.get().await;

    let VehicleEvent {
        world_id,
        vehicle_id,
        zone_id,
        character_id,
        team_id,
    } = vehicle_event;

    let vehicle_name = translators::vehicle_to_name(vehicle_id.as_str());

    if vehicle_name == "unknown" {
        return;
    }

    query("INSERT INTO vehicles (time, character_id, world_id, faction_id, zone_id, vehicle_id) VALUES (now(), $1, $2, $3, $4, $5);")
    .bind(character_id)
    .bind(world_id)
    .bind(team_id)
    .bind(zone_id)
    .bind(vehicle_name)
    .execute(pool)
    .await
    .unwrap();
}

async fn track_class(class_event: ClassEvent) {
    // println!("[ws/track_class]");
    let pool = PG.get().await;

    let ClassEvent {
        world_id,
        character_id,
        loadout_id,
        zone_id,
        team_id,
    } = class_event;

    let class_name = translators::loadout_to_class(loadout_id.as_str());

    if class_name == "unknown" {
        return;
    }

    query(
        "INSERT INTO classes (
        time, 
        character_id, 
        world_id, 
        faction_id, 
        zone_id, 
        class_id
    ) VALUES (now(), $1, $2, $3, $4, $5);",
    )
    .bind(character_id)
    .bind(world_id)
    .bind(team_id)
    .bind(zone_id)
    .bind(class_name)
    .execute(pool)
    .await
    .unwrap();
}

async fn track_analytics(analytics_event: AnalyticsEvent) {
    // println!("[ws/track_analytics]");
    let pool = PG.get().await;

    let AnalyticsEvent {
        world_id,
        event_name,
    } = analytics_event;

    query("INSERT INTO analytics (time, world_id, event_name) VALUES (now(), $1, $2);")
        .bind(world_id)
        .bind(event_name)
        .execute(pool)
        .await
        .unwrap();
}

async fn process_event(event: &Event) {
    let mut set = JoinSet::new();
    // println!("[ws/process_event] EVENT: {:?}", event);

    set.spawn(track_analytics(AnalyticsEvent {
        world_id: event.world_id.clone(),
        event_name: event.event_name.clone(),
    }));

    if event.character_id != "0" {
        // General population tracking
        set.spawn(track_pop(PopEvent {
            world_id: event.world_id.clone(),
            team_id: event.team_id.clone(),
            character_id: event.character_id.clone(),
            zone_id: event.zone_id.clone(),
        }));
    }

    if event.event_name == "VehicleDestroy" {
        set.spawn(track_vehicle(VehicleEvent {
            world_id: event.world_id.clone(),
            vehicle_id: event.vehicle_id.clone(),
            character_id: event.character_id.clone(),
            zone_id: event.zone_id.clone(),
            team_id: event.team_id.clone(),
        }));
    }

    if event.event_name == "Death" {
        set.spawn(track_class(ClassEvent {
            world_id: event.world_id.clone(),
            character_id: event.character_id.clone(),
            loadout_id: event.loadout_id.clone(),
            zone_id: event.zone_id.clone(),
            team_id: event.team_id.clone(),
        }));
    }

    if event.attacker_character_id != ""
        && event.attacker_character_id != "0"
        && (event.attacker_team_id != 0 || event.attacker_team_id != 0)
    {
        set.spawn(track_pop(PopEvent {
            world_id: event.world_id.clone(),
            team_id: event.attacker_team_id.clone(),
            character_id: event.attacker_character_id.clone(),
            zone_id: event.zone_id.clone(),
        }));

        if event.event_name == "VehicleDestroy" {
            set.spawn(track_vehicle(VehicleEvent {
                world_id: event.world_id.clone(),
                vehicle_id: event.attacker_vehicle_id.clone(),
                character_id: event.attacker_character_id.clone(),
                zone_id: event.zone_id.clone(),
                team_id: event.attacker_team_id.clone(),
            }));
        }

        if event.event_name == "Death" {
            set.spawn(track_class(ClassEvent {
                world_id: event.world_id.clone(),
                character_id: event.attacker_character_id.clone(),
                loadout_id: event.attacker_loadout_id.clone(),
                zone_id: event.zone_id.clone(),
                team_id: event.attacker_team_id.clone(),
            }));
        }
    }

    while let Some(_) = set.join_next().await {}
}

#[derive(Deserialize, Debug, Clone, Default)]
struct Event {
    event_name: String,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    world_id: i32,
    character_id: String,
    attacker_character_id: String,
    #[serde(default, deserialize_with = "deserialize_number_from_string")]
    attacker_team_id: i32,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    team_id: i32,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    zone_id: i32,

    // Class Tracking
    #[serde(default)]
    attacker_loadout_id: String,
    #[serde(default)]
    loadout_id: String,

    // Vehicle Tracking
    #[serde(default)]
    vehicle_id: String,
    #[serde(default)]
    attacker_vehicle_id: String,
}

#[derive(Deserialize, Debug, Clone)]
struct Payload {
    payload: Event,
}

async fn healthz() {
    let app = Router::new().route(
        "/healthz",
        get(|| async {
            Json(json!({
                "status": "ok",
            }))
        }),
    );

    let port: u16 = std::env::var("PORT")
        .unwrap_or("8999".to_string())
        .parse()
        .unwrap();
    let addr = SocketAddr::from(([0, 0, 0, 0], port));

    println!("[healthz] Listening on http://{}", addr);

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

#[tokio::main]
async fn main() {
    let addr: String = WS_ADDR.to_string();
    if addr == "" {
        println!("WS_ADDR not set");
        return;
    }
    let url = url::Url::parse(&addr).unwrap();

    let (tx, rx) = futures::channel::mpsc::unbounded();
    let (ws_stream, _) = connect_async(url).await.expect("Failed to connect");
    let (write, read) = ws_stream.split();

    let fused_writer = rx.map(Ok).forward(write).fuse();
    let fused_reader = read
        .for_each(|msg| async {
            let body = &msg.unwrap().to_string();

            let data: Payload = match serde_json::from_str(body) {
                Ok(data) => data,
                Err(_) => {
                    // println!("Error: {}; body: {}", e, body.clone());
                    return;
                }
            };

            if data.payload.event_name == "" {
                return;
            }

            process_event(&data.payload).await;
        })
        .fuse();

    pin_mut!(fused_writer, fused_reader);

    let init = tokio::spawn(send_init(tx.clone()));
    let mut healthz = tokio::spawn(healthz()).fuse();
    futures::select! {
        _ = fused_reader => {}
        _ = fused_writer => {}
        _ = healthz => {}
    }
    init.await.unwrap();
}
