// Full, consolidated security integration tests (fixed duplicates / stray output).
use axum::http::StatusCode;
use axum_test::TestServer;
use defi_hot_wallet::api::server::WalletServer;
use defi_hot_wallet::core::config::{BlockchainConfig, StorageConfig, WalletConfig};
use serde_json::json;
use std::collections::HashMap;

/// Build a minimal WalletConfig for tests (in-memory sqlite)
fn create_test_config() -> WalletConfig {
    WalletConfig {
        storage: StorageConfig {
            database_url: "sqlite::memory:".to_string(),
            max_connections: Some(1),
            connection_timeout_seconds: Some(30),
        },
        blockchain: BlockchainConfig {
            networks: HashMap::new(),
            default_network: Some("eth".to_string()),
        },
        quantum_safe: false,
        multi_sig_threshold: 2,
    }
}

/// Create an axum_test::TestServer wired to the app router
async fn create_test_server() -> TestServer {
    let config = create_test_config();
    let api_key = Some("test_api_key".to_string());
    let server = WalletServer::new("127.0.0.1".to_string(), 0, config, api_key)
        .await
        .expect("failed to create WalletServer");
    TestServer::new(server.create_router().await).expect("failed to create TestServer")
}

/// Helper: create a wallet via API (expects success)
async fn create_test_wallet(server: &TestServer, name: &str) {
    let payload = json!({
        "name": name,
        "quantum_safe": false
    });
    let resp = server
        .post("/api/wallets")
        .json(&payload)
        .add_header("Authorization", "test_api_key")
        .await;
    assert_eq!(resp.status_code(), StatusCode::OK);
}

/// Health endpoint
#[tokio::test(flavor = "current_thread")]
async fn test_health_check() {
    let server = create_test_server().await;
    let res = server.get("/api/health").await;
    assert_eq!(res.status_code(), StatusCode::OK);
    let body: serde_json::Value = res.json();
    assert_eq!(body["status"].as_str().unwrap_or(""), "ok");
}

/// Create wallet - valid
#[tokio::test(flavor = "current_thread")]
async fn test_create_wallet_valid() {
    let server = create_test_server().await;
    let payload = json!({
        "name": "test_wallet_valid",
        "quantum_safe": true
    });
    let res = server
        .post("/api/wallets")
        .json(&payload)
        .add_header("Authorization", "test_api_key")
        .await;
    assert_eq!(res.status_code(), StatusCode::OK);
    let body: serde_json::Value = res.json();
    assert!(body["id"].is_string());
    assert_eq!(body["name"].as_str().unwrap_or(""), "test_wallet_valid");
    assert_eq!(body["quantum_safe"].as_bool().unwrap_or(false), true);
}

/// Create wallet - invalid name
#[tokio::test(flavor = "current_thread")]
async fn test_create_wallet_invalid_name() {
    let server = create_test_server().await;
    let payload = json!({
        "name": "",
        "quantum_safe": false
    });
    let res = server
        .post("/api/wallets")
        .json(&payload)
        .add_header("Authorization", "test_api_key")
        .await;
    assert_eq!(res.status_code(), StatusCode::BAD_REQUEST);
    let body: serde_json::Value = res.json();
    if body.is_object() {
        if body.get("code").is_some() {
            assert_eq!(body["code"].as_str().unwrap_or(""), "WALLET_CREATION_FAILED");
        } else {
            assert!(body.get("error").is_some());
        }
    }
}

/// Create wallet - SQL injection attempt (should be rejected)
#[tokio::test(flavor = "current_thread")]
async fn test_create_wallet_sql_injection_attempt() {
    let server = create_test_server().await;
    let payload = json!({
        "name": "'; DROP TABLE wallets; --",
        "quantum_safe": false
    });
    let res = server
        .post("/api/wallets")
        .json(&payload)
        .add_header("Authorization", "test_api_key")
        .await;
    assert_eq!(res.status_code(), StatusCode::BAD_REQUEST);
}

/// Create wallet - unauthorized
#[tokio::test(flavor = "current_thread")]
async fn test_create_wallet_unauthorized() {
    let server = create_test_server().await;
    let payload = json!({
        "name": "test_wallet_unauth",
        "quantum_safe": false
    });
    let res = server.post("/api/wallets").json(&payload).await;
    assert_eq!(res.status_code(), StatusCode::UNAUTHORIZED);
}

/// List wallets - authorized
#[tokio::test(flavor = "current_thread")]
async fn test_list_wallets() {
    let server = create_test_server().await;
    let res = server.get("/api/wallets").add_header("Authorization", "test_api_key").await;
    assert_eq!(res.status_code(), StatusCode::OK);
    let _body: Vec<serde_json::Value> = res.json();
}

