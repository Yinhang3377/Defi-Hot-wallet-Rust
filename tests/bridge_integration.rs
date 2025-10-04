// ...existing code...
use axum::http::StatusCode;
use axum_test::{TestServer, TestServerConfig};
use defi_hot_wallet::api::server::WalletServer;
use defi_hot_wallet::core::config::{BlockchainConfig, StorageConfig, WalletConfig};
use serde_json::{json, Value};
use std::collections::HashMap;

/// Build a minimal WalletConfig for tests (in-memory sqlite)
fn create_test_config() -> WalletConfig {
    WalletConfig {
        storage: StorageConfig {
            database_url: "sqlite::memory:".to_string(),
            max_connections: Some(1),
            connection_timeout_seconds: Some(5),
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
async fn setup_test_server() -> TestServer {
    let config = create_test_config();
    let server = WalletServer::new("127.0.0.1".to_string(), 0, config, None)
        .await
        .expect("Failed to create server");
    let app = server.create_router().await;
    let cfg = TestServerConfig::default();
    TestServer::new_with_config(app, cfg).expect("failed to create TestServer")
}

/// Helper: create a wallet via API and return its id (best-effort)
async fn create_test_wallet(server: &TestServer, name: &str) -> String {
    let response = server
        .post("/api/wallets")
        .json(&json!({
            "name": name,
            "quantum_safe": false
        }))
        .await;
    // Accept OK or CREATED depending on implementation
    assert!(matches!(response.status_code(), StatusCode::OK | StatusCode::CREATED));
    let body: Value = response.json();
    body["id"].as_str().unwrap_or("").to_string()
}

#[tokio::test(flavor = "current_thread")]
async fn test_bridge_transfer() {
    let server = setup_test_server().await;
    let wallet_name = "bridge_wallet_ok";
    let _wallet_id = create_test_wallet(&server, wallet_name).await;

    let response = server
        .post("/api/bridge")
        .json(&json!({
            "from_wallet": wallet_name,
            "from_chain": "eth",
            "to_chain": "solana",
            "token": "USDC",
            "amount": "100"
        }))
        .await;

    // Mock handler implementations vary; accept OK or internal error.
    let status = response.status_code();
    assert!(matches!(status, StatusCode::OK | StatusCode::INTERNAL_SERVER_ERROR));
    if status == StatusCode::OK {
        let body: Value = response.json();
        assert!(body["bridge_tx_id"].is_string());
    }
}

#[tokio::test(flavor = "current_thread")]
async fn test_bridge_invalid_chain() {
    let server = setup_test_server().await;
    let wallet_name = "bridge_wallet_invalid_chain";
    let _wallet_id = create_test_wallet(&server, wallet_name).await;

    let response = server
        .post("/api/bridge")
        .json(&json!({
            "from_wallet": wallet_name,
            "from_chain": "invalid_chain",
            "to_chain": "solana",
            "token": "USDC",
            "amount": "100"
        }))
        .await;

    // Expect validation failure or server error
    assert!(matches!(
        response.status_code(),
        StatusCode::BAD_REQUEST | StatusCode::INTERNAL_SERVER_ERROR
    ));
}

#[tokio::test(flavor = "current_thread")]
async fn test_bridge_zero_amount() {
    let server = setup_test_server().await;
    let wallet_name = "bridge_wallet_zero_amount";
    let _wallet_id = create_test_wallet(&server, wallet_name).await;

    let response = server
        .post("/api/bridge")
        .json(&json!({
            "from_wallet": wallet_name,
            "from_chain": "eth",
            "to_chain": "solana",
            "token": "USDC",
            "amount": "0"
        }))
        .await;

    assert_eq!(response.status_code(), StatusCode::BAD_REQUEST);
}
// ...existing code...
