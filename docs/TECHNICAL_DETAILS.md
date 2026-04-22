# 🛠️ AKASA Sentinel Technical Details (v2.0.0 Global Standard)

This documentation provides an in-depth explanation of the internal architecture of AKASA Sentinel. Designed for developers and security researchers, it demonstrates Sentinel's integrity as a "Simple, Robust, Secure, Modern, and Smart" Monero payment engine.

---

## 1. Scanning Logic
Sentinel operates on the Zero-Spend-Key principle, ensuring the absolute security of merchant assets.

* Elliptic Curve Cryptography (ECC) Verification: Sentinel uses a Private View Key and a Public Spend Key to decrypt transaction outputs on the blockchain. This process is performed locally in memory without ever sending the keys to external nodes.
* Stealth Address Matching: Every transaction output on the Monero network is checked using a mathematical algorithm to see if there is a One-Time Public Key that matches the monitored wallet.

* Non-Custodial: Because Sentinel does not require a Private Spend Key, the system is by design technically incapable of moving or stealing funds.

---

## 2. Asynchronous Engine (High-Performance Async)
Sentinel is built on the Tokio Runtime, the industry standard for Rust applications requiring high concurrency.

* Multi-Producer Single-Consumer (MPSC): The internal communication path separates the tasks between the Scanner and the Webhook Worker. The Scanner does not need to wait for the webhook delivery to complete before continuing to scan the next block.
* Worker Isolation: Any failures in the notification process are isolated. If one merchant experiences a network outage, it will not impact the scanning speed of other merchants' wallets.

---

## 3. Network Privacy & Anonymity (Tor/SOCKS5)
Metadata privacy is just as important as transaction privacy.

* **SOCKS5 Proxy Support**: Sentinel supports routing traffic through SOCKS5 proxies (such as Tor) to hide the server's true IP address.
* **End-to-End Onion Routing**: Full support for `.onion` nodes, allowing Sentinel to communicate directly within the hidden network without exiting the public internet (Clearnet).
* **Docker Sidecar Deployment**: Recommended use of Tor as a companion container (sidecar) to ensure instant, standardized data path encryption.

---

## 4. Mempool & 0-Conf Logic (Smart)
Sentinel provides an instant payment experience through early detection.

* **Real-time Mempool Sync**: Sentinel monitors Monero node mempools to detect transactions before blocks are mined.
* **Idempotency Logic**: The system ensures that each transaction is processed only once. If a transaction found in the mempool is later included in a block, Sentinel reconciles the status without triggering unnecessary duplicate webhooks.

---

## 5. Multi-Wallet Scalability (Strong)
Sentinel is designed to handle hundreds of wallets in a single process.

* Parallel Wallet Watchers: Each registered wallet runs in its own Green Thread (Tokio Task). This enables parallel scanning that efficiently utilizes all CPU cores.
* Resource Efficiency: Despite handling multiple wallets, RAM usage remains low because Sentinel only stores active states and necessary metadata.

---

## 6. Database & Data Integrity
Sentinel uses highly optimized SQLite to maintain speed and data consistency.

* WAL Mode (Write-Ahead Logging): Allows reads and writes to occur simultaneously without locking, crucial for high transaction volumes.
* Database Indexing: Automatic indexes on the `tx_hash`, `address`, and `status` columns ensure instantaneous queries as data grows.
* **Graceful Shutdown**: Handles `SIGTERM/SIGINT` signals elegantly. Sentinel will complete any ongoing data writes and close the database connection cleanly to prevent data corruption (DB Corruption).
* **Auto-Pruning**: An automatic cleanup mechanism to remove old transaction logs, keeping the database size stable.

---

## 7. Notification Reliability (Webhooks)
Ensures merchants always receive updates about their payments.

* **HMAC-SHA256 Signing**: Each webhook payload is cryptographically signed. Merchants can verify that the data actually came from Sentinel via the `X-Akasa-Signature` header.
* **Exponential Backoff Retry**: If a webhook delivery fails, Sentinel will retry with gradually increasing delays (e.g., 1, 2, 4, 8 minutes) to give the merchant's server time to recover.

---

## 8. The Future: The Vault (v2.0.0)
Security enhancements in future releases will include:

* Argon2id Key Derivation: The global gold standard for converting passwords into encryption keys.
* AES-256-GCM Encryption: All sensitive data on disk will be fully encrypted. The View Key will only exist in

---
*© 2026 AKASA Sentinel by Siptruk. Sovereign technology for the global privacy community.*