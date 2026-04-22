/*
 * AKASA Sentinel - A high-performance, self-sovereign Monero payment gateway built for privacy and speed.
 * Copyright (C) 2026 Siptruk
 * Licensed under the GNU General Public License v3.0
 */

use monero::util::key::ViewPair;
use monero::{Address, Network, PrivateKey, PublicKey, Transaction};
use serde::Deserialize;
use std::str::FromStr;
use tracing::{error};

#[derive(Deserialize, Clone)]
pub struct MerchantWallet {
    pub label: String,
    pub confirmations: Option<u64>,
    pub view_key: String,
    pub public_spend_key: String,
    pub webhook_url: String,
    pub webhook_secret: String,

    #[serde(skip)]
    pub view_pair: Option<ViewPair>,
    #[serde(skip)]
    pub network: Network,
}

impl MerchantWallet {
    pub fn init(&mut self, network: Network) -> Result<(), String> {
        self.network = network;
        
        let priv_view = PrivateKey::from_str(&self.view_key)
            .map_err(|e| {
                error!("Invalid View Key for {}: {}", self.label, e);
                format!("Invalid View Key: {}", e)
            })?;

        let pub_spend = PublicKey::from_str(&self.public_spend_key)
            .map_err(|e| {
                error!("Invalid Public Spend Key for {}: {}", self.label, e);
                format!("Invalid Public Spend Key: {}", e)
            })?;

        self.view_pair = Some(ViewPair {
            view: priv_view,
            spend: pub_spend,
        });
        
        Ok(())
    }

    pub fn get_address(&self) -> String {
        if let Some(ref vp) = self.view_pair {
            let public_view_key = PublicKey::from_private_key(&vp.view);
            let addr = Address::standard(self.network, vp.spend, public_view_key);
            return addr.to_string();
        }
        "Invalid Wallet".to_string()
    }

    pub fn scan_transaction(&self, tx: &Transaction) -> Option<u64> {
        if let Some(ref vp) = self.view_pair {
            if let Ok(owned_outputs) = tx.check_outputs(vp, 0..1, 0..1) {
                if !owned_outputs.is_empty() {
                    let mut total: u64 = 0;
                    for output in owned_outputs {
                        if let Some(amt) = output.amount() {
                            total += amt.as_pico();
                        }
                    }
                    if total > 0 {
                        return Some(total);
                    }
                }
            }
        }
        None
    }
}