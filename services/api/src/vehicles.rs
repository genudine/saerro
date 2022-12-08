use async_graphql::{Context, Object};

pub struct Vehicles {
    world_id: String,
}

impl Vehicles {
    pub fn new(world_id: String) -> Self {
        Self { world_id }
    }
    async fn by_vehicle<'ctx>(&self, ctx: &Context<'ctx>, vehicle_name: &str) -> u32 {
        0
    }
}

#[Object]
impl Vehicles {
    async fn flash<'ctx>(&self, ctx: &Context<'ctx>) -> u32 {
        self.by_vehicle(ctx, "flash").await
    }
    async fn sunderer<'ctx>(&self, ctx: &Context<'ctx>) -> u32 {
        self.by_vehicle(ctx, "sunderer").await
    }
    async fn ant<'ctx>(&self, ctx: &Context<'ctx>) -> u32 {
        self.by_vehicle(ctx, "ant").await
    }
    async fn harasser<'ctx>(&self, ctx: &Context<'ctx>) -> u32 {
        self.by_vehicle(ctx, "harasser").await
    }
    async fn javelin<'ctx>(&self, ctx: &Context<'ctx>) -> u32 {
        self.by_vehicle(ctx, "javelin").await
    }

    // Tanks
    async fn lightning<'ctx>(&self, ctx: &Context<'ctx>) -> u32 {
        self.by_vehicle(ctx, "lightning").await
    }
    async fn prowler<'ctx>(&self, ctx: &Context<'ctx>) -> u32 {
        self.by_vehicle(ctx, "prowler").await
    }
    async fn vanguard<'ctx>(&self, ctx: &Context<'ctx>) -> u32 {
        self.by_vehicle(ctx, "vanguard").await
    }
    async fn magrider<'ctx>(&self, ctx: &Context<'ctx>) -> u32 {
        self.by_vehicle(ctx, "magrider").await
    }
    async fn chimera<'ctx>(&self, ctx: &Context<'ctx>) -> u32 {
        self.by_vehicle(ctx, "chimera").await
    }

    // Air
    async fn mosquito<'ctx>(&self, ctx: &Context<'ctx>) -> u32 {
        self.by_vehicle(ctx, "mosquito").await
    }
    async fn liberator<'ctx>(&self, ctx: &Context<'ctx>) -> u32 {
        self.by_vehicle(ctx, "liberator").await
    }
    async fn galaxy<'ctx>(&self, ctx: &Context<'ctx>) -> u32 {
        self.by_vehicle(ctx, "galaxy").await
    }
    async fn valkyrie<'ctx>(&self, ctx: &Context<'ctx>) -> u32 {
        self.by_vehicle(ctx, "valkyrie").await
    }
    async fn reaver<'ctx>(&self, ctx: &Context<'ctx>) -> u32 {
        self.by_vehicle(ctx, "reaver").await
    }
    async fn scythe<'ctx>(&self, ctx: &Context<'ctx>) -> u32 {
        self.by_vehicle(ctx, "scythe").await
    }
    async fn dervish<'ctx>(&self, ctx: &Context<'ctx>) -> u32 {
        self.by_vehicle(ctx, "dervish").await
    }
}
