use crate::{
    factions::{NC, NSO, TR, VS},
    utils::Filters,
    telemetry,
};
use async_graphql::{Context, Object};
use sqlx::{Pool, Postgres, Row};

/// A filterable list of currently active players.
pub struct Population {
    filters: Filters,
}

impl Population {
    pub fn new(filters: Option<Filters>) -> Self {
        Self {
            filters: filters.unwrap_or_default(),
        }
    }
}

impl Population {
    async fn by_faction<'ctx>(&self, ctx: &Context<'ctx>, faction: i32) -> i64 {
        let pool = ctx.data::<Pool<Postgres>>().unwrap();

        telemetry::db_read("players", "population_by_faction");
        let sql = format!(
            "SELECT count(*) FROM players WHERE last_updated > now() - interval '15 minutes' AND faction_id = $1 {};",
            self.filters.sql(),
        );

        println!("{}", sql);

        let query: i64 = sqlx::query(sql.as_str())
            .bind(faction)
            .fetch_one(pool)
            .await
            .unwrap()
            .get(0);

        query
    }
}

#[Object]
impl Population {
    async fn total<'ctx>(&self, ctx: &Context<'ctx>) -> i64 {
        telemetry::graphql_query("Population", "total");

        let pool = ctx.data::<Pool<Postgres>>().unwrap();

        telemetry::db_read("players", "population_total");
        let sql = format!(
            "SELECT count(*) FROM players WHERE last_updated > now() - interval '15 minutes' {};",
            self.filters.sql(),
        );

        println!("{}", sql);

        let query: i64 = sqlx::query(sql.as_str())
            .fetch_one(pool)
            .await
            .unwrap()
            .get(0);
        
        query
    }
    async fn nc<'ctx>(&self, ctx: &Context<'ctx>) -> i64 {
        telemetry::graphql_query("Population", "nc");
        self.by_faction(ctx, NC).await
    }
    async fn vs<'ctx>(&self, ctx: &Context<'ctx>) -> i64 {
        telemetry::graphql_query("Population", "vs");
        self.by_faction(ctx, VS).await
    }
    async fn tr<'ctx>(&self, ctx: &Context<'ctx>) -> i64 {
        telemetry::graphql_query("Population", "tr");
        self.by_faction(ctx, TR).await
    }
    async fn ns<'ctx>(&self, ctx: &Context<'ctx>) -> i64 {
        telemetry::graphql_query("Population", "ns");
        self.by_faction(ctx, NSO).await
    }
}

#[derive(Default)]
pub struct PopulationQuery;

#[Object]
impl PopulationQuery {
    /// A filterable list of currently active players.
    /// This is a core query that others will use to filter by,
    /// i.e. `emerald { population { total } }` is equivalent to `population(filter: { world: { name: "emerald" } }) { total }`
    pub async fn population(&self, filter: Option<Filters>) -> Population {
        Population::new(filter)
    }
}
