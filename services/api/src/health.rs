use async_graphql::{Context, Enum, Object};
use axum::{http::StatusCode, response::IntoResponse, Extension, Json};
use redis::{aio::MultiplexedConnection, pipe};

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
    /// The service is up and running
    Up,

    /// The service is down
    Down,
}

#[derive(Enum, Copy, Clone, Eq, PartialEq)]
enum WebsocketState {
    /// The Nanite Systems manifold is sending events, and the primary listener is processing data.
    Primary,

    /// The Daybreak Games manifold is sending events, and the backup listener is processing data; the primary listener is down.
    Backup,

    /// The entire event streaming system is down.
    Down,
}

pub struct Health {}

impl Health {
    async fn get_health<'ctx>(&self, ctx: &Context<'ctx>, pair: &str) -> WebsocketState {
        let mut con = ctx.data::<MultiplexedConnection>().unwrap().to_owned();
        let (primary, backup): (Option<String>, Option<String>) = pipe()
            .get(format!("heartbeat:{}:primary", pair))
            .get(format!("heartbeat:{}:backup", pair))
            .query_async(&mut con)
            .await
            .unwrap();

        match (primary, backup) {
            (Some(_), _) => WebsocketState::Primary,
            (None, Some(_)) => WebsocketState::Backup,
            _ => WebsocketState::Down,
        }
    }
}

/// Reports on the health of Saerro Listening Post
#[Object]
impl Health {
    /// Did a ping to Redis (our main datastore) succeed?
    async fn redis<'ctx>(&self, ctx: &Context<'ctx>) -> UpDown {
        let mut con = ctx.data::<MultiplexedConnection>().unwrap().to_owned();
        let ping: String = redis::cmd("PING")
            .query_async(&mut con)
            .await
            .unwrap_or_default();
        if ping == "PONG" {
            UpDown::Up
        } else {
            UpDown::Down
        }
    }

    /// What is the state of the websocket listener cluster for PC?
    #[graphql(name = "pc")]
    async fn pc<'ctx>(&self, ctx: &Context<'ctx>) -> WebsocketState {
        self.get_health(ctx, "pc").await
    }

    /// What is the state of the websocket listener cluster for PS4 US?
    #[graphql(name = "ps4us")]
    async fn ps4us<'ctx>(&self, ctx: &Context<'ctx>) -> WebsocketState {
        self.get_health(ctx, "ps4us").await
    }

    /// What is the state of the websocket listener cluster for PS4 EU?
    #[graphql(name = "ps4eu")]
    async fn ps4eu<'ctx>(&self, ctx: &Context<'ctx>) -> WebsocketState {
        self.get_health(ctx, "ps4eu").await
    }
}
