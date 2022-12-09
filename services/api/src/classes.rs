use crate::utils::{Filters, IdOrNameBy};
use async_graphql::{Context, Object};
use sqlx::{Pool, Postgres, Row};

/// A specific with optional faction filter.
pub struct Class {
    filters: Filters,
    class_name: String,
}

impl Class {
    async fn fetch<'ctx>(&self, ctx: &Context<'ctx>, filters: Filters) -> i64 {
        let pool = ctx.data::<Pool<Postgres>>().unwrap();

        let sql = format!(
            "SELECT count(distinct character_id) FROM classes WHERE time > now() - interval '15 minutes' AND class_id = $1 {};",
            filters.sql(),
        );

        println!("{}", sql);

        let query: i64 = sqlx::query(sql.as_str())
            .bind(self.class_name.as_str())
            .fetch_one(pool)
            .await
            .unwrap()
            .get(0);

        query
    }
}

#[Object]
impl Class {
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

/// Super-struct of each class.
pub struct Classes {
    filters: Filters,
}

impl Classes {
    pub fn new(filters: Option<Filters>) -> Self {
        Self {
            filters: filters.unwrap_or_default(),
        }
    }
}

#[Object]
impl Classes {
    async fn infiltrator(&self) -> Class {
        Class {
            filters: self.filters.clone(),
            class_name: "infiltrator".to_string(),
        }
    }
    async fn light_assault(&self) -> Class {
        Class {
            filters: self.filters.clone(),
            class_name: "light_assault".to_string(),
        }
    }
    async fn combat_medic(&self) -> Class {
        Class {
            filters: self.filters.clone(),
            class_name: "combat_medic".to_string(),
        }
    }
    async fn engineer(&self) -> Class {
        Class {
            filters: self.filters.clone(),
            class_name: "engineer".to_string(),
        }
    }
    async fn heavy_assault(&self) -> Class {
        Class {
            filters: self.filters.clone(),
            class_name: "heavy_assault".to_string(),
        }
    }
    async fn max(&self) -> Class {
        Class {
            filters: self.filters.clone(),
            class_name: "max".to_string(),
        }
    }
}

#[derive(Default)]
pub struct ClassesQuery;

#[Object]
impl ClassesQuery {
    /// Get all classes
    pub async fn classes(&self, filter: Option<Filters>) -> Classes {
        Classes::new(filter)
    }

    /// Get a specific class
    pub async fn class(&self, filter: Option<Filters>, class_name: String) -> Class {
        Class {
            filters: filter.unwrap_or_default(),
            class_name,
        }
    }
}
