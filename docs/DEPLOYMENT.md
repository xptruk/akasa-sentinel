# 🚀 AKASA Sentinel Deployment Guide

This document explains how to run AKASA Sentinel in a production environment to ensure stability, security, and resilience.

---

## 1. Running as a Systemd Service (Linux)
The best way to ensure Sentinel automatically starts when the server reboots and automatically restarts if it crashes.

1. Create a service file:
```bash
sudo nano /etc/systemd/system/akasa-sentinel.service
```

2. Enter the following configuration (adjust the folder path):
```ini
[Unit]
Description=AKASA Sentinel Monero Watcher
After=network.target

[Service]
Type=simple
User=youruser
WorkingDirectory=/home/youruser/akasa-sentinel
ExecStart=/home/youruser/akasa-sentinel/akasa-sentinel
Restart=always
RestartSec=5
Environment=RUST_LOG=info

[Install]
WantedBy=multi-user.target
```

3. Enable and run:
```bash
sudo systemctl daemon-reload
sudo systemctl enable akasa-sentinel
sudo systemctl start akasa-sentinel
```

---

## 2. Deployment with Docker (Modern)
If you prefer container isolation, use Docker.

**Simple Dockerfile:**
```dockerfile
FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY target/release/akasa-sentinel .
COPY .env .
COPY wallets.json .
CMD ["./akasa-sentinel"]
```

**Running:**
```bash
docker build -t akasa-sentinel .
docker run -d --name sentinel-node akasa-sentinel
```

---

## 3. Performance Optimization (Tuning)
To handle thousands of transactions and multiple wallets, ensure your file system limits are sufficient.

**Ulimit:**
Add the following line to `/etc/security/limits.conf` to allow the system to handle multiple simultaneous connections:
```text
* soft nofile 65535
* hard nofile 65535
```

---

## 4. Additional Security
- **Reverse Proxy**: If you expose Port 9090 to the internet, it is highly recommended to use **Nginx** with SSL (Certbot/LetsEncrypt).
- **Firewall**: Close port 9090 from public access unless absolutely necessary. Use `ufw allow from <YOUR_BACKEND_IP> to any port 9090`.

- **Log Rotation**: Use `logrotate` to prevent log files from bloating and filling up disk space.

---

## 5. Monitoring
You can monitor Sentinel's health through internal endpoints:
- `GET /v1/health` -> Checks whether the engine and database are active.
- `GET /v1/status` -> Brief statistics on the last block scan.

---
*© 2026 AKASA Sentinel by Siptruk. Sovereign technology for the global privacy community.*