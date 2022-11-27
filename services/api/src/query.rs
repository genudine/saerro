use crate::health::Health;
use crate::world::World;
use async_graphql::Object;

pub struct Query;

#[Object]
impl Query {
    async fn world(&self, id: String) -> World {
        World { id: id.clone() }
    }

    async fn world_by_name(&self, name: String) -> World {
        World::from_name(name)
    }

    async fn all_worlds(&self) -> Vec<World> {
        World::all_worlds()
    }

    async fn health(&self) -> Health {
        Health {}
    }
}
