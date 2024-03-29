use async_graphql::{futures_util::TryStreamExt, Context, Object, SimpleObject};
use chrono::{DateTime, Utc};
use sqlx::{query, Pool, Postgres, Row};
use crate::telemetry;

pub struct Analytics {}

#[derive(SimpleObject, Debug, Clone)]
pub struct Event {
    pub time: DateTime<Utc>,
    pub event_name: String,
    pub world_id: i32,
    pub count: i64,
}

#[Object]
impl Analytics {
    /// Get all events in analytics, bucket_size is in seconds
    async fn events<'ctx>(
        &self,
        ctx: &Context<'ctx>,
        #[graphql(default = 60)] bucket_size: u64,
        world_id: Option<i32>,
        #[graphql(default = false)] hi_precision: bool,
    ) -> Vec<Event> {
        telemetry::graphql_query("Analytics", "events");
        let pool = ctx.data::<Pool<Postgres>>().unwrap();

        telemetry::db_read("analytics", "events");
        let sql = format!("
            SELECT 
                time_bucket_gapfill('{} seconds', time, start => now() - '{}'::interval, finish => now()) AS bucket, 
                CASE WHEN count(*) IS NULL THEN 0 ELSE count(*) END AS count, 
                event_name, 
                world_id 
            FROM analytics 
            WHERE time > now() - '{}'::interval {} 
            GROUP BY bucket, world_id, event_name 
            ORDER BY bucket ASC",
            if hi_precision {
                5
            } else {
                bucket_size
            },
            if hi_precision {
                "1 hour"
            } else {
                "1 day"
            },
            if hi_precision {
                "1 hour"
            } else {
                "1 day"
            },
            if let Some(world_id) = world_id {
                format!("AND world_id = {}", world_id)
            } else {
                "".to_string()
            }
        );

        let mut result = query(sql.as_str()).fetch(pool);

        let mut events = Vec::new();
        while let Some(row) = result.try_next().await.unwrap() {
            events.push(Event {
                time: row.get("bucket"),
                event_name: row.get("event_name"),
                world_id: row.get("world_id"),
                count: row.get("count"),
            });
        }

        events
    }
}

#[derive(Default)]
pub struct AnalyticsQuery;

#[Object]
impl AnalyticsQuery {
    async fn analytics(&self) -> Analytics {
        Analytics {}
    }
}
