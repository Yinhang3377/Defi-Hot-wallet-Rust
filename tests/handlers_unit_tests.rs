use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use serde_json::Value;
use std::sync::Arc;

use defi_hot_wallet::api::handlers::{bridge_assets, health_check, metrics_handler};
use defi_hot_wallet::api::types::BridgeAssetsRequest;
use defi_hot_wallet::core::config::{StorageConfig, WalletConfig};
use defi_hot_wallet::core::wallet_manager::WalletManager;

#[tokio::test(flavor = "current_thread")]
async fn handlers_health_and_metrics() {
    // health_check()
    let h = health_check().await;
    let body: Value = h.0;
    assert_eq!(body["status"], "ok");
    assert!(body["version"].is_string());
    assert!(body["timestamp"].is_string());

    // metrics_handler()
    let m = metrics_handler().await;
    assert!(m.contains("defi_hot_wallet_requests_total"));
}

#[tokio::test(flavor = "current_thread")]
async fn handlers_bridge_assets_branches() {
    // prepare a WalletManager with in-memory sqlite
    let config = WalletConfig {
        storage: StorageConfig {
            database_url: "sqlite::memory:".to_string(),
            ..Default::default()
        },
        ..Default::default()
    };
    let wm = WalletManager::new(&config).await.expect("wallet manager init");
    let state = State(Arc::new(wm));

    // empty parameters -> Invalid parameters
    let req = BridgeAssetsRequest {
        from_wallet: "".to_string(),
        from_chain: "eth".to_string(),
        to_chain: "solana".to_string(),
        token: "USDC".to_string(),
        amount: "1.0".to_string(),
    };
    let res = bridge_assets(state.clone(), Json(req)).await;
    assert!(res.is_err());
    let (code, body) = res.err().unwrap();
    assert_eq!(code, StatusCode::BAD_REQUEST);
    assert_eq!(body.0.error, "Invalid parameters");

    // invalid amount (non-numeric)
    let req2 = BridgeAssetsRequest {
        from_wallet: "w".to_string(),
        from_chain: "eth".to_string(),
        to_chain: "solana".to_string(),
        token: "USDC".to_string(),
        amount: "abc".to_string(),
    };
    let res2 = bridge_assets(state.clone(), Json(req2)).await;
    assert!(res2.is_err());
    let (code2, body2) = res2.err().unwrap();
    assert_eq!(code2, StatusCode::BAD_REQUEST);
    assert_eq!(body2.0.error, "Invalid amount");

    // unsupported chain
    let req3 = BridgeAssetsRequest {
        from_wallet: "w".to_string(),
        from_chain: "invalid_chain".to_string(),
        to_chain: "solana".to_string(),
        token: "USDC".to_string(),
        amount: "1.0".to_string(),
    };
    let res3 = bridge_assets(state.clone(), Json(req3)).await;
    assert!(res3.is_err());
    let (code3, body3) = res3.err().unwrap();
    assert_eq!(code3, StatusCode::BAD_REQUEST);
    assert_eq!(body3.0.error, "Unsupported chain");

    // success path: create wallet first then call
    let wm_arc = state.0.clone();
    wm_arc.create_wallet("test-w", false).await.expect("create wallet");

    let req4 = BridgeAssetsRequest {
        from_wallet: "test-w".to_string(),
        from_chain: "eth".to_string(),
        to_chain: "solana".to_string(),
        token: "USDC".to_string(),
        amount: "1.0".to_string(),
    };

    let res4 = bridge_assets(state, Json(req4)).await;
    assert!(res4.is_ok());
    let br = res4.ok().unwrap().0;
    assert_eq!(br.bridge_tx_id, "mock_bridge_tx_hash");
}
