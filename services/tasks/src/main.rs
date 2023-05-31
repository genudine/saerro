use async_once::AsyncOnce;
use lazy_static::lazy_static;
use migrations::cmd_migrate;
use sqlx::query;
use std::env::args;

mod migrations;

lazy_static! {
    pub static ref PG: AsyncOnce<sqlx::PgPool> = AsyncOnce::new(async {
        let db_url = std::env::var("DATABASE_URL")
            .unwrap_or("postgres://saerrouser:saerro321@localhost:5432/data".to_string());
        sqlx::PgPool::connect(&db_url).await.unwrap()
    });
}

async fn cmd_prune() {
    println!("Pruning old data...");
    let pool = PG.get().await;

    let rows = query("DELETE FROM players WHERE time < NOW() - INTERVAL '15 minutes';")
        .execute(pool)
        .await
        .unwrap()
        .rows_affected();
    println!("Deleted {} rows of old player data", rows);

    let rows = query("DELETE FROM analytics WHERE time < NOW() - INTERVAL '1 day';")
        .execute(pool)
        .await
        .unwrap()
        .rows_affected();
    println!("Deleted {} rows of old analytics data", rows);
}

fn cmd_help() {
    println!("Usage: {} [command]", args().nth(0).unwrap());
    println!("Commands:");
    println!("  help - Show this help message");
    println!("  prune - Remove stale data from Redis");
    println!("  migrate - Reset and create database tables");
}

#[tokio::main]
async fn main() {
    let command = args().nth(1).unwrap_or("help".to_string());

    match command.as_str() {
        "help" => cmd_help(),
        "prune" => cmd_prune().await,
        "auto-prune" => loop {
            cmd_prune().await;
            tokio::time::sleep(tokio::time::Duration::from_secs(60 * 5)).await;
        },
        "maintenance" => {
            println!("Running maintenance tasks...");
            println!("Checking if DB is migrated...");
            if !migrations::is_migrated().await {
                println!("DB is not migrated, running migrations...");
                cmd_migrate().await;
            }

            println!("Running prune...");
            cmd_prune().await;
            println!("Done!");
        }
        "migrate" => cmd_migrate().await,
        _ => {
            println!("Unknown command: {}", command);
            cmd_help();
        }
    }
}
