/*
 * AKASA Sentinel - A high-performance, self-sovereign Monero payment gateway built for privacy and speed.
 * Copyright (C) 2026 Siptruk
 * Licensed under the GNU General Public License v3.0
 */

use tokio::net::TcpListener;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use serde_json::{json, Value};
use chrono::{Local, DateTime};
use std::collections::HashMap;
use std::net::IpAddr;
use std::sync::Arc;
use tokio::sync::Mutex;
use std::time::{Duration, Instant};
use sqlx::{Pool, Sqlite};
use tracing::{info, warn};

use crate::network::MoneroClient;

pub struct MgmtServer {
    pub start_time: DateTime<Local>,
    pub version: String,
    pub client: MoneroClient,
    pub db_pool: Pool<Sqlite>,
    pub api_key: String,
    rate_limiter: Arc<Mutex<HashMap<IpAddr, (u32, Instant)>>>,
}

impl MgmtServer {
    pub fn new(version: String, client: MoneroClient, db_pool: Pool<Sqlite>) -> Self {
        let api_key = std::env::var("MGMT_API_KEY").unwrap_or_else(|_| "akasa_default_secret".to_string());
        Self {
            start_time: Local::now(),
            version, client, db_pool, api_key,
            rate_limiter: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn run(self, addr: String) {
        let listener = TcpListener::bind(&addr).await.expect("Failed to bind management port");
        info!("🛡️  Management API v{} active at {}", self.version, addr);

        loop {
            if let Ok((mut socket, addr)) = listener.accept().await {
                let ip = addr.ip();
                
                let mut limiter = self.rate_limiter.lock().await;
                let (count, last_reset) = limiter.entry(ip).or_insert((0, Instant::now()));
                if Instant::now().duration_since(*last_reset) > Duration::from_secs(10) {
                    *count = 1; *last_reset = Instant::now();
                } else { *count += 1; }

                if *count > 15 {
                    warn!("[SECURITY] Rate limit hit: {}", ip);
                    let _ = socket.write_all(b"HTTP/1.1 429 Too Many Requests\r\n\r\n").await;
                    continue;
                }
                drop(limiter);

                let mut buffer = [0; 4096];
                let n = socket.read(&mut buffer).await.unwrap_or(0);
                let request = String::from_utf8_lossy(&buffer[..n]);

                if request.contains("POST /v1/subaddress/create") {
                    let expected_key = format!("X-API-Key: {}", self.api_key);
                    if !request.contains(&expected_key) {
                        warn!("[AUTH ERROR] Unauthorized attempt from {}", ip);
                        let _ = socket.write_all(b"HTTP/1.1 401 Unauthorized\r\n\r\n").await;
                        continue;
                    }
                    self.handle_create_subaddress(&mut socket, &request).await;
                } else if request.contains("GET /v1/health") {
                    self.handle_health(&mut socket).await;
                } else {
                    self.handle_status(&mut socket).await;
                }
                
                let _ = socket.shutdown().await;
            }
        }
    }

    async fn handle_create_subaddress(&self, socket: &mut tokio::net::TcpStream, request: &str) {
        let body = request.split("\r\n\r\n").last().unwrap_or("");
        let json_body: Value = serde_json::from_str(body).unwrap_or(json!({}));
        let label = json_body["label"].as_str().unwrap_or("api_call");

        match self.client.create_subaddress(label).await {
            Ok(result) => {
                let address = result["address"].as_str().unwrap_or("");
                let index = result["address_index"].as_u64().unwrap_or(0);
                let _ = sqlx::query("INSERT INTO wallet_subaddresses (address_index, address, label) VALUES (?, ?, ?)")
                    .bind(index as i64).bind(address).bind(label).execute(&self.db_pool).await;
                
                send_json_response(socket, 200, json!({"status": "success", "address": address, "index": index, "label": label})).await;
            },
            Err(e) => send_json_response(socket, 500, json!({"error": e.to_string()})).await,
        }
    }

    async fn handle_health(&self, socket: &mut tokio::net::TcpStream) {
        send_json_response(socket, 200, json!({"status": "ok", "timestamp": Local::now().to_rfc3339()})).await;
    }

    async fn handle_status(&self, socket: &mut tokio::net::TcpStream) {
        send_json_response(socket, 200, json!({
            "status": "online", 
            "engine": "AKASA Sentinel", 
            "version": self.version, 
            "uptime": (Local::now() - self.start_time).num_seconds()
        })).await;
    }
}

async fn send_json_response(socket: &mut tokio::net::TcpStream, code: u16, data: Value) {
    let response = format!(
        "HTTP/1.1 {} OK\r\nContent-Type: application/json\r\nConnection: close\r\n\r\n{}\n",
        code, data.to_string()
    );
    let _ = socket.write_all(response.as_bytes()).await;
}