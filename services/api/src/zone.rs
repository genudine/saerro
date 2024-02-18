use crate::{
    classes::Classes,
    population::Population,
    utils::{id_or_name_to_id, id_or_name_to_name, Filters, IdOrNameBy, ID_TO_ZONE, ZONE_IDS},
    vehicles::Vehicles,
    telemetry,
};
use async_graphql::Object;

/// An individual zone/continent.
pub struct Zone {
    filters: Filters,
}

impl Zone {
    pub fn new(filters: Option<Filters>) -> Self {
        Self {
            filters: filters.unwrap_or_default(),
        }
    }
}

#[Object]
impl Zone {
    /// The ID of the zone/continent.
    async fn id(&self) -> i32 {
        telemetry::graphql_query("Zone", "id");

        id_or_name_to_id(&ZONE_IDS, self.filters.zone.as_ref().unwrap()).unwrap()
    }

    /// The name of the continent, in official game capitalization.
    async fn name(&self) -> String {
        telemetry::graphql_query("Zone", "name");

        let name = id_or_name_to_name(&ID_TO_ZONE, self.filters.zone.as_ref().unwrap()).unwrap();

        // Capitalize the first letter
        name[0..1].to_uppercase() + &name[1..]
    }

    async fn population(&self) -> Population {
        telemetry::graphql_query("Zone", "population");

        Population::new(Some(self.filters.clone()))
    }

    async fn vehicles(&self) -> Vehicles {
        telemetry::graphql_query("Zone", "vehicles");

        Vehicles::new(Some(self.filters.clone()))
    }

    async fn classes(&self) -> Classes {
        telemetry::graphql_query("Zone", "classes");

        Classes::new(Some(self.filters.clone()))
    }
}

/// Super-struct for querying zones/continents.
pub struct Zones {
    filters: Filters,
}

impl Zones {
    pub fn new(filters: Option<Filters>) -> Self {
        Self {
            filters: filters.unwrap_or_default(),
        }
    }
}

#[Object]
impl Zones {
    /// Every zone/continent individually.
    async fn all(&self) -> Vec<Zone> {
        ID_TO_ZONE
            .iter()
            .map(|(id, _)| {
                Zone::new(Some(Filters {
                    world: self.filters.world.clone(),
                    faction: self.filters.faction.clone(),
                    zone: Some(IdOrNameBy::Id(*id)),
                }))
            })
            .collect()
    }

    async fn indar(&self) -> Zone {
        Zone::new(Some(Filters {
            world: self.filters.world.clone(),
            faction: self.filters.faction.clone(),
            zone: Some(IdOrNameBy::Id(2)),
        }))
    }

    async fn hossin(&self) -> Zone {
        Zone::new(Some(Filters {
            world: self.filters.world.clone(),
            faction: self.filters.faction.clone(),
            zone: Some(IdOrNameBy::Id(4)),
        }))
    }

    async fn amerish(&self) -> Zone {
        Zone::new(Some(Filters {
            world: self.filters.world.clone(),
            faction: self.filters.faction.clone(),
            zone: Some(IdOrNameBy::Id(6)),
        }))
    }

    async fn esamir(&self) -> Zone {
        Zone::new(Some(Filters {
            world: self.filters.world.clone(),
            faction: self.filters.faction.clone(),
            zone: Some(IdOrNameBy::Id(8)),
        }))
    }

    async fn oshur(&self) -> Zone {
        Zone::new(Some(Filters {
            world: self.filters.world.clone(),
            faction: self.filters.faction.clone(),
            zone: Some(IdOrNameBy::Id(344)),
        }))
    }
}

#[derive(Default)]
pub struct ZoneQuery;

#[Object]
impl ZoneQuery {
    pub async fn zone(&self, filter: Option<Filters>) -> Zone {
        Zone::new(filter)
    }

    pub async fn zones(&self, filter: Option<Filters>) -> Zones {
        Zones::new(filter)
    }
}
