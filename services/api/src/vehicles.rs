use crate::utils::{Filters, IdOrNameBy};
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

        let sql = format!(
            "SELECT count(distinct character_id) FROM vehicles WHERE time > now() - interval '15 minutes' AND vehicle_id = $1 {};",
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
        self.fetch(ctx, self.filters.clone()).await
    }
    async fn nc<'ctx>(&self, ctx: &Context<'ctx>) -> i64 {
        self.fetch(
            ctx,
            Filters {
                faction: Some(IdOrNameBy::Id(1)),
                ..self.filters.clone()
            },
        )
        .await
    }
    async fn tr<'ctx>(&self, ctx: &Context<'ctx>) -> i64 {
        self.fetch(
            ctx,
            Filters {
                faction: Some(IdOrNameBy::Id(2)),
                ..self.filters.clone()
            },
        )
        .await
    }
    async fn vs<'ctx>(&self, ctx: &Context<'ctx>) -> i64 {
        self.fetch(
            ctx,
            Filters {
                faction: Some(IdOrNameBy::Id(3)),
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
        let pool = ctx.data::<Pool<Postgres>>().unwrap();

        let sql = format!(
            "SELECT count(distinct character_id) FROM vehicles WHERE time > now() - interval '15 minutes' {};",
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
        Vehicle {
            filters: self.filters.clone(),
            vehicle_name: "flash".to_string(),
        }
    }
    async fn sunderer(&self) -> Vehicle {
        Vehicle {
            filters: self.filters.clone(),
            vehicle_name: "sunderer".to_string(),
        }
    }
    async fn ant(&self) -> Vehicle {
        Vehicle {
            filters: self.filters.clone(),
            vehicle_name: "ant".to_string(),
        }
    }
    async fn harasser(&self) -> Vehicle {
        Vehicle {
            filters: self.filters.clone(),
            vehicle_name: "harasser".to_string(),
        }
    }
    async fn javelin(&self) -> Vehicle {
        Vehicle {
            filters: self.filters.clone(),
            vehicle_name: "javelin".to_string(),
        }
    }

    // Tanks
    async fn lightning(&self) -> Vehicle {
        Vehicle {
            filters: self.filters.clone(),
            vehicle_name: "javelin".to_string(),
        }
    }
    async fn prowler(&self) -> Vehicle {
        Vehicle {
            filters: self.filters.clone(),
            vehicle_name: "prowler".to_string(),
        }
    }
    async fn vanguard(&self) -> Vehicle {
        Vehicle {
            filters: self.filters.clone(),
            vehicle_name: "vanguard".to_string(),
        }
    }
    async fn magrider(&self) -> Vehicle {
        Vehicle {
            filters: self.filters.clone(),
            vehicle_name: "magrider".to_string(),
        }
    }
    async fn chimera(&self) -> Vehicle {
        Vehicle {
            filters: self.filters.clone(),
            vehicle_name: "chimera".to_string(),
        }
    }

    // Air
    async fn mosquito(&self) -> Vehicle {
        Vehicle {
            filters: self.filters.clone(),
            vehicle_name: "mosquito".to_string(),
        }
    }
    async fn liberator(&self) -> Vehicle {
        Vehicle {
            filters: self.filters.clone(),
            vehicle_name: "liberator".to_string(),
        }
    }
    async fn galaxy(&self) -> Vehicle {
        Vehicle {
            filters: self.filters.clone(),
            vehicle_name: "galaxy".to_string(),
        }
    }
    async fn valkyrie(&self) -> Vehicle {
        Vehicle {
            filters: self.filters.clone(),
            vehicle_name: "valkyrie".to_string(),
        }
    }
    async fn reaver(&self) -> Vehicle {
        Vehicle {
            filters: self.filters.clone(),
            vehicle_name: "reaver".to_string(),
        }
    }
    async fn scythe(&self) -> Vehicle {
        Vehicle {
            filters: self.filters.clone(),
            vehicle_name: "scythe".to_string(),
        }
    }
    async fn dervish(&self) -> Vehicle {
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
