# 🛡️ AKASA Sentinel
[![GitHub Release](https://img.shields.io/github/v/release/xptruk/akasa-sentinel?style=flat-square&color=blue)](https://github.com/xptruk/akasa-sentinel/releases)
![Rust](https://img.shields.io/badge/language-Rust-orange?style=flat-square&logo=rust)
![License](https://img.shields.io/badge/license-GPLv3-blue?style=flat-square)
![Tor](https://img.shields.io/badge/network-Tor--Ready-purple?style=flat-square&logo=tor-project)
![Monero Badge](https://img.shields.io/badge/Privacy-Monero-F60?logo=monero&logoColor=fff&style=flat-square)

> **A high-performance, self-sovereign Monero payment gateway built for privacy and speed.**

AKASA Sentinel is a high-performance Monero (XMR) transaction scanning engine built in **Rust**. It's specifically designed for merchants and developers who prioritize total privacy and complete control over their own payment infrastructure without relying on third parties.

---

## 🌟 Why AKASA Sentinel?

In the digital economy, privacy is a fundamental right. Sentinel gives you the ability to accept Monero payments autonomously.

* **⚡ High Speed**: Real-time blockchain and mempool scanning based on Rust.
* **🔒 Absolute Security**: Uses a *Watch-only* scheme. Your funds remain secure in a cold wallet because Sentinel only requires a *View Key*.

* **🌐 Network Privacy**: Full support for Tor and SOCKS5 to hide your server's identity.
* **🤖 Smart Automation**: Instant webhook notifications as soon as a transaction is detected (0-Conf).

---

## ✨ Key Features

### 🛡️ Security & Privacy (Pro Focus)
- **Active API Protection**: Administrative access (Port 9090) is protected by `X-API-Key` authentication.
- **HMAC-SHA256 Webhook**: Every piece of data sent to your server is cryptographically signed to prevent data forgery.
- **Zero-Trust Architecture**: No Spend Key required. It's impossible for Sentinel to spend your funds.

### ⚙️ Performance & Scalability
- **Asynchronous Worker**: Uses a non-blocking architecture (Tokio/MPSC). The scanning process won't stop even if your merchant server is slow to respond.
- **Multi-Wallet Support**: Monitor dozens of stores or addresses simultaneously in one lightweight process.
- **Intelligent Failover**: Automatically switch to a *backup node* if the primary node experiences a failure.

### 🧠 Automatic Data Management
- **Automated Indexing**: Transaction data searches remain instantaneous even when the data reaches thousands.
- **Auto-Pruning**: Automatically cleans out outdated data to keep the database lightweight.

---

## ⚙️ Operational Mode

You can adjust Sentinel's workload through the `DATABASE_MODE` variable in the `.env` file:

1. **Standalone Mode (RAM-Only)**:
* Runs entirely in memory. Very fast and private.
* *Suitable for: Monitoring static addresses or devices with limited storage.*
2. **Integrated Mode (SQLite)**:
* Saves data to a local database. Enable the **Active API (Cashier)** feature.
* Suitable for: Commercial payment gateways that require dynamic invoice generation. *

---

## 🚀 Quick Setup

### 1. Environment Configuration (`.env`)
Copy the example file and set your API key:
```bash
cp .env.example .env
```
*Make sure to fill in `MGMT_API_KEY` to secure API access.*

### 2. Wallet Configuration (`wallets.json`)
Copy the example file, add your *View Key* and wallet settings.

```bash
cp wallet.json.example wallet.json
```

### 3. Run Sentinel
```bash
./akasa-sentinel
```

---

## 📖 Advanced Documentation
To keep this guide concise, technical and operational details are separated into the following files:

- **[TECHNICAL_DETAILS.md](./docs/TECHNICAL_DETAILS.md)**: An in-depth explanation of the ECC architecture, scanning logic, and network privacy.
- **[WEBHOOK_API.md](./docs/WEBHOOK_API.md)**: A complete specification of the Webhook JSON payload and an Active API guide.
- **[DEPLOYMENT.md](./docs/DEPLOYMENT.md)**: A guide to installation as a **Systemd Service**, using Docker, and server optimization.

---

## 📡 Webhook & API (Developer Guide)

### Webhook Events
Sentinel will send JSON data to your URL when the transaction status changes:
* `tx_mempool`: Transaction detected on the network (Confirmations 0).
* `tx_confirmed`: Payment has reached your confirmation target.
* `heartbeat`: Regular reports that your system remains active 24/7.

### Active API (Port 9090)
Use this API to automatically generate unique payment addresses for each of your customers.
> **Required Header:** `X-API-Key: <Your_Key>`

---

## ⚖️ License
This project is licensed under the **GPLv3 License** - see the [LICENSE](LICENSE) file for details. Technology sovereignty for all.

---

### ☕ Support the Developers
If AKASA Sentinel helps your business, consider donating to further develop privacy features:
`XMR: 447rquZUbZk2VUVjx9eBn28U71GH872ESgVhWUNhbLfY81upf2psoUpJo4rZB8chgQHcpHX2bKKhdFyRaF2Fn4cq2X4Tvto`

---
*© 2026 AKASA Sentinel by Siptruk. Sovereign technology for the global privacy community.*