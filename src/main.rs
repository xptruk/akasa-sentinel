/*
 * AKASA Sentinel - A high-performance, self-sovereign Monero payment gateway built for privacy and speed.
 * Copyright (C) 2026 Siptruk
 * Licensed under the GNU General Public License v3.0
 */

mod crypto;
mod network;
mod watcher;
mod db;
mod mgmt;

use crate::crypto::MerchantWallet;
use crate::network::MoneroClient;
use crate::watcher::GhostWatcher;
use crate::mgmt::MgmtServer;
use monero::Network;
use sqlx::{Pool, Sqlite};
use std::{fs, process};
use tracing::{info, error, warn};

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .with_target(false)
        .init();

    let display_name = get_display_name();
    let version = env!("CARGO_PKG_VERSION");

    info!("--- {} v{} ---", display_name, version);

    let network_str = std::env::var("MONERO_NETWORK").unwrap_or_else(|_| "mainnet".to_string());
    let network = match network_str.to_lowercase().as_str() {
        "stagenet" => Network::Stagenet,
        "testnet" => Network::Testnet,
        _ => Network::Mainnet,
    };

    info!("Network Mode: {:?}", network);

    let db_mode = std::env::var("DATABASE_MODE").unwrap_or_else(|_| "standalone".to_string());
    let mut db_pool: Option<Pool<Sqlite>> = None;

    info!("Operation Mode: {}", db_mode);

    if db_mode == "integrated" {
        let db_url = std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| "sqlite://akasa_sentinel.db".to_string());
            
        match db::init_db(&db_url).await {
            Ok(pool) => {
                info!("Database: Connected ✅ -> {}", db_url);
                db_pool = Some(pool);
            }
            Err(e) => {
                error!("\n❌ DATABASE ERROR: {}", e);
                process::exit(1);
            }
        }
    } else {
        info!("Database: Standalone (RAM-only/Passive) 🟢");
    }

    let node_timeout = std::env::var("NODE_TIMEOUT")
        .unwrap_or_else(|_| "15".to_string())
        .parse::<u64>()
        .unwrap_or(15);

    let node_urls_raw = std::env::var("REMOTE_NODE_URLS").expect("REMOTE_NODE_URLS not set");
    let node_urls: Vec<String> = node_urls_raw
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();

    let monero_client = MoneroClient::new(node_urls, node_timeout);

    info!("Connecting to Monero Network (Timeout: {}s)...", node_timeout);
    match monero_client.check_node_health().await {
        Ok(true) => info!("Node Connection: Online 🛰️"),
        _ => {
            error!("❌ ERROR: Initial node health check failed.");
            process::exit(1);
        }
    }

    if let Some(pool) = db_pool.clone() {
        let mgmt_addr = std::env::var("MGMT_ADDR").unwrap_or_else(|_| "127.0.0.1:9090".to_string());
        let mgmt_server = MgmtServer::new(
            version.to_string(),
            monero_client.clone(),
            pool
        );

        tokio::spawn(async move {
            mgmt_server.run(mgmt_addr).await;
        });
    } else {
        warn!("⚠️  Management API is disabled (Requires integrated mode).");
    }

    let json_data = fs::read_to_string("wallets.json").unwrap_or_else(|_| {
        error!("\n❌ ERROR: wallets.json not found.");
        process::exit(1);
    });

    let mut wallets: Vec<MerchantWallet> = serde_json::from_str(&json_data)
        .expect("Failed to parse wallets.json");

    for wallet in &mut wallets {
        if let Err(e) = wallet.init(network) {
            error!("❌ WALLET INIT ERROR: {}", e);
            process::exit(1);
        }
        info!("Watching Wallet: {} [{}]", wallet.label, wallet.get_address());
    }

    let mut watcher = GhostWatcher::new(wallets, monero_client, db_pool);

    info!("{} is now watching for payments... 🛡️", display_name);

    tokio::select! {
        _ = watcher.run() => {
            warn!("Watcher stopped unexpectedly.");
        }
        _ = tokio::signal::ctrl_c() => {
            info!("\n🛑 Shutdown signal received (Ctrl+C).");
            info!("Closing database connections and cleaning up...");
            info!("AKASA Sentinel offline. Goodbye! 👋");
            process::exit(0);
        }
    }
}

fn get_display_name() -> String {
    let pkg_name = env!("CARGO_PKG_NAME");

    pkg_name
        .replace("-", " ")
        .split_whitespace()
        .map(|word| {
            if word == "akasa" {
                "AKASA".to_string()
            } else {
                let mut c = word.chars();
                match c.next() {
                    None => String::new(),
                    Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
                }
            }
        })
        .collect::<Vec<String>>()
        .join(" ")
}