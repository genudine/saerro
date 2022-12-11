use crate::utils::ID_TO_WORLD;
use async_graphql::{Context, Enum, Object, SimpleObject};
use axum::{http::StatusCode, response::IntoResponse, Extension, Json};
use chrono::{DateTime, Utc};
use sqlx::{query, Pool, Postgres, Row};

pub async fn get_health(Extension(pool): Extension<Pool<Postgres>>) -> impl IntoResponse {
    let events_resp =
        query("SELECT count(*) FROM analytics WHERE time > now() - interval '5 minutes'")
            .fetch_one(&pool)
            .await;

    match events_resp {
        Ok(row) => {
            let events_row: i64 = row.get(0);

            if events_row == 0 {
                return (
                    StatusCode::SERVICE_UNAVAILABLE,
                    Json(json!({
                        "websocket": "down",
                        "database": "up"
                    })),
                );
            } else {
                return (
                    StatusCode::OK,
                    Json(json!({
                        "websocket": "up",
                        "database": "up"
                    })),
                );
            }
        }
        Err(_) => {
            return (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(json!({
                    "websocket": "down",
                    "database": "down"
                })),
            );
        }
    }
}

#[derive(Enum, Copy, Clone, Eq, PartialEq)]
enum UpDown {
    /// The service is up and running
    Up,

    /// The service is down
    Down,
}

pub struct Health {}

impl Health {
    async fn most_recent_event_time<'ctx>(
        &self,
        ctx: &Context<'ctx>,
        world_id: i32,
    ) -> (UpDown, Option<DateTime<Utc>>) {
        let pool = ctx.data::<Pool<Postgres>>().unwrap();

        let events_resp =
            query("SELECT time FROM analytics WHERE world_id = $1 ORDER BY time DESC LIMIT 1")
                .bind(world_id)
                .fetch_one(pool)
                .await;

        match events_resp {
            Ok(row) => {
                let last_event: DateTime<Utc> = row.get(0);

                if last_event < Utc::now() - chrono::Duration::minutes(5) {
                    return (UpDown::Down, None);
                } else {
                    return (UpDown::Up, Some(last_event));
                }
            }
            Err(_) => {
                return (UpDown::Down, None);
            }
        }
    }
}

/// Reports on the health of Saerro Listening Post
#[Object]
impl Health {
    /// Did a ping to Postgres (our main datastore) succeed?
    async fn database<'ctx>(&self, ctx: &Context<'ctx>) -> UpDown {
        let pool = ctx.data::<Pool<Postgres>>().unwrap();

        let events_resp =
            query("SELECT count(*) FROM analytics WHERE time > now() - interval '5 minutes'")
                .fetch_one(pool)
                .await;

        match events_resp {
            Ok(_) => UpDown::Up,
            Err(_) => UpDown::Down,
        }
    }

    /// Is the websocket processing jobs?
    async fn ingest<'ctx>(&self, ctx: &Context<'ctx>) -> UpDown {
        let pool = ctx.data::<Pool<Postgres>>().unwrap();

        let events_resp =
            query("SELECT count(*) FROM analytics WHERE time > now() - interval '5 minutes'")
                .fetch_one(pool)
                .await;

        match events_resp {
            Ok(row) => {
                let events_row: i64 = row.get(0);

                if events_row == 0 {
                    return UpDown::Down;
                } else {
                    return UpDown::Up;
                }
            }
            Err(_) => UpDown::Down,
        }
    }

    /// Is the websocket actually turned on?
    async fn ingest_reachable(&self) -> UpDown {
        reqwest::get(
            std::env::var("WEBSOCKET_HEALTHCHECK")
                .unwrap_or("http://localhost:8999/health".to_string()),
        )
        .await
        .map(|_| UpDown::Up)
        .unwrap_or(UpDown::Down)
    }

    /// Shows a disclaimer for the worlds check
    async fn worlds_disclaimer(&self) -> String {
        "This is a best-effort check. A world reports `DOWN` when it doesn't have new events for 5 minutes. It could be broken, it could be the reality of the game state.".to_string()
    }

    /// Checks if a world has had any events for the last 5 minutes
    async fn worlds<'ctx>(&self, ctx: &Context<'ctx>) -> Vec<WorldUpDown> {
        let mut worlds = Vec::new();
        for (id, name) in ID_TO_WORLD.iter() {
            let (status, last_event) = self.most_recent_event_time(ctx, *id).await;
            worlds.push(WorldUpDown {
                id: *id,
                name: name.to_string(),
                status,
                last_event,
            });
        }
        worlds
    }
}

#[derive(SimpleObject)]
struct WorldUpDown {
    id: i32,
    name: String,
    status: UpDown,
    last_event: Option<DateTime<Utc>>,
}

#[derive(Default)]
pub struct HealthQuery;

#[Object]
impl HealthQuery {
    /// Reports on the health of Saerro Listening Post
    pub async fn health(&self) -> Health {
        Health {}
    }
}