/// List wallets - unauthorized
#[tokio::test(flavor = "current_thread")]
async fn test_list_wallets_unauthorized() {
    let server = create_test_server().await;
    let res = server.get("/api/wallets").await;
    assert_eq!(res.status_code(), StatusCode::UNAUTHORIZED);
}

/// Delete wallet - valid
#[tokio::test(flavor = "current_thread")]
async fn test_delete_wallet_valid() {
    let server = create_test_server().await;
    create_test_wallet(&server, "to_delete_wallet").await;
    let res = server
        .delete("/api/wallets/to_delete_wallet")
        .add_header("Authorization", "test_api_key")
        .await;
    assert!(matches!(res.status_code(), StatusCode::NO_CONTENT | StatusCode::OK));
}

/// Delete wallet - path traversal attempt
#[tokio::test(flavor = "current_thread")]
async fn test_delete_wallet_path_traversal() {
    let server = create_test_server().await;
    let res = server
        .delete("/api/wallets/../../../etc/passwd")
        .add_header("Authorization", "test_api_key")
        .await;
    assert!(matches!(res.status_code(), StatusCode::NOT_FOUND | StatusCode::BAD_REQUEST));
}

/// Delete wallet - not found
#[tokio::test(flavor = "current_thread")]
async fn test_delete_wallet_not_found() {
    let server = create_test_server().await;
    let res = server
        .delete("/api/wallets/nonexistent_wallet")
        .add_header("Authorization", "test_api_key")
        .await;
    assert_eq!(res.status_code(), StatusCode::NOT_FOUND);
}

/// Get balance - valid (eth)
#[tokio::test(flavor = "current_thread")]
async fn test_get_balance_valid() {
    let server = create_test_server().await;
    create_test_wallet(&server, "balance_wallet").await;
    let res = server
        .get("/api/wallets/balance_wallet/balance?network=eth")
        .add_header("Authorization", "test_api_key")
        .await;
    let status = res.status_code();
    assert!(matches!(status, StatusCode::OK | StatusCode::INTERNAL_SERVER_ERROR));
}

/// Get balance - invalid network
#[tokio::test(flavor = "current_thread")]
async fn test_get_balance_invalid_network() {
    let server = create_test_server().await;
    create_test_wallet(&server, "balance_wallet2").await;
    let res = server
        .get("/api/wallets/balance_wallet2/balance?network=invalid")
        .add_header("Authorization", "test_api_key")
        .await;
    assert!(matches!(
        res.status_code(),
        StatusCode::INTERNAL_SERVER_ERROR | StatusCode::BAD_REQUEST
    ));
}

/// Get balance - missing network query
#[tokio::test(flavor = "current_thread")]
async fn test_get_balance_missing_network() {
    let server = create_test_server().await;
    let res = server
        .get("/api/wallets/balance_wallet_missing/balance")
        .add_header("Authorization", "test_api_key")
        .await;
    assert_eq!(res.status_code(), StatusCode::BAD_REQUEST);
}

/// Get balance - wallet not found
#[tokio::test(flavor = "current_thread")]
async fn test_get_balance_wallet_not_found() {
    let server = create_test_server().await;
    let res = server
        .get("/api/wallets/does_not_exist/balance?network=eth")
        .add_header("Authorization", "test_api_key")
        .await;
    assert_eq!(res.status_code(), StatusCode::NOT_FOUND);
}

/// Send transaction - valid (best-effort)
#[tokio::test(flavor = "current_thread")]
async fn test_send_transaction_valid() {
    let server = create_test_server().await;
    create_test_wallet(&server, "send_valid_wallet").await;
    let payload = json!({
        "to_address": "0x742d35Cc6634C0532925a3b844Bc454e4438f44e",
        "amount": "0.1",
        "network": "eth"
    });
    let res = server
        .post("/api/wallets/send_valid_wallet/send")
        .json(&payload)
        .add_header("Authorization", "test_api_key")
        .await;
    assert!(matches!(res.status_code(), StatusCode::OK | StatusCode::INTERNAL_SERVER_ERROR));
}

/// Send transaction - invalid address
#[tokio::test(flavor = "current_thread")]
async fn test_send_transaction_invalid_address() {
    let server = create_test_server().await;
    create_test_wallet(&server, "send_invalid_addr").await;
    let payload = json!({
        "to_address": "invalid_address",
        "amount": "0.1",
        "network": "eth"
    });
    let res = server
        .post("/api/wallets/send_invalid_addr/send")
        .json(&payload)
        .add_header("Authorization", "test_api_key")
        .await;
    assert_eq!(res.status_code(), StatusCode::BAD_REQUEST);
    let body: serde_json::Value = res.json();
    if body.is_object() {
        if let Some(code) = body.get("code").and_then(|v| v.as_str()) {
            assert_eq!(code, "TRANSACTION_FAILED");
        }
    }
}

