use crate::{
    factions::{NC, TR, VS},
    utils::{Filters, IdOrNameBy},
    telemetry,
};
use async_graphql::{Context, Object};
use sqlx::{Pool, Postgres, Row};

/// A specific vehicle
pub struct Vehicle {
    filters: Filters,
    vehicle_name: String,
}

impl Vehicle {
    async fn fetch<'ctx>(&self, ctx: &Context<'ctx>, filters: Filters) -> i64 {
        let pool = ctx.data::<Pool<Postgres>>().unwrap();

        telemetry::db_read("vehicles", "fetch");
        let sql = format!(
            "SELECT count(*) FROM vehicles WHERE last_updated > now() - interval '15 minutes' AND vehicle_name = $1 {};",
            filters.sql(),
        );

        println!("{}", sql);

        let query: i64 = sqlx::query(sql.as_str())
            .bind(self.vehicle_name.as_str())
            .fetch_one(pool)
            .await
            .unwrap()
            .get(0);

        query
    }
}

#[Object]
impl Vehicle {
    async fn total<'ctx>(&self, ctx: &Context<'ctx>) -> i64 {
        telemetry::graphql_query("Vehicle", "total");

        self.fetch(ctx, self.filters.clone()).await
    }
    async fn nc<'ctx>(&self, ctx: &Context<'ctx>) -> i64 {
        telemetry::graphql_query("Vehicle", "nc");

        self.fetch(
            ctx,
            Filters {
                faction: Some(IdOrNameBy::Id(NC)),
                ..self.filters.clone()
            },
        )
        .await
    }
    async fn tr<'ctx>(&self, ctx: &Context<'ctx>) -> i64 {
        telemetry::graphql_query("Vehicle", "tr");

        self.fetch(
            ctx,
            Filters {
                faction: Some(IdOrNameBy::Id(TR)),
                ..self.filters.clone()
            },
        )
        .await
    }
    async fn vs<'ctx>(&self, ctx: &Context<'ctx>) -> i64 {
        telemetry::graphql_query("Vehicle", "vs");

        self.fetch(
            ctx,
            Filters {
                faction: Some(IdOrNameBy::Id(VS)),
                ..self.filters.clone()
            },
        )
        .await
    }
}

/// Super-struct for all vehicles.
pub struct Vehicles {
    filters: Filters,
}

impl Vehicles {
    pub fn new(filters: Option<Filters>) -> Self {
        Self {
            filters: filters.unwrap_or_default(),
        }
    }
}

#[Object]
impl Vehicles {
    async fn total<'ctx>(&self, ctx: &Context<'ctx>) -> i64 {
        telemetry::graphql_query("Vehicles", "total");

        let pool = ctx.data::<Pool<Postgres>>().unwrap();

        telemetry::db_read("players", "vehicles_total");
        let sql = format!(
            "SELECT count(*) FROM vehicles WHERE last_updated > now() - interval '15 minutes' {};",
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

    // Transport
    async fn flash(&self) -> Vehicle {
        telemetry::graphql_query("Vehicle", "flash");

        Vehicle {
            filters: self.filters.clone(),
            vehicle_name: "flash".to_string(),
        }
    }
    async fn sunderer(&self) -> Vehicle {
        telemetry::graphql_query("Vehicle", "sunderer");

        Vehicle {
            filters: self.filters.clone(),
            vehicle_name: "sunderer".to_string(),
        }
    }
    async fn ant(&self) -> Vehicle {
        telemetry::graphql_query("Vehicle", "ant");

        Vehicle {
            filters: self.filters.clone(),
            vehicle_name: "ant".to_string(),
        }
    }
    async fn harasser(&self) -> Vehicle {
        telemetry::graphql_query("Vehicle", "harasser");

        Vehicle {
            filters: self.filters.clone(),
            vehicle_name: "harasser".to_string(),
        }
    }
    async fn javelin(&self) -> Vehicle {
        telemetry::graphql_query("Vehicle", "javelin");

        Vehicle {
            filters: self.filters.clone(),
            vehicle_name: "javelin".to_string(),
        }
    }
    async fn corsair(&self) -> Vehicle {
        telemetry::graphql_query("Vehicle", "corsair");

        Vehicle {
            filters: self.filters.clone(),
            vehicle_name: "corsair".to_string(),
        }
    }

    // Tanks
    async fn lightning(&self) -> Vehicle {
        telemetry::graphql_query("Vehicle", "lightning");

        Vehicle {
            filters: self.filters.clone(),
            vehicle_name: "lightning".to_string(),
        }
    }
    async fn prowler(&self) -> Vehicle {
        telemetry::graphql_query("Vehicle", "prowler");

        Vehicle {
            filters: self.filters.clone(),
            vehicle_name: "prowler".to_string(),
        }
    }
    async fn vanguard(&self) -> Vehicle {
        telemetry::graphql_query("Vehicle", "vanguard");

        Vehicle {
            filters: self.filters.clone(),
            vehicle_name: "vanguard".to_string(),
        }
    }
    async fn magrider(&self) -> Vehicle {
        telemetry::graphql_query("Vehicle", "magrider");

        Vehicle {
            filters: self.filters.clone(),
            vehicle_name: "magrider".to_string(),
        }
    }
    async fn chimera(&self) -> Vehicle {
        telemetry::graphql_query("Vehicle", "chimera");

        Vehicle {
            filters: self.filters.clone(),
            vehicle_name: "chimera".to_string(),
        }
    }

    // Air
    async fn mosquito(&self) -> Vehicle {
        telemetry::graphql_query("Vehicle", "mosquito");

        Vehicle {
            filters: self.filters.clone(),
            vehicle_name: "mosquito".to_string(),
        }
    }
    async fn liberator(&self) -> Vehicle {
        telemetry::graphql_query("Vehicle", "liberator");

        Vehicle {
            filters: self.filters.clone(),
            vehicle_name: "liberator".to_string(),
        }
    }
    async fn galaxy(&self) -> Vehicle {
        telemetry::graphql_query("Vehicle", "galaxy");

        Vehicle {
            filters: self.filters.clone(),
            vehicle_name: "galaxy".to_string(),
        }
    }
    async fn valkyrie(&self) -> Vehicle {
        telemetry::graphql_query("Vehicle", "valkyrie");

        Vehicle {
            filters: self.filters.clone(),
            vehicle_name: "valkyrie".to_string(),
        }
    }
    async fn reaver(&self) -> Vehicle {
        telemetry::graphql_query("Vehicle", "reaver");

        Vehicle {
            filters: self.filters.clone(),
            vehicle_name: "reaver".to_string(),
        }
    }
    async fn scythe(&self) -> Vehicle {
        telemetry::graphql_query("Vehicle", "scythe");

        Vehicle {
            filters: self.filters.clone(),
            vehicle_name: "scythe".to_string(),
        }
    }
    async fn dervish(&self) -> Vehicle {
        telemetry::graphql_query("Vehicle", "dervish");

        Vehicle {
            filters: self.filters.clone(),
            vehicle_name: "dervish".to_string(),
        }
    }
}

#[derive(Default)]
pub struct VehicleQuery;

#[Object]
impl VehicleQuery {
    pub async fn vehicles(&self, filter: Option<Filters>) -> Vehicles {
        Vehicles::new(filter)
    }
}
