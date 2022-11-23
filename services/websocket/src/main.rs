use futures::{pin_mut, FutureExt};
use futures_util::StreamExt;
use once_cell::sync::Lazy;
use redis::Commands;
use serde::Deserialize;
use serde_json::json;
use std::{env, time::SystemTime};
use tokio_tungstenite::{connect_async, tungstenite::Message};

mod translators;

pub static REDIS_CLIENT: Lazy<redis::Client> = Lazy::new(|| {
    redis::Client::open(std::env::var("REDIS_ADDR").unwrap_or("redis://localhost:6379".to_string()))
        .unwrap()
});

static PAIR: Lazy<String> = Lazy::new(|| env::var("PAIR").unwrap_or_default());
static ROLE: Lazy<String> = Lazy::new(|| env::var("ROLE").unwrap_or("primary".to_string()));
static WS_ADDR: Lazy<String> = Lazy::new(|| env::var("WS_ADDR").unwrap_or_default());

async fn send_init(tx: futures::channel::mpsc::UnboundedSender<Message>) {
    let worlds_raw = env::var("WORLDS").unwrap_or_default();
    if worlds_raw == "" {
        println!("WORLDS not set");
        return;
    }
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

    println!("Sent setup message");
}

struct PopEvent {
    world_id: String,
    team_id: String,
    character_id: String,
    timestamp: u64,
}

struct VehicleEvent {
    world_id: String,
    vehicle_id: String,
    character_id: String,
    timestamp: u64,
}

struct ClassEvent {
    world_id: String,
    character_id: String,
    loadout_id: String,
    timestamp: u64,
}

async fn track_pop(pop_event: PopEvent) {
    let mut con = REDIS_CLIENT.get_connection().unwrap();

    let PopEvent {
        world_id,
        team_id,
        character_id,
        timestamp,
    } = pop_event;

    let key = format!("wp:{}/{}", world_id, team_id);
    let _: () = con.zadd(key, character_id, timestamp).unwrap();
}

async fn track_vehicle(vehicle_event: VehicleEvent) {
    let mut con = REDIS_CLIENT.get_connection().unwrap();

    let VehicleEvent {
        world_id,
        vehicle_id,
        timestamp,
        character_id,
    } = vehicle_event;

    let vehicle_name = translators::vehicle_to_name(vehicle_id.as_str());

    if vehicle_name == "unknown" {
        return;
    }

    let key = format!("v:{}/{}", world_id, vehicle_name);
    let _: () = con.zadd(key, character_id, timestamp).unwrap();
}

async fn track_class(class_event: ClassEvent) {
    let mut con = REDIS_CLIENT.get_connection().unwrap();

    let ClassEvent {
        world_id,
        character_id,
        loadout_id,
        timestamp,
    } = class_event;

    let class_name = translators::loadout_to_class(loadout_id.as_str());

    if class_name == "unknown" {
        return;
    }

    let key = format!("c:{}/{}", world_id, class_name);
    let _: () = con.zadd(key, character_id, timestamp).unwrap();
}

fn should_process_event() -> bool {
    let mut con = REDIS_CLIENT.get_connection().unwrap();
    let role: String = ROLE.parse().unwrap();
    let heartbeat_key = format!("heartbeat:{}", PAIR.to_string());

    if role == "primary" {
        let _: () = con.set_ex(heartbeat_key, "1", 60).unwrap();
        return false;
    }

    match con.get(heartbeat_key) {
        Ok(1) => true,
        _ => false,
    }
}

fn process_event(event: &Event) {
    if should_process_event() {
        return;
    }

    let timestamp = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    // General population tracking
    track_pop(PopEvent {
        world_id: event.world_id.clone(),
        team_id: event.team_id.clone(),
        character_id: event.character_id.clone(),
        timestamp,
    })
    .now_or_never();

    if event.event_name == "VehicleDestroy" {
        track_vehicle(VehicleEvent {
            world_id: event.world_id.clone(),
            vehicle_id: event.vehicle_id.clone(),
            character_id: event.character_id.clone(),
            timestamp,
        })
        .now_or_never();
    }

    if event.event_name == "Death" {
        track_class(ClassEvent {
            world_id: event.world_id.clone(),
            character_id: event.character_id.clone(),
            loadout_id: event.loadout_id.clone(),
            timestamp,
        })
        .now_or_never();
    }

    if event.attacker_character_id != ""
        && (event.attacker_team_id != "" || event.attacker_team_id != "0")
    {
        track_pop(PopEvent {
            world_id: event.world_id.clone(),
            team_id: event.attacker_team_id.clone(),
            character_id: event.attacker_character_id.clone(),
            timestamp,
        })
        .now_or_never();

        if event.event_name == "VehicleDestroy" {
            track_vehicle(VehicleEvent {
                world_id: event.world_id.clone(),
                vehicle_id: event.attacker_vehicle_id.clone(),
                character_id: event.attacker_character_id.clone(),
                timestamp,
            })
            .now_or_never();
        }

        if event.event_name == "Death" {
            track_class(ClassEvent {
                world_id: event.world_id.clone(),
                character_id: event.attacker_character_id.clone(),
                loadout_id: event.attacker_loadout_id.clone(),
                timestamp,
            })
            .now_or_never();
        }
    }
}

#[derive(Deserialize, Debug, Clone)]
struct Event {
    event_name: String,
    world_id: String,
    character_id: String,
    attacker_character_id: String,
    attacker_team_id: String,
    team_id: String,

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
        .for_each(|msg| async move {
            // println!("Processing event: {:?}", msg);

            let body = &msg.unwrap().to_string();
            let data: Payload = serde_json::from_str(body).unwrap_or(Payload {
                payload: Event {
                    event_name: "".to_string(),
                    world_id: "".to_string(),
                    character_id: "".to_string(),
                    attacker_character_id: "".to_string(),
                    attacker_team_id: "".to_string(),
                    team_id: "".to_string(),
                    attacker_loadout_id: "".to_string(),
                    loadout_id: "".to_string(),
                    vehicle_id: "".to_string(),
                    attacker_vehicle_id: "".to_string(),
                },
            });

            if data.payload.event_name == "" {
                return;
            }

            process_event(&data.payload);
        })
        .fuse();

    pin_mut!(fused_writer, fused_reader);

    let init = tokio::spawn(send_init(tx.clone()));

    futures::select! {
        _ = fused_reader => {}
        _ = fused_writer => {}
    }

    init.await.unwrap();
}