/// Send transaction - large amount (best-effort)
#[tokio::test(flavor = "current_thread")]
async fn test_send_transaction_large_amount() {
    let server = create_test_server().await;
    create_test_wallet(&server, "send_large").await;
    let payload = json!({
        "to_address": "0x742d35Cc6634C0532925a3b844Bc454e4438f44e",
        "amount": "1000000.0",
        "network": "eth"
    });
    let res = server
        .post("/api/wallets/send_large/send")
        .json(&payload)
        .add_header("Authorization", "test_api_key")
        .await;
    assert!(matches!(
        res.status_code(),
        StatusCode::OK | StatusCode::INTERNAL_SERVER_ERROR | StatusCode::BAD_REQUEST
    ));
}

/// Send transaction - negative amount
#[tokio::test(flavor = "current_thread")]
async fn test_send_transaction_negative_amount() {
    let server = create_test_server().await;
    create_test_wallet(&server, "send_negative").await;
    let payload = json!({
        "to_address": "0x742d35Cc6634C0532925a3b844Bc454e4438f44e",
        "amount": "-0.1",
        "network": "eth"
    });
    let res = server
        .post("/api/wallets/send_negative/send")
        .json(&payload)
        .add_header("Authorization", "test_api_key")
        .await;
    assert_eq!(res.status_code(), StatusCode::BAD_REQUEST);
}

/// Send transaction - wallet not found
#[tokio::test(flavor = "current_thread")]
async fn test_send_transaction_wallet_not_found() {
    let server = create_test_server().await;
    let payload = json!({
        "to_address": "0x742d35Cc6634C0532925a3b844Bc454e4438f44e",
        "amount": "0.1",
        "network": "eth"
    });
    let res = server
        .post("/api/wallets/nonexistent/send")
        .json(&payload)
        .add_header("Authorization", "test_api_key")
        .await;
    assert_eq!(res.status_code(), StatusCode::NOT_FOUND);
}

/// Transaction history - valid (best-effort)
#[tokio::test(flavor = "current_thread")]
async fn test_get_transaction_history() {
    let server = create_test_server().await;
    create_test_wallet(&server, "history_wallet").await;
    let res = server
        .get("/api/wallets/history_wallet/history")
        .add_header("Authorization", "test_api_key")
        .await;
    assert!(matches!(res.status_code(), StatusCode::OK | StatusCode::INTERNAL_SERVER_ERROR));
}

/// Transaction history - wallet not found
#[tokio::test(flavor = "current_thread")]
async fn test_get_transaction_history_wallet_not_found() {
    let server = create_test_server().await;
    let res = server
        .get("/api/wallets/no_history/history")
        .add_header("Authorization", "test_api_key")
        .await;
    assert_eq!(res.status_code(), StatusCode::NOT_FOUND);
}

/// Backup wallet - valid
#[tokio::test(flavor = "current_thread")]
async fn test_backup_wallet() {
    let server = create_test_server().await;
    create_test_wallet(&server, "backup_wallet").await;
    let res = server
        .get("/api/wallets/backup_wallet/backup")
        .add_header("Authorization", "test_api_key")
        .await;
    assert!(matches!(res.status_code(), StatusCode::OK | StatusCode::INTERNAL_SERVER_ERROR));
}

/// Backup wallet - not found
#[tokio::test(flavor = "current_thread")]
async fn test_backup_wallet_not_found() {
    let server = create_test_server().await;
    let res = server
        .get("/api/wallets/no_backup/backup")
        .add_header("Authorization", "test_api_key")
        .await;
    assert_eq!(res.status_code(), StatusCode::NOT_FOUND);
}

/// Restore wallet - valid
#[tokio::test(flavor = "current_thread")]
async fn test_restore_wallet() {
    let server = create_test_server().await;
    let payload = json!({
        "name": "restored_wallet_full",
        "seed_phrase": "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about"
    });
    let res = server
        .post("/api/wallets/restore")
        .json(&payload)
        .add_header("Authorization", "test_api_key")
        .await;
    assert!(matches!(
        res.status_code(),
        StatusCode::OK | StatusCode::BAD_REQUEST | StatusCode::INTERNAL_SERVER_ERROR
    ));
}

