use crate::{
    classes::Classes,
    population::Population,
    utils::{id_or_name_to_id, id_or_name_to_name, Filters, IdOrNameBy, ID_TO_WORLD, WORLD_IDS},
    vehicles::Vehicles,
    zone::Zones,
};
use async_graphql::Object;

pub struct World {
    filter: Filters,
}

impl World {
    pub fn new(filter: IdOrNameBy) -> Self {
        Self {
            filter: Filters {
                world: Some(filter),
                faction: None,
                zone: None,
            },
        }
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
    /// The ID of the world.
    async fn id(&self) -> i32 {
        id_or_name_to_id(&WORLD_IDS, self.filter.world.as_ref().unwrap()).unwrap()
    }

    /// The name of the world, in official game capitalization.
    async fn name(&self) -> String {
        let name = id_or_name_to_name(&ID_TO_WORLD, self.filter.world.as_ref().unwrap()).unwrap();

        // Special case for SolTech, lol.
        if name == "soltech" {
            return "SolTech".to_string();
        }

        // Capitalize the first letter
        name[0..1].to_uppercase() + &name[1..]
    }

    /// Population filtered to this world.
    async fn population(&self) -> Population {
        Population::new(Some(Filters {
            world: self.filter.world.clone(),
            faction: None,
            zone: None,
        }))
    }

    /// Vehicles filtered to this world.
    async fn vehicles(&self) -> Vehicles {
        Vehicles::new(Some(Filters {
            world: self.filter.world.clone(),
            faction: None,
            zone: None,
        }))
    }

    /// Classes filtered to this world.
    async fn classes(&self) -> Classes {
        Classes::new(Some(Filters {
            world: self.filter.world.clone(),
            faction: None,
            zone: None,
        }))
    }

    /// Get a specific zone/continent on this world.
    async fn zones(&self) -> Zones {
        Zones::new(Some(self.filter.clone()))
    }
}

#[derive(Default)]
pub struct WorldQuery;

#[Object]
impl WorldQuery {
    /// A world by ID or name.
    pub async fn world(&self, by: IdOrNameBy) -> World {
        World::new(by)
    }

    /// All worlds. This is a convenience method for getting all worlds in one query.
    /// If you want all of them as aggregate instead of as individual units, use `population`, `vehicles`, `classes` directly instead.
    pub async fn all_worlds(&self) -> Vec<World> {
        ID_TO_WORLD
            .iter()
            .map(|(id, _)| World::new(IdOrNameBy::Id(*id)))
            .collect()
    }

    /// The Connery world in US West on PC
    /// Shorthand for `world(by: { id: 1 }})`
    pub async fn connery(&self) -> World {
        World::new(IdOrNameBy::Id(1))
    }

    /// The Miller world in EU on PC
    /// Shorthand for `world(by: { id: 10 }})`
    pub async fn miller(&self) -> World {
        World::new(IdOrNameBy::Id(10))
    }

    /// The Cobalt world in EU on PC
    /// Shorthand for `world(by: { id: 13 }})`
    pub async fn cobalt(&self) -> World {
        World::new(IdOrNameBy::Id(13))
    }

    /// The Emerald world in US East on PC
    /// Shorthand for `world(by: { id: 17 }})`
    pub async fn emerald(&self) -> World {
        World::new(IdOrNameBy::Id(17))
    }

    /// The Jaeger world in US East on PC
    /// Shorthand for `world(by: { id: 19 }})`
    pub async fn jaeger(&self) -> World {
        World::new(IdOrNameBy::Id(19))
    }

    /// The SolTech world in Japan on PC
    /// Shorthand for `world(by: { id: 40 }})`
    pub async fn soltech(&self) -> World {
        World::new(IdOrNameBy::Id(40))
    }

    /// The Genudine world in US East on PS4
    /// Shorthand for `world(by: { id: 1000 }})`
    pub async fn genudine(&self) -> World {
        World::new(IdOrNameBy::Id(1000))
    }

    /// The Ceres world in EU on PS4
    /// Shorthand for `world(by: { id: 2000 }})`
    pub async fn ceres(&self) -> World {
        World::new(IdOrNameBy::Id(2000))
    }
}
