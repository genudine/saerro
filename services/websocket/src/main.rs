use async_once::AsyncOnce;
use axum::{routing::get, Json, Router};
use futures::{pin_mut, FutureExt};
use futures_util::StreamExt;
use lazy_static::lazy_static;
use serde::Deserialize;
use serde_aux::prelude::*;
use serde_json::json;
use sqlx::{postgres::PgPoolOptions, query, Row};
use std::{env, net::SocketAddr};
use tokio::task::JoinSet;
use tokio_tungstenite::{connect_async, tungstenite::Message};

mod translators;
mod telemetry;

lazy_static! {
    static ref WS_ADDR: String = env::var("WS_ADDR").unwrap_or_default();
    static ref PG: AsyncOnce<sqlx::PgPool> = AsyncOnce::new(async {
        let db_url = std::env::var("DATABASE_URL")
            .unwrap_or("postgres://saerrouser:saerro321@127.0.0.1:5432/data".to_string());
        PgPoolOptions::new().connect(&db_url).await.unwrap()
    });
}

async fn send_init(tx: futures::channel::mpsc::UnboundedSender<Message>) {
    let worlds_raw = env::var("WORLDS").unwrap_or("all".to_string());
    let worlds: Vec<&str> = worlds_raw.split(',').collect();

    let experience_ids = vec![
        2, 3, 4, 5, 6, 7, 34, 51, 53, 55, 57, 86, 88, 89, 90, 91, 92, 93, 94, 95, 96, 97, 98, 99,
        100, 129, 130, 131, 132, 133, 134, 135, 136, 137, 138, 139, 140, 141, 142, 201, 233, 293,
        294, 302, 303, 353, 354, 355, 438, 439, 503, 505, 579, 581, 584, 653, 656, 674, 675,
    ];
    let mut events = experience_ids
        .iter()
        .map(|id| format!("GainExperience_experience_id_{}", id))
        .collect::<Vec<String>>();

    events.push("Death".to_string());
    events.push("VehicleDestroy".to_string());

    // Send setup message
    let setup_msg = json!({
        "action": "subscribe",
        "worlds": worlds,
        "eventNames": events,
        "characters": ["all"],
        "logicalAndCharactersWithWorlds": true,
        "service": "event",
    });

    tx.unbounded_send(Message::text(setup_msg.to_string()))
        .unwrap();

    println!("[ws] Sent setup message");
    println!("[ws/setup] {}", setup_msg.to_string())
}

#[derive(Clone)]
struct PopEvent {
    world_id: i32,
    team_id: i32,
    character_id: String,
    zone_id: i32,
    loadout_id: String,
    vehicle_id: String,
}

#[derive(Debug)]
struct AnalyticsEvent {
    world_id: i32,
    event_name: String,
}

async fn get_team_id(character_id: String) -> Result<i32, sqlx::Error> {
    let pool = PG.get().await;

    telemetry::db_read("players", "get_team_id");
    let team_id: i32 = query("SELECT faction_id FROM players WHERE character_id = $1 LIMIT 1;")
        .bind(character_id.clone())
        .fetch_one(pool)
        .await?
        .get(0);

    if team_id == 0 {
        return Err(sqlx::Error::RowNotFound);
    }

    Ok(team_id)
}