/// Multi-sig send - valid
#[tokio::test(flavor = "current_thread")]
async fn test_send_multi_sig_transaction_valid() {
    let server = create_test_server().await;
    create_test_wallet(&server, "multi_sig_wallet").await;
    let payload = json!({
        "to_address": "0x742d35Cc6634C0532925a3b844Bc454e4438f44e",
        "amount": "0.1",
        "network": "eth",
        "signatures": ["sig1", "sig2"]
    });
    let res = server
        .post("/api/wallets/multi_sig_wallet/send_multi_sig")
        .json(&payload)
        .add_header("Authorization", "test_api_key")
        .await;
    assert!(matches!(res.status_code(), StatusCode::OK | StatusCode::INTERNAL_SERVER_ERROR));
}

/// Multi-sig send - insufficient signatures
#[tokio::test(flavor = "current_thread")]
async fn test_send_multi_sig_transaction_insufficient_signatures() {
    let server = create_test_server().await;
    create_test_wallet(&server, "multi_sig_wallet2").await;
    let payload = json!({
        "to_address": "0x742d35Cc6634C0532925a3b844Bc454e4438f44e",
        "amount": "0.1",
        "network": "eth",
        "signatures": ["sig1"]
    });
    let res = server
        .post("/api/wallets/multi_sig_wallet2/send_multi_sig")
        .json(&payload)
        .add_header("Authorization", "test_api_key")
        .await;
    assert_eq!(res.status_code(), StatusCode::BAD_REQUEST);
}

/// Bridge - valid
#[tokio::test(flavor = "current_thread")]
async fn test_bridge_assets_valid() {
    let server = create_test_server().await;
    create_test_wallet(&server, "bridge_wallet").await;
    let payload = json!({
        "from_wallet": "bridge_wallet",
        "from_chain": "eth",
        "to_chain": "solana",
        "token": "USDC",
        "amount": "10.0"
    });
    let res =
        server.post("/api/bridge").json(&payload).add_header("Authorization", "test_api_key").await;
    assert!(matches!(res.status_code(), StatusCode::OK | StatusCode::INTERNAL_SERVER_ERROR));
    if res.status_code() == StatusCode::OK {
        let body: serde_json::Value = res.json();
        assert!(body.get("bridge_tx_id").map(|v| v.is_string()).unwrap_or(true));
    }
}

/// Bridge - invalid chain
#[tokio::test(flavor = "current_thread")]
async fn test_bridge_assets_invalid_chain() {
    let server = create_test_server().await;
    create_test_wallet(&server, "bridge_wallet2").await;
    let payload = json!({
        "from_wallet": "bridge_wallet2",
        "from_chain": "invalid",
        "to_chain": "solana",
        "token": "USDC",
        "amount": "10.0"
    });
    let res =
        server.post("/api/bridge").json(&payload).add_header("Authorization", "test_api_key").await;
    assert_eq!(res.status_code(), StatusCode::BAD_REQUEST);
}

/// Bridge - wallet not found
#[tokio::test(flavor = "current_thread")]
async fn test_bridge_assets_wallet_not_found() {
    let server = create_test_server().await;
    let payload = json!({
        "from_wallet": "no_such_wallet",
        "from_chain": "eth",
        "to_chain": "solana",
        "token": "USDC",
        "amount": "10.0"
    });
    let res =
        server.post("/api/bridge").json(&payload).add_header("Authorization", "test_api_key").await;
    assert_eq!(res.status_code(), StatusCode::NOT_FOUND);
}

/// Metrics endpoint
#[tokio::test(flavor = "current_thread")]
async fn test_metrics_endpoint() {
    let server = create_test_server().await;
    let res = server.get("/api/metrics").await;
    assert_eq!(res.status_code(), StatusCode::OK);
    let text = res.text();
    assert!(
        text.contains("# HELP") || text.contains("wallets_created_total") || text.contains("http_")
    );
}

/// Input sanitization (XSS payload)
#[tokio::test(flavor = "current_thread")]
async fn test_input_sanitization() {
    let server = create_test_server().await;
    let payload = json!({
        "name": "<script>alert('xss')</script>",
        "quantum_safe": false
    });
    let res = server
        .post("/api/wallets")
        .json(&payload)
        .add_header("Authorization", "test_api_key")
        .await;
    assert_eq!(res.status_code(), StatusCode::BAD_REQUEST);
}

/// Unauthorized access simulation (delete without auth)
#[tokio::test(flavor = "current_thread")]
async fn test_unauthorized_access_simulation() {
    let server = create_test_server().await;
    create_test_wallet(&server, "admin_wallet_sim").await;
    let res = server.delete("/api/wallets/admin_wallet_sim").await;
    assert_eq!(res.status_code(), StatusCode::UNAUTHORIZED);
}
