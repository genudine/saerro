use once_cell::sync::Lazy;
use redis::Commands;
use std::env::args;
use std::ops::Sub;
use std::time::{Duration, SystemTime};

pub static REDIS_CLIENT: Lazy<redis::Client> = Lazy::new(|| {
    redis::Client::open(std::env::var("REDIS_ADDR").unwrap_or("redis://localhost:6379".to_string()))
        .unwrap()
});

fn cmd_prune() {
    println!("Pruning old data...");
    let mut con = REDIS_CLIENT.get_connection().unwrap();

    let prune_after = SystemTime::now()
        .sub(Duration::from_secs(60))
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let keys: Vec<String> = con.keys("wp:*").unwrap();
    for key in keys {
        println!("-> Pruning {}", key);
        let removed_items: u64 = con.zrembyscore(key, 0, prune_after).unwrap();
        println!("==> Removed {} items", removed_items);
    }
}

fn cmd_help() {
    println!("Usage: {} [command]", args().nth(0).unwrap());
    println!("Commands:");
    println!("  help - Show this help message");
    println!("  prune - Remove stale data from Redis");
}

fn main() {
    let command = args().nth(1).unwrap_or("help".to_string());

    match command.as_str() {
        "help" => cmd_help(),
        "prune" => cmd_prune(),
        _ => {
            println!("Unknown command: {}", command);
            cmd_help();
        }
    }
}