async fn track_pop(pop_event: PopEvent) {
    // println!("[ws/track_pop]");
    let pool = PG.get().await;

    let PopEvent {
        world_id,
        team_id,
        character_id,
        zone_id,
        loadout_id,
        vehicle_id,
    } = pop_event;

    let class_name = translators::loadout_to_class(loadout_id.as_str());
    let vehicle_name = if vehicle_id == "" {
        "unknown".to_string()
    } else {
        translators::vehicle_to_name(vehicle_id.as_str())
    };

    telemetry::db_write("players", "track_pop");
    query(
        "
        INSERT INTO players (last_updated, character_id, world_id, faction_id, zone_id, class_name) 
        VALUES (now(), $1, $2, $3, $4, $5) 
        ON CONFLICT (character_id) DO UPDATE SET 
            last_updated = EXCLUDED.last_updated,
            world_id = EXCLUDED.world_id,
            faction_id = EXCLUDED.faction_id,
            zone_id = EXCLUDED.zone_id,
            class_name = EXCLUDED.class_name
    ;",
    )
    .bind(character_id.clone())
    .bind(world_id)
    .bind(team_id)
    .bind(zone_id)
    .bind(class_name)
    .execute(pool)
    .await
    .unwrap();

    if vehicle_name != "unknown" {
        telemetry::db_write("vehicles", "track_pop");
        query("INSERT INTO vehicles (last_updated, character_id, world_id, faction_id, zone_id, vehicle_name) 
        VALUES (now(), $1, $2, $3, $4, $5) 
        ON CONFLICT (character_id) DO UPDATE SET
            last_updated = EXCLUDED.last_updated,
            world_id = EXCLUDED.world_id,
            faction_id = EXCLUDED.faction_id,
            zone_id = EXCLUDED.zone_id,
            vehicle_name = EXCLUDED.vehicle_name
    ;")
        .bind(character_id)
        .bind(world_id)
        .bind(team_id)
        .bind(zone_id)
        .bind(vehicle_name)
        .execute(pool)
        .await
        .unwrap();
    }
}

async fn track_analytics(analytics_event: AnalyticsEvent) {
    // println!("[ws/track_analytics] {:?}", analytics_event);
    let pool = PG.get().await;

    let AnalyticsEvent {
        world_id,
        event_name,
    } = analytics_event;

    telemetry::db_write("analytics", "track_analytics");
    match query("INSERT INTO analytics (time, world_id, event_name) VALUES (now(), $1, $2);")
        .bind(world_id)
        .bind(event_name)
        .execute(pool)
        .await
    {
        Ok(_) => {}
        Err(e) => {
            println!("[ws/track_analytics] ERR => {:?}", e);
        }
    }
}

async fn process_death_event(event: &Event) {
    let mut set = JoinSet::new();
    // println!("[ws/process_event] EVENT: {:?}", event);

    set.spawn(track_analytics(AnalyticsEvent {
        world_id: event.world_id.clone(),
        event_name: event.event_name.clone(),
    }));

    if event.character_id != "" && event.character_id != "0" {
        set.spawn(track_pop(PopEvent {
            world_id: event.world_id.clone(),
            team_id: event.team_id.clone(),
            character_id: event.character_id.clone(),
            zone_id: event.zone_id.clone(),
            loadout_id: event.loadout_id.clone(),
            vehicle_id: event.vehicle_id.clone(),
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
            loadout_id: event.attacker_loadout_id.clone(),
            vehicle_id: event.attacker_vehicle_id.clone(),
        }));
    }

    while let Some(_) = set.join_next().await {}
}

async fn process_exp_event(event: &Event) {
    telemetry::experience_event(&event.world_id, &event.experience_id); 
    let mut set = JoinSet::new();
    // println!("[ws/process_event] EVENT: {:?}", event);

    set.spawn(track_analytics(AnalyticsEvent {
        world_id: event.world_id.clone(),
        event_name: format!(
            "{}_{}",
            event.event_name.clone(),
            event.experience_id.clone()
        ),
    }));

    // Vehicle EXP events
    let vehicle_id = match event.experience_id {
        201 => "11".to_string(),        // Galaxy Spawn Bonus
        233 => "2".to_string(),         // Sunderer Spawn Bonus
        674 | 675 => "160".to_string(), // ANT stuff
        _ => "".to_string(),
    };

    set.spawn(track_pop(PopEvent {
        world_id: event.world_id.clone(),
        team_id: event.team_id.clone(),
        character_id: event.character_id.clone(),
        zone_id: event.zone_id.clone(),
        loadout_id: event.loadout_id.clone(),
        vehicle_id: vehicle_id.clone(),
    }));

    while let Some(_) = set.join_next().await {}
}
#[derive(Deserialize, Debug, Clone, Default)]
struct Event {
    event_name: String,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    world_id: i32,
    character_id: String,
    #[serde(default)]
    attacker_character_id: String,
    #[serde(default, deserialize_with = "deserialize_number_from_string")]
    attacker_team_id: i32,
    #[serde(default, deserialize_with = "deserialize_number_from_string")]
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

    #[serde(default, deserialize_with = "deserialize_number_from_string")]
    experience_id: i32,
    // #[serde(default)]
    // other_id: String,
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
    ).route(
        "/metrics",
        get(telemetry::handler)
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

    println!("[ws] Connecting to {}", url);

    let (tx, rx) = futures::channel::mpsc::unbounded();
    let (ws_stream, _) = connect_async(url).await.expect("Failed to connect");
    let (write, read) = ws_stream.split();

    let fused_writer = rx.map(Ok).forward(write).fuse();
    let fused_reader = read
        .for_each(|msg| async {
            let body = &msg.unwrap().to_string();

            let mut data: Payload = match serde_json::from_str(body) {
                Ok(data) => data,
                Err(_e) => {
                    // println!("Error: {}; body: {}", e, body.clone());
                    telemetry::event_dropped(&0, &"".to_string(), "decoding failure");
                    return;
                }
            };

            if data.payload.event_name == "" {
                telemetry::event_dropped(&data.payload.world_id, &data.payload.event_name, "not event");
                return;
            }

            telemetry::event(&data.payload.world_id, &data.payload.event_name);

            if data.payload.event_name == "Death" || data.payload.event_name == "VehicleDestroy" {
                process_death_event(&data.payload).await;
                return;
            }

            if data.payload.event_name == "GainExperience" {
                if data.payload.team_id == 0 {
                    match get_team_id(data.payload.character_id.clone()).await {
                        Ok(team_id) => {
                            data.payload.team_id = team_id;
                        }
                        Err(_) => {
                            telemetry::event_dropped(&data.payload.world_id, &data.payload.event_name, "team_id missing");
                        }
                    }
                }
                process_exp_event(&data.payload).await;
                return;
            }

            telemetry::event_dropped(&data.payload.world_id, &data.payload.event_name, "unprocessable");
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
