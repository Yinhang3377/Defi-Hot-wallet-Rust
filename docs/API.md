# DeFi Hot Wallet API Documentation

## Base URL

```
http://localhost:8080/api
```

## Authentication

Currently, the API does not require authentication for demonstration purposes. In production, implement proper authentication mechanisms such as:

- JWT tokens
- API keys
- OAuth 2.0
- Hardware security keys

## Endpoints

### Health Check

Check if the wallet service is running and healthy.

```http
GET /health
```

**Response:**
```json
{
  "status": "healthy",
  "timestamp": "2024-01-15T12:00:00Z",
  "version": "0.1.0"
}
```

### Metrics

Get Prometheus metrics for monitoring.

```http
GET /metrics
```

**Response:**
```
# HELP wallets_created_total Total number of wallets created
# TYPE wallets_created_total counter
wallets_created_total 5

# HELP transactions_sent_total Total number of transactions sent
# TYPE transactions_sent_total counter
transactions_sent_total 12
```

### Wallets

#### Create Wallet

Create a new wallet with optional quantum-safe encryption.

```http
POST /wallets
Content-Type: application/json

{
  "name": "my-wallet",
  "quantum_safe": true
}
```

**Response:**
```json
{
  "id": "550e8400-e29b-41d4-a716-446655440000",
  "name": "my-wallet",
  "quantum_safe": true,
  "created_at": "2024-01-15T12:00:00Z"
}
```

#### List Wallets

Get a list of all wallets (metadata only).

```http
GET /wallets
```

**Response:**
```json
[
  {
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "name": "my-wallet",
    "quantum_safe": true,
    "created_at": "2024-01-15T12:00:00Z"
  }
]
```

#### Delete Wallet

Delete a wallet by name.

```http
DELETE /wallets/{name}
```

**Response:**
```
204 No Content
```

#### Get Balance

Get wallet balance for a specific network.

```http
GET /wallets/{name}/balance?network=eth
```

**Parameters:**
- `network`: Network name (eth, solana, etc.)

**Response:**
```json
{
  "balance": "1.234567890",
  "network": "eth",
  "symbol": "ETH"
}
```

#### Send Transaction

Send a transaction from a wallet.

```http
POST /wallets/{name}/send
Content-Type: application/json

{
  "to_address": "0x742d35Cc6635C0532925a3b8D400e8B78fFe4860",
  "amount": "0.1",
  "network": "eth"
}
```

**Response:**
```json
{
  "tx_hash": "0x1234567890abcdef...",
  "status": "sent"
}
```

## Error Responses

All endpoints may return the following error responses:

### 400 Bad Request
```json
{
  "error": "Invalid request parameters",
  "code": "BAD_REQUEST"
}
```

### 404 Not Found
```json
{
  "error": "Wallet not found: my-wallet",
  "code": "WALLET_NOT_FOUND"
}
```

### 500 Internal Server Error
```json
{
  "error": "Internal server error",
  "code": "INTERNAL_ERROR"
}
```

## Rate Limiting

The API implements rate limiting to prevent abuse:

- 100 requests per minute per IP
- 10 wallet creation requests per hour per IP
- 50 transaction requests per hour per wallet

## Security Features

### Request Validation

All requests are validated for:
- Input sanitization
- Parameter validation
- Size limits
- Content type verification

### Audit Logging

All API requests are logged with:
- Timestamp
- IP address
- User agent
- Request parameters (sensitive data excluded)
- Response status

### Monitoring

The API provides comprehensive monitoring through:
- Prometheus metrics
- Health checks
- Performance tracking
- Error rates

## Example Usage

### Using curl

```bash
# Create a wallet
curl -X POST http://localhost:8080/api/wallets \
  -H "Content-Type: application/json" \
  -d '{"name": "test-wallet", "quantum_safe": true}'

# Check balance
curl "http://localhost:8080/api/wallets/test-wallet/balance?network=eth"

# Send transaction
curl -X POST http://localhost:8080/api/wallets/test-wallet/send \
  -H "Content-Type: application/json" \
  -d '{
    "to_address": "0x742d35Cc6635C0532925a3b8D400e8B78fFe4860",
    "amount": "0.1",
    "network": "eth"
  }'
```

### Using JavaScript

```javascript
const API_BASE = 'http://localhost:8080/api';

// Create wallet
async function createWallet(name, quantumSafe = true) {
  const response = await fetch(`${API_BASE}/wallets`, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
    },
    body: JSON.stringify({
      name: name,
      quantum_safe: quantumSafe
    })
  });
  return response.json();
}

// Get balance
async function getBalance(walletName, network) {
  const response = await fetch(
    `${API_BASE}/wallets/${walletName}/balance?network=${network}`
  );
  return response.json();
}

// Send transaction
async function sendTransaction(walletName, toAddress, amount, network) {
  const response = await fetch(`${API_BASE}/wallets/${walletName}/send`, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
    },
    body: JSON.stringify({
      to_address: toAddress,
      amount: amount,
      network: network
    })
  });
  return response.json();
}
```

### Using Python

```python
import requests

API_BASE = 'http://localhost:8080/api'

def create_wallet(name, quantum_safe=True):
    response = requests.post(f'{API_BASE}/wallets', json={
        'name': name,
        'quantum_safe': quantum_safe
    })
    return response.json()

def get_balance(wallet_name, network):
    response = requests.get(
        f'{API_BASE}/wallets/{wallet_name}/balance',
        params={'network': network}
    )
    return response.json()

def send_transaction(wallet_name, to_address, amount, network):
    response = requests.post(f'{API_BASE}/wallets/{wallet_name}/send', json={
        'to_address': to_address,
        'amount': amount,
        'network': network
    })
    return response.json()
```

## WebSocket API (Future)

Planned WebSocket endpoints for real-time updates:

```javascript
// Connect to WebSocket
const ws = new WebSocket('ws://localhost:8080/ws');

// Subscribe to wallet events
ws.send(JSON.stringify({
  type: 'subscribe',
  wallet: 'my-wallet',
  events: ['balance_change', 'transaction_confirmed']
}));

// Handle events
ws.onmessage = (event) => {
  const data = JSON.parse(event.data);
  console.log('Wallet event:', data);
};
```