use crate::{classes::Classes, vehicles::Vehicles};
use async_graphql::{Context, Object};
use lazy_static::lazy_static;
use std::collections::HashMap;

lazy_static! {
    static ref WORLD_NAME_TO_ID: HashMap<&'static str, &'static str> = HashMap::from([
        ("connery", "1"),
        ("miller", "10"),
        ("cobalt", "13"),
        ("emerald", "17"),
        ("jaeger", "19"),
        ("soltech", "40"),
        ("genudine", "1000"),
        ("ceres", "2000"),
    ]);
    static ref WORLD_ID_TO_NAME: HashMap<&'static str, &'static str> = HashMap::from([
        ("1", "Connery"),
        ("10", "Miller"),
        ("13", "Cobalt"),
        ("17", "Emerald"),
        ("19", "Jaeger"),
        ("40", "SolTech"),
        ("1000", "Genudine"),
        ("2000", "Ceres"),
    ]);
}
pub struct World {
    pub id: String,
}

impl World {
    pub fn from_name(name: String) -> World {
        let id = WORLD_NAME_TO_ID
            .get(name.to_lowercase().as_str())
            .unwrap_or(&"-1");

        World { id: id.to_string() }
    }

    pub fn all_worlds() -> Vec<World> {
        WORLD_ID_TO_NAME
            .keys()
            .map(|id| World { id: id.to_string() })
            .collect()
    }
}

/// **A PlanetSide 2 world.**
///
/// This can be fetched at the top level with `world(id: "1")` or `worldByName(name: "Connery")`.
/// ...or get all of them with `allWorlds`.
///
/// If World.id is not valid or known to the API, World.name will return "Unknown".
#[Object]
impl World {
    async fn id(&self) -> &str {
        &self.id
    }

    async fn name(&self) -> String {
        WORLD_ID_TO_NAME
            .get(self.id.as_str())
            .unwrap_or(&"Unknown")
            .to_string()
    }

    async fn population<'ctx>(&self, ctx: &Context<'ctx>) -> u32 {
        0
    }

    async fn faction_population(&self) -> FactionPopulation {
        FactionPopulation {
            world_id: self.id.clone(),
        }
    }

    async fn vehicles(&self) -> Vehicles {
        Vehicles::new(self.id.clone())
    }

    async fn classes(&self) -> Classes {
        Classes::new(self.id.clone())
    }
}

struct FactionPopulation {
    world_id: String,
}

impl FactionPopulation {
    async fn by_faction<'ctx>(&self, ctx: &Context<'ctx>, faction: u8) -> u32 {
        0
    }
}

#[Object]
impl FactionPopulation {
    async fn vs<'ctx>(&self, ctx: &Context<'ctx>) -> u32 {
        self.by_faction(ctx, 1).await
    }

    async fn nc<'ctx>(&self, ctx: &Context<'ctx>) -> u32 {
        self.by_faction(ctx, 2).await
    }

    async fn tr<'ctx>(&self, ctx: &Context<'ctx>) -> u32 {
        self.by_faction(ctx, 3).await
    }

    async fn ns<'ctx>(&self, ctx: &Context<'ctx>) -> u32 {
        self.by_faction(ctx, 4).await
    }
}
