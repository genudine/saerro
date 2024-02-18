use lazy_static::lazy_static;
use prometheus::{ 
  IntGaugeVec,
  register_int_gauge_vec,
  TextEncoder,
  gather
};

lazy_static! {
  // incoming events
  pub static ref EVENTS: IntGaugeVec = register_int_gauge_vec!("saerro_ws_events_count", "Events processed", &[
    "world_id", "event_name"
  ]).unwrap();
  pub static ref EVENTS_DROPPED: IntGaugeVec = register_int_gauge_vec!("saerro_ws_events_dropped_count", "Events dropped", &[
    "world_id", "event_name", "reason"
  ]).unwrap();

  pub static ref EXPERIENCE_EVENTS: IntGaugeVec = register_int_gauge_vec!("saerro_ws_experience_events_count", "Experience Events processed by Exp ID", &[
    "world_id", "experience_id"
  ]).unwrap();

  // database stuff
  pub static ref DB_WRITES: IntGaugeVec = register_int_gauge_vec!("saerro_ws_db_writes", "Writes to Postgres", &[
    "table", "op"
  ]).unwrap();
  pub static ref DB_READS: IntGaugeVec = register_int_gauge_vec!("saerro_ws_db_reads", "Reads from Postgres", &[
    "table", "op"
  ]).unwrap();
  // static ref DB_WTIME: HistogramVec = register_histogram_vec!("saerro_ws_db_write_time", &[
  //   "table", "op"
  // ]).unwrap();
  // static ref DB_RTIME: HistogramVec = register_histogram_vec!("saerro_ws_db_read_time", &[
  //   "table", "op"
  // ]).unwrap();
}

pub async fn handler() -> String {
  let encoder = TextEncoder::new();
  let mut buffer = String::new();

  let metrics = gather();
  encoder.encode_utf8(&metrics, &mut buffer).expect("prometheus metrics failed to render");

  buffer
}

pub fn event(world_id: &i32, event_name: &String) {
  EVENTS.with_label_values(&[
    &world_id.to_string(),
    &event_name,
  ]).inc();
}

pub fn event_dropped(world_id: &i32, event_name: &String, reason: &str) {
  EVENTS_DROPPED.with_label_values(&[
    &world_id.to_string(),
    &event_name,
    reason,
  ]).inc();
}

pub fn experience_event(world_id: &i32, experience_id: &i32) {
  EXPERIENCE_EVENTS.with_label_values(&[
    &world_id.to_string(),
    &experience_id.to_string(),
  ]).inc();
}

pub fn db_write(table: &str, op: &str) {
  DB_WRITES.with_label_values(&[table, op]).inc();
}

pub fn db_read(table: &str, op: &str) {
  DB_READS.with_label_values(&[table, op]).inc();
}