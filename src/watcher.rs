/*
 * AKASA Sentinel - A high-performance, self-sovereign Monero payment gateway built for privacy and speed.
 * Copyright (C) 2026 Siptruk
 * Licensed under the GNU General Public License v3.0
 */

use crate::crypto::MerchantWallet;
use crate::network::MoneroClient;
use hmac::{Hmac, Mac};
use sha2::Sha256;
use sqlx::{Pool, Sqlite, Row};
use std::time::Duration;
use tokio::time::{sleep, Instant};
use tokio::sync::mpsc; 
use serde_json::{json, Value};
use chrono::Local;
use monero::Transaction;
use tracing::{info, warn, error};

type HmacSha256 = Hmac<Sha256>;

struct WebhookJob {
    wallet: MerchantWallet,
    payload: Value,
}

pub struct GhostWatcher {
    wallets: Vec<MerchantWallet>,
    client: MoneroClient,
    db_pool: Option<Pool<Sqlite>>,
    last_scanned_height: u64,
    scan_interval: u64,
    last_heartbeat: Instant,
    webhook_tx: mpsc::Sender<WebhookJob>,
}

impl GhostWatcher {
    pub fn new(
        wallets: Vec<MerchantWallet>, 
        client: MoneroClient, 
        db_pool: Option<Pool<Sqlite>>
    ) -> Self {
        let scan_interval = std::env::var("SCAN_INTERVAL").unwrap_or_else(|_| "20".to_string()).parse::<u64>().unwrap_or(20);
        let max_retries = std::env::var("WEBHOOK_MAX_RETRIES").unwrap_or_else(|_| "10".to_string()).parse::<u32>().unwrap_or(10);
        let retry_delay = std::env::var("WEBHOOK_RETRY_DELAY").unwrap_or_else(|_| "60".to_string()).parse::<u64>().unwrap_or(60);

        let (tx, rx) = mpsc::channel(100);

        Self::spawn_webhook_worker(rx, max_retries, retry_delay);

        Self { 
            wallets, 
            client, 
            db_pool, 
            last_scanned_height: 0,
            scan_interval,
            last_heartbeat: Instant::now(),
            webhook_tx: tx,
        }
    }

    fn spawn_webhook_worker(mut rx: mpsc::Receiver<WebhookJob>, max_retries: u32, retry_delay: u64) {
        tokio::spawn(async move {
            info!("🚀 Webhook Background Worker: Active");
            while let Some(job) = rx.recv().await {
                let mut attempts = 0;
                let hash = job.payload["hash"].as_str().unwrap_or("heartbeat");

                while attempts < max_retries {
                    if Self::execute_http_post(&job.wallet, &job.payload).await.is_ok() {
                        info!("✅ Webhook Successful: [{}] {}", job.wallet.label, hash);
                        break;
                    }
                    attempts += 1;
                    if attempts < max_retries {
                        warn!("⚠️ Webhook Failed [{}] ({} / {}), retrying in {}s...", job.wallet.label, attempts, max_retries, retry_delay);
                        sleep(Duration::from_secs(retry_delay)).await;
                    } else {
                        error!("❌ Max retries reached for webhook [{}] {}", job.wallet.label, hash);
                    }
                }
            }
        });
    }

    async fn execute_http_post(wallet: &MerchantWallet, payload: &Value) -> Result<(), reqwest::Error> {
        let client = reqwest::Client::new();
        let mut mac = HmacSha256::new_from_slice(wallet.webhook_secret.as_bytes()).expect("HMAC Error");
        mac.update(payload.to_string().as_bytes());
        let signature = hex::encode(mac.finalize().into_bytes());

        client.post(&wallet.webhook_url)
            .header("X-Akasa-Signature", signature)
            .json(payload)
            .send()
            .await?
            .error_for_status()?;
            
        Ok(())
    }

    pub async fn run(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if let Ok(height) = self.client.get_current_height().await {
            self.last_scanned_height = height;
        }

        loop {
            if self.last_heartbeat.elapsed() >= Duration::from_secs(600) {
                self.send_heartbeat().await;
                self.last_heartbeat = Instant::now();
            }

            if let Ok(hashes) = self.client.get_mempool().await {
                for hash in hashes {
                    let _ = self.process_transaction(&hash, "unconfirmed", None).await;
                }
            }

            if let Ok(current_height) = self.client.get_current_height().await {
                if current_height > self.last_scanned_height {
                    info!("⛓️ New Block Detected: #{}", current_height);
                    self.check_pending_confirmations(current_height).await;
                    self.last_scanned_height = current_height;
                }
            }

            sleep(Duration::from_secs(self.scan_interval)).await;
        }
    }

