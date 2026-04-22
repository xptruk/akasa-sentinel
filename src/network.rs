/*
 * AKASA Sentinel - A high-performance, self-sovereign Monero payment gateway built for privacy and speed.
 * Copyright (C) 2026 Siptruk
 * Licensed under the GNU General Public License v3.0
 */

use reqwest::Client;
use reqwest::header::{HeaderMap, HeaderValue, CONTENT_TYPE, USER_AGENT};
use serde_json::{json, Value};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tracing::{warn};

#[derive(Clone)]
pub struct MoneroClient {
    pub node_urls: Vec<String>,
    current_index: Arc<RwLock<usize>>,
    client: Client,
}

impl MoneroClient {
    
    pub fn new(node_urls: Vec<String>, timeout_secs: u64) -> Self {
        let sanitized_urls: Vec<String> = node_urls.into_iter().map(|url| {
            let trimmed = url.trim();
            let mut final_url = if !trimmed.starts_with("http://") && !trimmed.starts_with("https://") {
                format!("http://{}", trimmed)
            } else {
                trimmed.to_string()
            };

            if final_url.ends_with('/') {
                final_url.pop();
            }
            final_url
        }).collect();

        let name = env!("CARGO_PKG_NAME");
        let version = env!("CARGO_PKG_VERSION");
        let user_agent = format!("{}/{}", name, version);

        let mut headers = HeaderMap::new();
        headers.insert(USER_AGENT, HeaderValue::from_str(&user_agent).unwrap_or(HeaderValue::from_static("akasa-sentinel/1.0.0")));
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

        let mut client_builder = Client::builder()
            .timeout(Duration::from_secs(timeout_secs))
            .default_headers(headers);

        if let Ok(proxy_url) = std::env::var("SOCKS5_PROXY") {
            if !proxy_url.is_empty() {
                if let Ok(proxy) = reqwest::Proxy::all(&proxy_url) {
                    client_builder = client_builder.proxy(proxy);
                }
            }
        }

        Self {
            node_urls: sanitized_urls,
            current_index: Arc::new(RwLock::new(0)),
            client: client_builder.build().unwrap_or_else(|_| Client::new()),
        }
    }

    async fn rpc_call(&self, path: &str, params: Option<Value>) -> Result<Value, Box<dyn std::error::Error + Send + Sync>> {
        let idx = *self.current_index.read().await;
        let mut current_idx = idx;

        loop {
            let url = format!("{}{}", self.node_urls[current_idx], path);
            
            let request = if let Some(p) = &params {
                self.client.post(&url).json(p)
            } else {
                self.client.post(&url).body("{}")
            };

            match request.send().await {
                Ok(res) if res.status().is_success() => {
                    if let Ok(json_res) = res.json::<Value>().await {
                        return Ok(json_res);
                    }
                }
                _ => {
                    warn!("⚠️ Node {} is not connected, switching to another node...", self.node_urls[current_idx]);
                }
            }

            current_idx = (current_idx + 1) % self.node_urls.len();
            if current_idx == idx {
                return Err("❌ All nodes are unreachable".into());
            }

            let mut write_idx = self.current_index.write().await;
            *write_idx = current_idx;
        }
    }

    pub async fn get_current_height(&self) -> Result<u64, Box<dyn std::error::Error + Send + Sync>> {
        let res = self.rpc_call("/get_info", None).await?;
        Ok(res["height"].as_u64().unwrap_or(0))
    }

    pub async fn check_node_health(&self) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        match self.rpc_call("/get_info", None).await {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }

    pub async fn get_mempool(&self) -> Result<Vec<String>, Box<dyn std::error::Error + Send + Sync>> {
        let res = self.rpc_call("/get_transaction_pool", None).await?;
        let mut hashes = Vec::new();
        
        if let Some(transactions) = res["transactions"].as_array() {
            for tx in transactions {
                if let Some(hash) = tx["id_hash"].as_str() {
                    hashes.push(hash.to_string());
                }
            }
        }
        Ok(hashes)
    }

    pub async fn get_tx_data(&self, tx_hash: &str) -> Result<Value, Box<dyn std::error::Error + Send + Sync>> {
        let params = json!({
            "txs_hashes": [tx_hash],
            "decode_as_json": true
        });

        let res = self.rpc_call("/get_transactions", Some(params)).await?;
        
        if let Some(tx_list) = res["txs"].as_array() {
            if let Some(tx_data) = tx_list.get(0) {
                return Ok(tx_data.clone());
            }
        }
        
        Err(format!("Transaction {} not found", tx_hash).into())
    }

    pub async fn create_subaddress(&self, label: &str) -> Result<Value, Box<dyn std::error::Error + Send + Sync>> {
        let params = json!({
            "account_index": 0,
            "label": label
        });

        let res = self.rpc_call("/json_rpc", Some(json!({
            "jsonrpc": "2.0",
            "id": "0",
            "method": "create_address",
            "params": params
        }))).await?;

        if let Some(error) = res.get("error") {
            let msg = error["message"].as_str().unwrap_or("Unknown RPC Error");
            return Err(format!("Wallet RPC Failure: {}", msg).into());
        }

        Ok(res["result"].clone())
    }
}