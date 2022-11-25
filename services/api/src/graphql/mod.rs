use crate::redispool::RedisPool;

use self::types::World;
use juniper::{graphql_object, FieldResult, ID};
use rocket::response::content::RawHtml;

pub mod types;

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

pub struct Query;

#[graphql_object(context = Context)]
impl Query {
    fn world(id: ID) -> FieldResult<Option<World>> {
        Ok(Some(World { id }))
    }
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
