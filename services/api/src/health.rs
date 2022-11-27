use async_graphql::{Enum, Object};
use axum::{http::StatusCode, response::IntoResponse, Extension, Json};
use redis::pipe;

pub async fn get_health(
    Extension(mut redis): Extension<redis::aio::MultiplexedConnection>,
) -> impl IntoResponse {
    let (ping, pc, ps4us, ps4eu): (String, bool, bool, bool) = pipe()
        .cmd("PING")
        .get("heartbeat:pc")
        .get("heartbeat:ps4us")
        .get("heartbeat:ps4eu")
        .query_async(&mut redis)
        .await
        .unwrap_or_default();

    if ping != "PONG" {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(json!({
                "status": "error",
                "message": "Redis is not responding",
            })),
        );
    }

    (
        StatusCode::OK,
        Json(json!({
            "status": if ping == "PONG" && pc && ps4us && ps4eu { "ok" } else { "degraded" },
            "redis": ping == "PONG",
            "pc": if pc { "primary" } else { "backup/down" },
            "ps4us": if ps4us { "primary" } else { "backup/down" },
            "ps4eu": if ps4eu { "primary" } else { "backup/down" },
        })),
    )
}

#[derive(Enum, Copy, Clone, Eq, PartialEq)]
enum UpDown {
    Up,
    Down,
}

#[derive(Enum, Copy, Clone, Eq, PartialEq)]
enum WebsocketState {
    Primary,
    Backup,
    Down,
}

pub struct Health {}

#[Object]
impl Health {
    async fn redis(&self) -> UpDown {
        UpDown::Up
    }

    #[graphql(name = "pc")]
    async fn pc(&self) -> WebsocketState {
        WebsocketState::Primary
    }

    #[graphql(name = "ps4us")]
    async fn ps4us(&self) -> WebsocketState {
        WebsocketState::Primary
    }

    #[graphql(name = "ps4eu")]
    async fn ps4eu(&self) -> WebsocketState {
        WebsocketState::Primary
    }
}
