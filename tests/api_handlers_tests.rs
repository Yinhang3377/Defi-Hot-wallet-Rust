// ...existing code...
//! tests/api_handlers_tests.rs
//!
//! Tests for individual API handlers in `src/api/handlers.rs`.
use axum::http::StatusCode;
use axum_test::TestServer;
use defi_hot_wallet::{
    api::server::WalletServer,
    api::types::BridgeAssetsRequest,
    api::types::ErrorResponse,
    core::config::{StorageConfig, WalletConfig},
};
use futures::future::join_all;
use serde_json::json;
use serde_json::Value;
use std::sync::Arc;
use uuid::Uuid;

/// Helper function to create a test server with an in-memory database.
async fn setup_test_server() -> TestServer {
    let config = WalletConfig {
        storage: StorageConfig {
            database_url: "sqlite::memory:".to_string(),
            ..Default::default()
        },
        ..Default::default()
    };
    let server = WalletServer::new("127.0.0.1".to_string(), 0, config, None).await.unwrap();
    TestServer::new(server.create_router().await).unwrap()
}

/// Same as `setup_test_server` but allows providing an API key (Some) to exercise auth branches.
async fn setup_test_server_with_key(api_key: Option<String>) -> TestServer {
    let config = WalletConfig {
        storage: StorageConfig {
            database_url: "sqlite::memory:".to_string(),
            ..Default::default()
        },
        ..Default::default()
    };
    let server = WalletServer::new("127.0.0.1".to_string(), 0, config, api_key).await.unwrap();
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

// ---------------------------------------------------------------------------
// Additional exhaustive tests for bridge_assets handler covering every branch
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "current_thread")]
async fn test_bridge_empty_parameters_each_field() {
    let base = BridgeAssetsRequest {
        from_wallet: "w".to_string(),
        from_chain: "eth".to_string(),
        to_chain: "solana".to_string(),
        token: "USDC".to_string(),
        amount: "1.0".to_string(),
    };

    // Create server once and reuse to avoid repeated expensive setup.
    let server = setup_test_server().await;

    // For each field, create a request with that single field empty and assert Invalid parameters
    let fields = vec!["from_wallet", "from_chain", "to_chain", "token", "amount"];
    for field in fields {
        let mut req = base.clone();
        match field {
            "from_wallet" => req.from_wallet = String::new(),
            "from_chain" => req.from_chain = String::new(),
            "to_chain" => req.to_chain = String::new(),
            "token" => req.token = String::new(),
            "amount" => req.amount = String::new(),
            _ => {}
        }

        let response = server.post("/api/bridge").json(&req).await;
        response.assert_status(StatusCode::BAD_REQUEST);
        let body: ErrorResponse = response.json();
        assert_eq!(body.error, "Invalid parameters");
    }
}

#[tokio::test(flavor = "current_thread")]
async fn test_bridge_invalid_amount_non_numeric_and_negative() {
    // non-numeric
    let req = BridgeAssetsRequest {
        from_wallet: "test_wallet".to_string(),
        from_chain: "eth".to_string(),
        to_chain: "solana".to_string(),
        token: "USDC".to_string(),
        amount: "abc".to_string(),
    };
    let server = setup_test_server().await;
    let res = server.post("/api/bridge").json(&req).await;
    res.assert_status(StatusCode::BAD_REQUEST);
    let body: ErrorResponse = res.json();
    assert_eq!(body.error, "Invalid amount");

    // negative amount
    let req2 = BridgeAssetsRequest { amount: "-5.0".to_string(), ..req };
    let res2 = server.post("/api/bridge").json(&req2).await;
    res2.assert_status(StatusCode::BAD_REQUEST);
    let body2: ErrorResponse = res2.json();
    assert_eq!(body2.error, "Invalid amount");
}

#[tokio::test(flavor = "current_thread")]
async fn test_bridge_wallet_lifecycle_and_success() {
    // Create a wallet via the API then call /api/bridge to get success branch
    let server = setup_test_server().await;

    let wallet_name = format!("ok_{}", Uuid::new_v4().simple());
    // create wallet using raw json to avoid importing CreateWalletRequest
    let create_res = server
        .post("/api/wallets")
        .json(&json!({ "name": wallet_name, "quantum_safe": false }))
        .await;
    create_res.assert_status_ok();

    // Now bridge
    let req = BridgeAssetsRequest {
        from_wallet: wallet_name.clone(),
        from_chain: "eth".to_string(),
        to_chain: "solana".to_string(),
        token: "USDC".to_string(),
        amount: "10.0".to_string(),
    };

    let res = server.post("/api/bridge").json(&req).await;
    res.assert_status_ok();
    // Deserialize bridge response produced by server.rs
    let body: serde_json::Value = res.json();
    assert_eq!(body["bridge_tx_id"], serde_json::Value::String("mock_bridge_tx_hash".to_string()));
}

#[tokio::test(flavor = "current_thread")]
async fn test_bridge_unauthorized_when_api_key_set() {
    // create server with an API key set -> requests without Authorization should 401
    let server = setup_test_server_with_key(Some("secret-key".to_string())).await;

    let req = BridgeAssetsRequest {
        from_wallet: "any".to_string(),
        from_chain: "eth".to_string(),
        to_chain: "solana".to_string(),
        token: "USDC".to_string(),
        amount: "1.0".to_string(),
    };

    let res = server.post("/api/bridge").json(&req).await;
    res.assert_status(StatusCode::UNAUTHORIZED);
    let body: ErrorResponse = res.json();
    assert_eq!(body.error, "Unauthorized");
}

#[tokio::test(flavor = "current_thread")]
async fn test_bridge_concurrent_requests() {
    let server = setup_test_server().await;
    let wallet_name = format!("concurrent_{}", Uuid::new_v4().simple());
    let create =
        server.post("/api/wallets").json(&json!({ "name": wallet_name, "quantum_safe": false }));
    create.await.assert_status_ok();

    let req = BridgeAssetsRequest {
        from_wallet: wallet_name.clone(),
        from_chain: "eth".to_string(),
        to_chain: "solana".to_string(),
        token: "USDC".to_string(),
        amount: "2.0".to_string(),
    };

    // Fire 4 concurrent bridge requests (reduced from 8) to reduce contention and test time.
    let server = Arc::new(server);
    let futs: Vec<_> = (0..4)
        .map(|_| {
            let srv = server.clone();
            let body = req.clone();
            async move { srv.post("/api/bridge").json(&body).await }
        })
        .collect();

    let results = join_all(futs).await;
    for r in results {
        r.assert_status_ok();
        let body: serde_json::Value = r.json();
        assert_eq!(
            body["bridge_tx_id"],
            serde_json::Value::String("mock_bridge_tx_hash".to_string())
        );
    }
}

#[tokio::test(flavor = "current_thread")]
async fn test_bridge_assets_handler_invalid_amount() {
    let request = BridgeAssetsRequest {
        from_wallet: "test_wallet".to_string(),
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
async fn test_bridge_assets_handler_wallet_not_found() {
    let request = BridgeAssetsRequest {
        from_wallet: "nonexistent_wallet".to_string(),
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

    let request = BridgeAssetsRequest {
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
    // Intentionally use an unsupported 'from_chain' value
    let request = BridgeAssetsRequest {
        from_wallet: wallet_name,
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
// ...existing code...
