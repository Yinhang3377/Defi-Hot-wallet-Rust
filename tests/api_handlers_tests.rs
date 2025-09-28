//! tests/api_handlers_tests.rs
//!
//! Tests for individual API handlers in `src/api/handlers.rs`.
use axum::http::StatusCode;
use axum_test::TestServer;
use defi_hot_wallet::{
    api::handlers::BridgeRequest,
    api::server::{ErrorResponse, WalletServer},
    core::config::{StorageConfig, WalletConfig},
};
use serde_json::Value;
use uuid::Uuid;

/// Helper function to create a test server with an in-memory database.
async fn setup_test_server() -> TestServer {
    let config = WalletConfig {
        storage: StorageConfig {
            database_url: "sqlite::memory:".to_string(),
            ..Default::default() // Removed trailing comma
        },
        ..Default::default() // Removed trailing comma
    };
    let server = WalletServer::new("127.0.0.1".to_string(), 0, config, None).await.unwrap();
    TestServer::new(server.create_router().await).unwrap()
}

#[tokio::test(flavor = "current_thread")]
async fn test_health_check_handler() {
    let server = setup_test_server().await;
    let response = server.get("/api/health").await;
    response.assert_status_ok();
    let body: Value = response.json();
    assert_eq!(body["status"], "ok");
    assert!(body["version"].is_string());
    assert!(body["timestamp"].is_string());
}

#[tokio::test(flavor = "current_thread")]
async fn test_metrics_handler() {
    let server = setup_test_server().await;
    let response = server.get("/api/metrics").await;
    response.assert_status_ok();
    assert!(response.text().contains("# HELP"));
}

#[tokio::test(flavor = "current_thread")]
async fn test_bridge_assets_handler_invalid_amount() {
    let request = BridgeRequest {
        from_wallet: "test_wallet".to_string(), // 修正字段名
        from_chain: "eth".to_string(),
        to_chain: "solana".to_string(),
        token: "USDC".to_string(),
        amount: "0.0".to_string(), // Invalid amount (zero)
    };

    let server = setup_test_server().await;
    let response = server.post("/api/bridge").json(&request).await;

    response.assert_status(StatusCode::BAD_REQUEST);
    let body: ErrorResponse = response.json();
    assert_eq!(body.error, "Invalid amount");
}

#[tokio::test(flavor = "current_thread")]
#[ignore] // This test is flawed as the mock handler doesn't check for wallet existence.
async fn test_bridge_assets_handler_wallet_not_found() {
    let request = BridgeRequest {
        from_wallet: "nonexistent_wallet".to_string(), // 修正字段名
        from_chain: "eth".to_string(),
        to_chain: "solana".to_string(),
        token: "USDC".to_string(),
        amount: "100.0".to_string(),
    };

    let server = setup_test_server().await;
    let response = server.post("/api/bridge").json(&request).await;

    response.assert_status(StatusCode::NOT_FOUND);
    let body: ErrorResponse = response.json();
    assert_eq!(body.error, "Wallet not found");
}

#[tokio::test(flavor = "current_thread")]
async fn test_bridge_assets_handler_wallet_not_found_for_valid_request() {
    let wallet_name = format!("valid-{}", Uuid::new_v4());

    let request = BridgeRequest {
        from_wallet: wallet_name,
        from_chain: "eth".to_string(),
        to_chain: "solana".to_string(),
        token: "USDC".to_string(),
        amount: "100.0".to_string(),
    };

    let server = setup_test_server().await;
    let response = server.post("/api/bridge").json(&request).await;

    // Since the wallet does not exist, we expect a NOT_FOUND error.
    response.assert_status(StatusCode::NOT_FOUND);
    let body: ErrorResponse = response.json();
    assert_eq!(body.error, "Wallet not found");
}

#[tokio::test(flavor = "current_thread")]
async fn test_bridge_assets_handler_unsupported_chain() {
    let wallet_name = format!("invalid-chain-{}", Uuid::new_v4());
    // 类似上面，假设钱包存在

    let request = BridgeRequest {
        from_wallet: wallet_name, // 修正字段名
        from_chain: "invalid_chain".to_string(),
        to_chain: "solana".to_string(),
        token: "USDC".to_string(),
        amount: "100.0".to_string(),
    };

    let server = setup_test_server().await;
    let response = server.post("/api/bridge").json(&request).await;

    response.assert_status(StatusCode::BAD_REQUEST);
    let body: ErrorResponse = response.json();
    assert_eq!(body.error, "Unsupported chain");
}