use futures::{pin_mut, FutureExt};
use futures_util::StreamExt;
use once_cell::sync::Lazy;
use redis::Commands;
use serde::Deserialize;
use serde_json::json;
use std::{env, time::SystemTime};
use tokio_tungstenite::{connect_async, tungstenite::Message};

pub static REDIS_CLIENT: Lazy<redis::Client> = Lazy::new(|| {
    redis::Client::open(std::env::var("REDIS_ADDR").unwrap_or("redis://localhost:6379".to_string()))
        .unwrap()
});

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

fn process_event(event: &Event) {
    let mut con = REDIS_CLIENT.get_connection().unwrap();

    let timestamp = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let key: String = format!("wp:{}/{}", event.world_id, event.team_id);
    con.zadd::<String, u64, String, ()>(key, event.character_id.clone(), timestamp)
        .unwrap();

    if event.attacker_character_id != "" {
        let key = format!("wp:{}/{}", event.world_id, event.attacker_team_id);
        con.zadd::<String, u64, String, ()>(key, event.attacker_character_id.clone(), timestamp)
            .unwrap();
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
}

#[derive(Deserialize, Debug, Clone)]
struct Payload {
    payload: Event,
}

#[tokio::main]
async fn main() {
    let addr = env::var("WS_ADDR").unwrap_or_default();
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
            let body = &msg.unwrap().to_string();
            let data: Payload = serde_json::from_str(body).unwrap_or(Payload {
                payload: Event {
                    event_name: "".to_string(),
                    world_id: "".to_string(),
                    character_id: "".to_string(),
                    attacker_character_id: "".to_string(),
                    attacker_team_id: "".to_string(),
                    team_id: "".to_string(),
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
