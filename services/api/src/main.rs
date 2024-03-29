mod analytics;
mod classes;
mod factions;
mod health;
mod population;
mod query;
mod telemetry;
mod utils;
mod vehicles;
mod world;
mod zone;

use async_graphql::{
    http::GraphiQLSource, EmptyMutation, EmptySubscription, Request, Response, Schema,
};
use axum::{
    extract::Query,
    http::{header::CONTENT_TYPE, Method},
    response::{Html, IntoResponse, Redirect},
    routing::{get, post},
    Extension, Json, Router,
};
use std::net::SocketAddr;
use tower_http::cors::{Any, CorsLayer};

#[macro_use]
extern crate serde_json;

async fn index() -> Html<&'static str> {
    telemetry::http_request("/", "GET");
    Html(include_str!("html/index.html"))
}

async fn ingest() -> Html<&'static str> {
    telemetry::http_request("/ingest", "GET");
    Html(include_str!("html/ingest.html"))
}

async fn handle_404() -> Html<&'static str> {
    Html(include_str!("html/404.html"))
}

async fn graphql_handler_post(
    Extension(schema): Extension<Schema<query::Query, EmptyMutation, EmptySubscription>>,
    Json(query): Json<Request>,
) -> Json<Response> {
    telemetry::http_request("/graphql", "POST");
    Json(schema.execute(query).await)
}

async fn graphql_handler_get(
    Extension(schema): Extension<Schema<query::Query, EmptyMutation, EmptySubscription>>,
    query: Query<Request>,
) -> axum::response::Response {
    telemetry::http_request("/graphql", "GET");

    if query.query == "" {
        return Redirect::to("/graphiql").into_response();
    }

    Json(schema.execute(query.0).await).into_response()
}

async fn graphiql() -> impl IntoResponse {
    telemetry::http_request("/graphiql", "GET");

    Html(
        GraphiQLSource::build()
            .endpoint("/graphql")
            .title("Saerro Listening Post")
            .finish(),
    )
}

#[tokio::main]
async fn main() {
    let db_url = std::env::var("DATABASE_URL")
        .unwrap_or("postgres://saerrouser:saerro321@127.0.0.1:5432/data".to_string());
    let db = sqlx::PgPool::connect(&db_url).await.unwrap();

    let schema = Schema::build(query::Query::default(), EmptyMutation, EmptySubscription)
        .data(db.clone())
        .finish();

    let app = Router::new()
        .route("/", get(index))
        .route("/ingest", get(ingest))
        .route("/health", get(health::get_health))
        .route(
            "/graphql",
            post(graphql_handler_post).get(graphql_handler_get),
        )
        .route("/graphiql", get(graphiql))
        .route("/metrics", get(telemetry::handler))
        .route("/metrics/combined", get(telemetry::handler_combined))
        .fallback(handle_404)
        .layer(Extension(db))
        .layer(Extension(schema))
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_headers([CONTENT_TYPE])
                .allow_methods([Method::GET, Method::POST, Method::OPTIONS]),
        );

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
