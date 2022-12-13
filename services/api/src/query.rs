use crate::{
    analytics::AnalyticsQuery, classes::ClassesQuery, health::HealthQuery,
    population::PopulationQuery, vehicles::VehicleQuery, world::WorldQuery, zone::ZoneQuery,
};
use async_graphql::MergedObject;

#[derive(MergedObject, Default)]
pub struct Query(
    PopulationQuery,
    VehicleQuery,
    ClassesQuery,
    WorldQuery,
    ZoneQuery,
    HealthQuery,
    AnalyticsQuery,
);