    async fn send_heartbeat(&self) {
        let payload = json!({
            "event": "heartbeat", "status": "active",
            "block_height": self.last_scanned_height, "timestamp": Local::now().to_rfc3339()
        });

        for wallet in &self.wallets {
            let _ = self.webhook_tx.send(WebhookJob { wallet: wallet.clone(), payload: payload.clone() }).await;
        }
    }

    async fn check_pending_confirmations(&self, current_height: u64) {
        if let Some(pool) = &self.db_pool {
            let result = sqlx::query("SELECT tx_hash, block_height, wallet_label, confirms_req FROM processed_txs WHERE status = 'pending'")
                .fetch_all(pool).await;

            if let Ok(rows) = result {
                for row in rows {
                    let tx_hash: String = row.get("tx_hash");
                    let h_tx: i64 = row.get("block_height");
                    let wallet_label: String = row.get("wallet_label");
                    let conf_req: i64 = row.get("confirms_req");

                    let conf_now = current_height.saturating_sub(h_tx as u64) + 1;

                    if conf_now >= conf_req as u64 {
                        if let Some(wallet) = self.wallets.iter().find(|w| w.label == wallet_label) {
                            let _ = self.update_and_notify_final(wallet, &tx_hash, conf_now).await;
                        }
                    }
                }
            }
        }
    }

    async fn process_transaction(&self, tx_hash: &str, status: &str, tx_height: Option<u64>) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let tx_data = match self.client.get_tx_data(tx_hash).await {
            Ok(data) => data,
            Err(_) => return Ok(()),
        };

        let tx_hex = tx_data["hex"].as_str().unwrap_or("");
        let tx_bytes = hex::decode(tx_hex).unwrap_or_default();
        let monero_tx: Option<Transaction> = monero::consensus::deserialize(&tx_bytes).ok();

        if let Some(tx) = monero_tx {
            for wallet in &self.wallets {
                if let Some(amount_pico) = wallet.scan_transaction(&tx) {
                    info!(wallet = %wallet.label, hash = %tx_hash, "Payment matched");
                    self.handle_merchant_payment(wallet, tx_hash, amount_pico, status, tx_height).await?;
                }
            }
        }
        Ok(())
    }

    async fn handle_merchant_payment(&self, wallet: &MerchantWallet, hash: &str, amt: u64, status: &str, tx_h: Option<u64>) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let required_conf = wallet.confirmations.unwrap_or(10);
        
        if let Some(pool) = &self.db_pool {
            let _ = sqlx::query(
                "INSERT OR IGNORE INTO processed_txs (tx_hash, wallet_label, amount_xmr, status, confirms_req, block_height) VALUES (?, ?, ?, ?, ?, ?)"
            )
            .bind(hash).bind(&wallet.label).bind(amt as f64 / 1e12).bind(if status == "unconfirmed" { "pending" } else { "confirmed" })
            .bind(required_conf as i64).bind(tx_h.map(|h| h as i64)).execute(pool).await;
        }

        let payload = json!({
            "event": if status == "unconfirmed" { "tx_mempool" } else { "tx_confirmed" },
            "wallet": wallet.label, "address": wallet.get_address(),
            "amount": amt as f64 / 1e12, "hash": hash,
            "confirmations_required": required_conf, "timestamp": Local::now().to_rfc3339()
        });

        let _ = self.webhook_tx.send(WebhookJob { wallet: wallet.clone(), payload }).await;
        Ok(())
    }

    async fn update_and_notify_final(&self, wallet: &MerchantWallet, hash: &str, conf: u64) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if let Some(pool) = &self.db_pool {
            sqlx::query("UPDATE processed_txs SET status = 'confirmed', confirms_now = ?, webhook_sent = 1 WHERE tx_hash = ?")
                .bind(conf as i64).bind(hash).execute(pool).await?;
        }

        info!("🏁 Transaction Confirmed: {} ({} depth)", hash, conf);

        let payload = json!({
            "event": "tx_confirmed", "wallet": wallet.label,
            "address": wallet.get_address(), "amount": 0.0,
            "hash": hash, "confirmations": conf, "timestamp": Local::now().to_rfc3339()
        });

        let _ = self.webhook_tx.send(WebhookJob { wallet: wallet.clone(), payload }).await;
        Ok(())
    }
}