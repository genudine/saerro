use crate::util::zcount;
use async_graphql::{Context, Object};
use redis::aio::MultiplexedConnection;

pub struct Classes {
    world_id: String,
}

impl Classes {
    pub fn new(world_id: String) -> Self {
        Self { world_id }
    }
    async fn by_class<'ctx>(&self, ctx: &Context<'ctx>, class_name: &str) -> u32 {
        let con = ctx.data::<MultiplexedConnection>().unwrap().to_owned();
        zcount(con, format!("c:{}/{}", self.world_id, class_name)).await
    }
}

#[Object]
impl Classes {
    async fn infiltrator<'ctx>(&self, ctx: &Context<'ctx>) -> u32 {
        self.by_class(ctx, "infiltrator").await
    }
    async fn light_assault<'ctx>(&self, ctx: &Context<'ctx>) -> u32 {
        self.by_class(ctx, "light_assault").await
    }
    async fn combat_medic<'ctx>(&self, ctx: &Context<'ctx>) -> u32 {
        self.by_class(ctx, "combat_medic").await
    }
    async fn engineer<'ctx>(&self, ctx: &Context<'ctx>) -> u32 {
        self.by_class(ctx, "engineer").await
    }
    async fn heavy_assault<'ctx>(&self, ctx: &Context<'ctx>) -> u32 {
        self.by_class(ctx, "heavy_assault").await
    }
    async fn max<'ctx>(&self, ctx: &Context<'ctx>) -> u32 {
        self.by_class(ctx, "max").await
    }
}
