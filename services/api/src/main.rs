mod classes;
mod health;
mod query;
mod util;
mod vehicles;
mod world;

use async_graphql::{
    extensions::ApolloTracing,
    http::{playground_source, GraphQLPlaygroundConfig},
    EmptyMutation, EmptySubscription, Request, Response, Schema,
};
use axum::{
    extract::Query,
    http::Method,
    response::{Html, IntoResponse, Redirect},
    routing::{get, post},
    Extension, Json, Router,
};
use std::net::SocketAddr;
use tower_http::cors::{Any, CorsLayer};

#[macro_use]
extern crate serde_json;

async fn index() -> Html<&'static str> {
    Html(include_str!("html/index.html"))
}

async fn handle_404() -> Html<&'static str> {
    Html(include_str!("html/404.html"))
}

async fn graphql_handler_post(
    Extension(schema): Extension<Schema<query::Query, EmptyMutation, EmptySubscription>>,
    Json(query): Json<Request>,
) -> Json<Response> {
    Json(schema.execute(query).await)
}

async fn graphql_handler_get(
    Extension(schema): Extension<Schema<query::Query, EmptyMutation, EmptySubscription>>,
    query: Query<Request>,
) -> axum::response::Response {
    match query.operation_name {
        Some(_) => Json(schema.execute(query.0).await).into_response(),
        None => Redirect::to("/graphql/playground").into_response(),
    }
}
async fn graphql_playground() -> impl IntoResponse {
    Html(playground_source(GraphQLPlaygroundConfig::new("/graphql")))
}

#[tokio::main]
async fn main() {
    let redis_url = format!(
        "redis://{}:{}",
        std::env::var("REDIS_HOST").unwrap_or("localhost".to_string()),
        std::env::var("REDIS_PORT").unwrap_or("6379".to_string()),
    );

    let redis = redis::Client::open(redis_url)
        .unwrap()
        .get_multiplexed_tokio_connection()
        .await
        .unwrap();

    let schema = Schema::build(query::Query, EmptyMutation, EmptySubscription)
        .data(redis.clone())
        .extension(ApolloTracing)
        .finish();

    let app = Router::new()
        .route("/", get(index))
        .route("/health", get(health::get_health))
        .route(
            "/graphql",
            post(graphql_handler_post).get(graphql_handler_get),
        )
        .route("/graphql/playground", get(graphql_playground))
        .fallback(handle_404)
        .layer(Extension(redis))
        .layer(Extension(schema))
        .layer(CorsLayer::new().allow_origin(Any).allow_methods([
            Method::GET,
            Method::POST,
            Method::OPTIONS,
        ]));

    let port: u16 = std::env::var("PORT")
        .unwrap_or("8000".to_string())
        .parse()
        .unwrap();
    let addr = SocketAddr::from(([0, 0, 0, 0], port));

    println!("Listening on http://{}", addr);

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
