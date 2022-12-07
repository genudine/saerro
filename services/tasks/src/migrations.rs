use crate::PG;
use sqlx::query;

pub async fn cmd_migrate() {
    println!("Migrating database...");

    tokio::join!(migrate_players(), migrate_classes(), migrate_vehicles(),);
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
        time TIMESTAMPTZ NOT NULL,
        character_id TEXT NOT NULL,
        world_id INT NOT NULL,
        faction_id INT NOT NULL,
        zone_id INT NOT NULL);",
    )
    .execute(pool)
    .await
    .unwrap();

    println!("PLAYERS => create_hypertable");
    query(
        "SELECT create_hypertable('players', 'time',
            chunk_time_interval => INTERVAL '1 minute', if_not_exists => TRUE);",
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

async fn migrate_classes() {
    let pool = PG.get().await;

    println!("-> Migrating classes");

    println!("CLASSES => DROP TABLE IF EXISTS classes");
    query("DROP TABLE IF EXISTS classes")
        .execute(pool)
        .await
        .unwrap();

    println!("CLASSES => CREATE TABLE classes");
    query(
        "CREATE TABLE classes (
        time TIMESTAMPTZ NOT NULL,
        character_id TEXT NOT NULL,
        world_id INT NOT NULL,
        faction_id INT NOT NULL,
        zone_id INT NOT NULL, 
        class_id TEXT NOT NULL);",
    )
    .execute(pool)
    .await
    .unwrap();

    println!("CLASSES => create_hypertable");
    query(
        "SELECT create_hypertable('classes', 'time', 
            chunk_time_interval => INTERVAL '1 minute', if_not_exists => TRUE);",
    )
    .execute(pool)
    .await
    .unwrap();

    println!("CLASSES => add_retention_policy");
    query("SELECT add_retention_policy('classes', INTERVAL '15 minutes');")
        .execute(pool)
        .await
        .unwrap();

    println!("CLASSES => done!");
}

async fn migrate_vehicles() {
    let pool = PG.get().await;

    println!("-> Migrating vehicles");

    println!("VEHICLES => DROP TABLE IF EXISTS vehicles");
    query("DROP TABLE IF EXISTS vehicles")
        .execute(pool)
        .await
        .unwrap();

    println!("VEHICLES => CREATE TABLE vehicles");
    query(
        "CREATE TABLE vehicles (
        time TIMESTAMPTZ NOT NULL,
        character_id TEXT NOT NULL,
        world_id INT NOT NULL,
        faction_id INT NOT NULL,
        zone_id INT NOT NULL, 
        vehicle_id TEXT NOT NULL);",
    )
    .execute(pool)
    .await
    .unwrap();

    println!("VEHICLES => create_hypertable");
    query(
        "SELECT create_hypertable('vehicles', 'time', 
            chunk_time_interval => INTERVAL '1 minute', if_not_exists => TRUE);",
    )
    .execute(pool)
    .await
    .unwrap();

    println!("VEHICLES => add_retention_policy");

    query("SELECT add_retention_policy('vehicles', INTERVAL '15 minutes');")
        .execute(pool)
        .await
        .unwrap();

    println!("VEHICLES => done!");
}
