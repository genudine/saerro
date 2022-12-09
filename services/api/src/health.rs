use async_graphql::{Context, Enum, Object};
use axum::{http::StatusCode, response::IntoResponse, Extension, Json};
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

    /// Is the websocket connection to the Nanite Systems manifold up?
    async fn websocket<'ctx>(&self, ctx: &Context<'ctx>) -> UpDown {
        let pool = ctx.data::<Pool<Postgres>>().unwrap();

        match query("SELECT count(*) FROM analytics WHERE time > now() - interval '5 minutes'")
            .fetch_one(pool)
            .await
        {
            Ok(i) => {
                let num: i64 = i.get(0);
                if num == 0 {
                    UpDown::Down
                } else {
                    UpDown::Up
                }
            }
            Err(_) => UpDown::Down,
        }
    }
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
