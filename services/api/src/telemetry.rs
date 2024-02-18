use lazy_static::lazy_static;
use prometheus::{ 
  IntGauge, 
  IntGaugeVec,
  register_int_gauge_vec,
  register_int_gauge,
  TextEncoder,
  gather
};
use sqlx::{Pool, Postgres, Row};
use axum::Extension;
use chrono::{DateTime, Utc};

lazy_static! {
  // http
  pub static ref HTTP_REQUEST: IntGaugeVec = register_int_gauge_vec!("saerro_api_http_requests", "HTTP requests", &[
    "route", "method"
  ]).unwrap();
  pub static ref GRAPHQL_QUERY: IntGaugeVec = register_int_gauge_vec!("saerro_api_graphql_query", "GraphQL queries", &[
    "major", "minor"
  ]).unwrap();

  // counters
  pub static ref PLAYERS_TRACKED: IntGauge = register_int_gauge!("saerro_players_tracked", "All players tracked by Saerro right now").unwrap();
  pub static ref VEHICLES_TRACKED: IntGauge = register_int_gauge!("saerro_vehicles_tracked", "All vehicles tracked by Saerro right now").unwrap();
  pub static ref OLDEST_PLAYER: IntGauge = register_int_gauge!("saerro_oldest_player", "Oldest player tracked").unwrap();
  pub static ref NEWEST_PLAYER: IntGauge = register_int_gauge!("saerro_newest_player", "Newest player tracked").unwrap();
  pub static ref OLDEST_VEHICLE: IntGauge = register_int_gauge!("saerro_oldest_vehicle", "Oldest vehicle tracked").unwrap();
  pub static ref NEWEST_VEHICLE: IntGauge = register_int_gauge!("saerro_newest_vehicle", "Newest vehicle tracked").unwrap();

  // database stuff
  pub static ref DB_WRITES: IntGaugeVec = register_int_gauge_vec!("saerro_api_db_writes", "Writes to Postgres", &[
    "table", "op"
  ]).unwrap();
  pub static ref DB_READS: IntGaugeVec = register_int_gauge_vec!("saerro_api_db_reads", "Reads from Postgres", &[
    "table", "op"
  ]).unwrap();
  // static ref DB_WTIME: HistogramVec = register_histogram_vec!("saerro_ws_db_write_time", &[
  //   "table", "op"
  // ]).unwrap();
  // static ref DB_RTIME: HistogramVec = register_histogram_vec!("saerro_ws_db_read_time", &[
  //   "table", "op"
  // ]).unwrap();
}

pub async fn handler(Extension(pool): Extension<Pool<Postgres>>) -> String { 
  update_data_gauges(pool).await;

  // Final output
  let encoder = TextEncoder::new();
  let mut buffer = String::new();

  let metrics = gather();
  encoder.encode_utf8(&metrics, &mut buffer).expect("prometheus metrics failed to render");

  buffer
}

pub async fn handler_combined(Extension(pool): Extension<Pool<Postgres>>) -> String {
  let url = std::env::var("WEBSOCKET_HEALTHCHECK")
    .unwrap_or("http://127.0.0.1:8999/healthz".to_string()).replace("/healthz", "/metrics");
  
  let local = handler(Extension(pool)).await;
  let remote = match reqwest::get(url).await {
    Ok(r) => r.text().await.expect("failed to text lol"),
    Err(_) => String::from("")
  };


  format!("{}{}", local, remote)
}

// pub fn db_write(table: &str, op: &str) {
//   DB_WRITES.with_label_values(&[table, op]).inc();
// }

pub fn db_read(table: &str, op: &str) {
  DB_READS.with_label_values(&[table, op]).inc();
}

pub fn http_request(route: &str, method: &str) {
  HTTP_REQUEST.with_label_values(&[route, method]).inc();
}

pub fn graphql_query(major: &str, minor: &str) {
  GRAPHQL_QUERY.with_label_values(&[major, minor]).inc();
}

async fn update_data_gauges(pool: Pool<Postgres>) {
  // Do some easy queries to fill our non-cumulative gauges
  db_read("players", "count_all");
  let player_count: i64 = sqlx::query("SELECT count(*) FROM players")
    .fetch_one(&pool)
    .await
    .unwrap()
    .get(0);
  PLAYERS_TRACKED.set(player_count);

  db_read("players", "get_newest");
  let player_newest: DateTime<Utc> = sqlx::query("SELECT last_updated FROM players ORDER BY last_updated DESC LIMIT 1")
    .fetch_one(&pool)
    .await
    .unwrap()
    .get(0);
  NEWEST_PLAYER.set(player_newest.timestamp());

  db_read("players", "get_oldest");
  let player_oldest: DateTime<Utc> = sqlx::query("SELECT last_updated FROM players ORDER BY last_updated ASC LIMIT 1")
    .fetch_one(&pool)
    .await
    .unwrap()
    .get(0);
  OLDEST_PLAYER.set(player_oldest.timestamp());

  db_read("vehicles", "count_all");
  let vehicle_count: i64 = sqlx::query("SELECT count(*) FROM vehicles")
    .fetch_one(&pool)
    .await
    .unwrap()
    .get(0);
  VEHICLES_TRACKED.set(vehicle_count);

  db_read("vehicles", "get_newest");
  let vehicle_newest: DateTime<Utc> = sqlx::query("SELECT last_updated FROM vehicles ORDER BY last_updated DESC LIMIT 1")
    .fetch_one(&pool)
    .await
    .unwrap()
    .get(0);
  NEWEST_VEHICLE.set(vehicle_newest.timestamp());

  db_read("vehicles", "get_oldest");
  let vehicle_oldest: DateTime<Utc> = sqlx::query("SELECT last_updated FROM vehicles ORDER BY last_updated ASC LIMIT 1")
  .fetch_one(&pool)
  .await
  .unwrap()
  .get(0);
  OLDEST_VEHICLE.set(vehicle_oldest.timestamp());
}