use crate::PG;
use sqlx::{query, Row};

pub async fn cmd_migrate() {
    println!("Migrating database...");

    tokio::join!(migrate_players(), migrate_analytics());
}

async fn migrate_players() {
    let pool = PG.get().await;

    println!("-> Migrating players");

    println!("PLAYERS => DROP TABLE IF EXISTS players");
    query("DROP TABLE IF EXISTS players")
        .execute(pool)
        .await
        .unwrap();

    println!("PLAYERS => CREATE TABLE players");
    query(
        "CREATE TABLE players (
        character_id TEXT NOT NULL,
        time TIMESTAMPTZ NOT NULL,
        world_id INT NOT NULL,
        faction_id INT NOT NULL,
        zone_id INT NOT NULL,
        class_id TEXT NOT NULL,
        vehicle_id TEXT NOT NULL
        );",
    )
    .execute(pool)
    .await
    .unwrap();

    println!("PLAYERS => create_hypertable");
    query(
        "SELECT create_hypertable('players', 'time',
            chunk_time_interval => INTERVAL '5 minutes', if_not_exists => TRUE);",
    )
    .execute(pool)
    .await
    .unwrap();

    println!("PLAYERS => add_retention_policy");
    query("SELECT add_retention_policy('players', INTERVAL '15 minutes');")
        .execute(pool)
        .await
        .unwrap();

    println!("PLAYERS => done!");
}

async fn migrate_analytics() {
    let pool = PG.get().await;

    println!("-> Migrating analytics");
    println!("ANALYTICS => CREATE TABLE IF NOT EXISTS analytics");
    query(
        "CREATE TABLE IF NOT EXISTS analytics (
        time TIMESTAMPTZ NOT NULL,
        event_name TEXT NOT NULL,
        world_id INT NOT NULL);",
    )
    .execute(pool)
    .await
    .unwrap();

    println!("ANALYTICS => create_hypertable");
    query(
        "SELECT create_hypertable('analytics', 'time', 
            chunk_time_interval => INTERVAL '1 hour', if_not_exists => TRUE);",
    )
    .execute(pool)
    .await
    .unwrap();

    println!("ANALYTICS => add_retention_policy");
    query("SELECT add_retention_policy('analytics', INTERVAL '1 day', if_not_exists => TRUE);")
        .execute(pool)
        .await
        .unwrap();

    println!("ANALYTICS => done!");
}

pub async fn is_migrated() -> bool {
    let pool = PG.get().await;

    let tables: i64 = query("SELECT count(1) FROM pg_tables WHERE schemaname = 'public' AND tablename IN ('players', 'analytics');")
        .fetch_one(pool)
        .await
        .unwrap()
        .get(0);

    tables == 2
}
