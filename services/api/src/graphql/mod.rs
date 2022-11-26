use crate::redispool::RedisPool;

use self::types::{Health, World};
use juniper::{graphql_object, meta::Field, FieldResult, ID};
use rocket::response::content::RawHtml;

pub mod types;

pub struct Query;

#[graphql_object(context = Context)]
impl Query {
    fn world(id: ID) -> FieldResult<World> {
        Ok(World { id })
    }

    fn allWorlds() -> FieldResult<Vec<World>> {
        Ok(vec![
            World {
                id: ID::from("1".to_string()),
            },
            World {
                id: ID::from("10".to_string()),
            },
            World {
                id: ID::from("13".to_string()),
            },
            World {
                id: ID::from("17".to_string()),
            },
            World {
                id: ID::from("19".to_string()),
            },
            World {
                id: ID::from("40".to_string()),
            },
            World {
                id: ID::from("1000".to_string()),
            },
            World {
                id: ID::from("2000".to_string()),
            },
        ])
    }

    fn worldByName(name: String) -> FieldResult<World> {
        let id = match name.to_lowercase().as_str() {
            "connery" => "1",
            "miller" => "10",
            "cobalt" => "13",
            "emerald" => "17",
            "jaeger" => "19",
            "soltech" => "40",
            "genudine" => "1000",
            "ceres" => "2000",
            _ => "-1",
        };

        Ok(World {
            id: ID::from(id.to_string()),
        })
    }

    fn health() -> FieldResult<Health> {
        Ok(Health {})
    }
}

pub struct Context {
    con: RedisPool,
}

impl juniper::Context for Context {}

#[get("/graphiql")]
pub fn graphiql() -> RawHtml<String> {
    juniper_rocket::graphiql_source("/graphql", None)
}

#[get("/")]
pub fn playground() -> RawHtml<String> {
    juniper_rocket::playground_source("/graphql", None)
}

#[post("/", data = "<query>")]
pub async fn post_graphql(
    query: juniper_rocket::GraphQLRequest,
    schema: &rocket::State<Schema>,
    con: &RedisPool,
) -> juniper_rocket::GraphQLResponse {
    query.execute(&*schema, &Context { con: con.clone() }).await
}

#[get("/?<query..>")]
pub async fn get_graphql(
    query: juniper_rocket::GraphQLRequest,
    schema: &rocket::State<Schema>,
    con: &RedisPool,
) -> juniper_rocket::GraphQLResponse {
    query.execute(&*schema, &Context { con: con.clone() }).await
}

pub type Schema = juniper::RootNode<
    'static,
    Query,
    juniper::EmptyMutation<Context>,
    juniper::EmptySubscription<Context>,
>;

pub fn schema() -> Schema {
    Schema::new(
        Query,
        juniper::EmptyMutation::<Context>::new(),
        juniper::EmptySubscription::<Context>::new(),
    )
}
