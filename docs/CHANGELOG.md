# 📋 Changelog

All notable changes to **AKASA Sentinel** will be documented in this file.
This project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## [1.0.0]
### 🚀 Initial Stable Release
This is the first official stable release of AKASA Sentinel, providing a high-performance, self-sovereign gateway for Monero payments.

### ✨ Features
- **High-Performance Scanning**: Asynchronous transaction detection engine built with Rust and Tokio.
- **Intelligent Node Failover**: Automatically switches to healthy backup nodes if the primary Monero node becomes unreachable or falls out of sync.
- **Watch-Only Architecture**: Maximum security using only Private View Keys; no Spend Keys required.
- **Dual Operational Modes**: 
  - `Standalone`: RAM-only for maximum privacy and speed.
  - `Integrated`: Persistence via SQLite for cashier and invoice management.
- **Autonomous Database Purge**: Automatically removes expired records to maintain optimal disk usage and performance.
- **Heartbeat Mechanism**: Periodic health signals (every 10 minutes) to ensure persistent connectivity between Sentinel and the merchant's backend.
- **Configurable Performance**: Fine-tune scanning intervals and resource usage via environment variables to match hardware capabilities.
- **Cryptographic Security**: 
  - `HMAC-SHA256` signed webhooks via `X-Akasa-Signature` for data integrity.
  - `X-API-Key` protected Management API (Port 9090).
- **Network Anonymity**: Native support for SOCKS5 and Tor proxies for secure `.onion` node connections.
- **Real-time 0-Conf**: Instant mempool detection with full lifecycle tracking to block confirmation.
- **Scalable Concurrency**: Multi-wallet monitoring using non-blocking MPSC channels.
- **Graceful Shutdown**: Ensures data integrity by cleanly closing database connections and finishing active tasks on exit.

---
*© 2026 AKASA Sentinel by Siptruk. Sovereign technology for the global privacy community.*