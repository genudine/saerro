use async_graphql::{InputObject, OneofObject};
use lazy_static::lazy_static;
use std::collections::HashMap;

lazy_static! {
    pub static ref WORLD_IDS: HashMap<String, i32> = HashMap::from([
        ("connery".to_string(), 1),
        ("miller".to_string(), 10),
        ("cobalt".to_string(), 13),
        ("emerald".to_string(), 17),
        ("jaeger".to_string(), 19),
        ("soltech".to_string(), 40),
        ("genudine".to_string(), 1000),
        ("ceres".to_string(), 2000),
    ]);
    pub static ref ID_TO_WORLD: HashMap<i32, String> = WORLD_IDS
        .iter()
        .map(|(name, id)| (id.to_owned(), name.to_owned()))
        .collect();
    pub static ref FACTION_IDS: HashMap<String, i32> = HashMap::from([
        ("vs".to_string(), 1),
        ("nc".to_string(), 2),
        ("tr".to_string(), 3),
        ("ns".to_string(), 4),
    ]);
    pub static ref ID_TO_FACTION: HashMap<i32, String> = FACTION_IDS
        .iter()
        .map(|(name, id)| (id.to_owned(), name.to_owned()))
        .collect();
    pub static ref ZONE_IDS: HashMap<String, i32> = HashMap::from([
        ("indar".to_string(), 2),
        ("hossin".to_string(), 4),
        ("amerish".to_string(), 6),
        ("esamir".to_string(), 8),
        ("oshur".to_string(), 344),
    ]);
    pub static ref ID_TO_ZONE: HashMap<i32, String> = ZONE_IDS
        .iter()
        .map(|(name, id)| (id.to_owned(), name.to_owned()))
        .collect();
}

/// Allows for one of the following:
/// - By ID, example: `{ id: 1 }`
/// - By name (case-insensitive), example: `{ name: "Connery" }`
#[derive(OneofObject, Clone)]
pub enum IdOrNameBy {
    Id(i32),
    Name(String),
}

pub fn id_or_name_to_id(map: &HashMap<String, i32>, by: &IdOrNameBy) -> Option<i32> {
    match by {
        IdOrNameBy::Id(id) => Some(*id),
        IdOrNameBy::Name(name) => map.get(&name.to_lowercase()).map(|id| *id),
    }
}

pub fn id_or_name_to_name(map: &HashMap<i32, String>, id: &IdOrNameBy) -> Option<String> {
    match id {
        IdOrNameBy::Id(id) => map.get(id).map(|name| name.to_owned()),
        IdOrNameBy::Name(name) => Some(name.to_owned()),
    }
}

/// A filter for core queries, allows for filtering by world, faction, and zone.
/// Omitting a field will not filter by that field, so for example:
/// `{ world: { id: 1 }, faction: { name: "VS" } }`
/// will filter by world ID 1 and faction name "VS", but also search in every continent.
#[derive(InputObject, Default, Clone)]
pub struct Filters {
    /// The world to filter by, like Connery, Emerald, etc.
    pub world: Option<IdOrNameBy>,
    /// The faction to filter by, like VS, NC, TR, or NS
    pub faction: Option<IdOrNameBy>,
    /// The zone or continent to filter by, like Indar, Amerish, etc.
    pub zone: Option<IdOrNameBy>,
}

impl Filters {
    pub fn sql(&self) -> String {
        let mut sql = String::new();
        if let Some(world) = &self.world {
            if let Some(world_id) = id_or_name_to_id(&WORLD_IDS, world) {
                sql.push_str(&format!(" AND world_id = {}", world_id));
            }
        }
        if let Some(faction) = &self.faction {
            if let Some(faction_id) = id_or_name_to_id(&FACTION_IDS, faction) {
                sql.push_str(&format!(" AND faction_id = {}", faction_id));
            }
        }
        if let Some(zone) = &self.zone {
            if let Some(zone_id) = id_or_name_to_id(&ZONE_IDS, zone) {
                sql.push_str(&format!(" AND zone_id = {}", zone_id));
            }
        }
        sql
    }
}
