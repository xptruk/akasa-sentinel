# 📡 Webhook & API Integration Guide

This document details how AKASA Sentinel communicates with your server and how you can control Sentinel through the Management API.

---

## 1. Webhook Security (HMAC-SHA256)
Sentinel sends a cryptographic signature for every request. You must verify this signature to ensure the data actually comes from your Sentinel and not from a third party.

* **Header**: `X-Akasa-Signature`
* **Mechanism**: Sentinel calculates an `HMAC-SHA256` hash of the entire request body using the `WEBHOOK_SECRET` you set in your `.env` file.

---

## 2. Webhook Events (Event Types)

### A. `tx_mempool` (Instant Detection 0-Conf)
Sent as soon as a transaction appears on the Monero network, even if it hasn't been included in a block yet.
```json
{
"event": "tx_mempool",
"data": {
"address": "447r...",
"tx_hash": "62c5b...",
"amount": 1.25,
"confirmations": 0,
"label": "Kasir_Sumedang"
}
}
```

### B. `tx_confirmed` (Valid Payment)
Sent when a transaction has reached the number of confirmations you specify (e.g., 10 confirmations).
```json
{
"event": "tx_confirmed",
"data": {
"address": "447r...",
"tx_hash": "62c5b...",
"amount": 1.25,
"confirmations": 10,
"block_height": 3124565,
"label": "Kasir_Sumedang"
}
}
```

### C. `heartbeat` (Health Signal)
Sent every 10 minutes to ensure the connection between Sentinel and your backend remains alive.
```json
{
"event": "heartbeat",
"status": "online",
"timestamp": 1713800000,
"sync_height": 3124570
}
```

---

## 3. Management API (Port 9090)
Use this API to interact with Sentinel dynamically. All requests must include the `X-API-Key` header.

### [POST] Create Sub-address
Used to create a unique payment address for each customer (Integrated Mode only).

* **Endpoint**: `/v1/subaddress/create`
* **Payload**:
```json
{ "label": "Order_#12345" }
```
* **Response**:
```json
{ "address": "8A...", "label": "Order_#12345" }
```

### [GET] Health Check
Checks the engine and database status.
* **Endpoint**: `/v1/health`
* **Response**: `200 OK`

---

## 🛠️ Signature Verification Example (Node.js)
Use this logic on your server to securely receive webhooks:

```javascript
const crypto = require('crypto');

function verifySignature(payload, signature, secret) { 
const hmac = crypto.createHmac('sha256', secret); 
const digest = hmac.update(payload).digest('hex'); 
return digest === signature;
}
```

---
*© 2026 AKASA Sentinel by Siptruk. Sovereign technology for the global privacy community.*