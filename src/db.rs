/*
 * AKASA Sentinel - A high-performance, self-sovereign Monero payment gateway built for privacy and speed.
 * Copyright (C) 2026 Siptruk
 * Licensed under the GNU General Public License v3.0
 */

use sqlx::{Pool, Sqlite, sqlite::{SqliteConnectOptions, SqlitePool}};
use std::str::FromStr;
use std::time::Duration;
use tokio::time;
use tracing::{info};

pub async fn init_db(url: &str) -> Result<Pool<Sqlite>, Box<dyn std::error::Error + Send + Sync>> {
    let options = SqliteConnectOptions::from_str(url)?.create_if_missing(true);
    let pool = SqlitePool::connect_with(options).await?;
    
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS processed_txs (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            tx_hash TEXT UNIQUE,
            wallet_label TEXT,
            amount_xmr REAL,
            status TEXT,
            confirms_now INTEGER,
            confirms_req INTEGER,
            block_height INTEGER,
            webhook_sent BOOLEAN DEFAULT 0,
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
        )"
    ).execute(&pool).await?;

    sqlx::query("CREATE INDEX IF NOT EXISTS idx_tx_status ON processed_txs (status);").execute(&pool).await?;
    sqlx::query("CREATE INDEX IF NOT EXISTS idx_tx_height ON processed_txs (block_height);").execute(&pool).await?;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS wallet_subaddresses (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            address_index INTEGER UNIQUE,
            address TEXT UNIQUE,
            label TEXT UNIQUE,
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
        )"
    ).execute(&pool).await?;
    
    sqlx::query("CREATE INDEX IF NOT EXISTS idx_sub_address ON wallet_subaddresses (address);").execute(&pool).await?;

    let pruning_enabled = std::env::var("DB_PRUNING_ENABLED").unwrap_or_else(|_| "false".to_string()) == "true";
    if pruning_enabled {
        let pool_clone = pool.clone();
        tokio::spawn(async move { run_pruning_task(pool_clone).await; });
    }
    
    Ok(pool)
}

async fn run_pruning_task(pool: Pool<Sqlite>) {
    let pruning_days = std::env::var("DB_PRUNING_DAYS").unwrap_or_else(|_| "30".to_string()).parse::<i32>().unwrap_or(30);
    let interval_hours = std::env::var("DB_PRUNING_INTERVAL_HOURS").unwrap_or_else(|_| "24".to_string()).parse::<u64>().unwrap_or(24);
    let mut interval = time::interval(Duration::from_secs(interval_hours * 3600));

    info!("🧹 DB Pruning: Active (Retention: {} days)", pruning_days);

    loop {
        interval.tick().await;
        let _ = sqlx::query("DELETE FROM processed_txs WHERE created_at < date('now', ?)")
            .bind(format!("-{} days", pruning_days)).execute(&pool).await;
    }
}