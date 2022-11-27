use crate::health::Health;
use crate::world::World;
use async_graphql::Object;

pub struct Query;

#[Object]
impl Query {
    /// Returns a graph for the world with the given ID.
    /// If the world does not exist, this will not fail.
    async fn world(&self, id: String) -> World {
        World { id: id.clone() }
    }

    /// Returns a graph for the world specified by it's human name.
    /// This is case-insensitive; but will not fail.
    async fn world_by_name(&self, name: String) -> World {
        World::from_name(name)
    }

    /// Returns a graph of all known live play worlds.
    async fn all_worlds(&self) -> Vec<World> {
        World::all_worlds()
    }

    /// Reports on the health of Saerro Listening Post
    async fn health(&self) -> Health {
        Health {}
    }
}
