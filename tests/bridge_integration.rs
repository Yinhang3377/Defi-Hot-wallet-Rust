use axum::http::StatusCode;
use axum_test::{TestServer, TestServerConfig};
use defi_hot_wallet::api::server::WalletServer;
use defi_hot_wallet::core::config::{BlockchainConfig, StorageConfig, WalletConfig};
use serde_json::{json, Value};
use tokio;

/// 鍒涘缓娴嬭瘯閰嶇疆锛屼娇鐢ㄥ唴瀛樻暟鎹簱
fn create_test_config() -> WalletConfig {
    WalletConfig {
        storage: StorageConfig {
            database_url: "sqlite::memory:".to_string(), // 浣跨敤鍐呭瓨鏁版嵁搴撲互閬垮厤鏂囦欢绯荤粺闂
            max_connections: Some(1),
            connection_timeout_seconds: Some(5),
        },
        blockchain: BlockchainConfig {
            networks: WalletConfig::default().blockchain.networks, // 淇濈暀榛樿缃戠粶閰嶇疆
            default_network: Some("eth".to_string()),
        },
        quantum_safe: false,
        multi_sig_threshold: 2,
    }
}

/// 杈呭姪鍑芥暟锛氳缃苟杩斿洖涓€涓祴璇曟湇鍔″櫒瀹炰緥
async fn setup_test_server() -> TestServer {
    let config = create_test_config();
    let server = WalletServer::new("127.0.0.1".to_string(), 0, config, None)
        .await
        .expect("Failed to create server");
    let app = server.create_router().await;
    let config = TestServerConfig::default();
    TestServer::new_with_config(app, config).unwrap()
}

/// 杈呭姪鍑芥暟锛氬湪娴嬭瘯鏈嶅姟鍣ㄤ笂鍒涘缓涓€涓挶鍖呬互渚涙祴璇曚娇鐢?async fn create_test_wallet(server: &TestServer, name: &str) -> String {
    let response = server
        .post("/api/wallets")
        .json(&json!({
            "name": name,
            "quantum_safe": false
        }))
        .await;
    response.assert_status_ok();
    let body: Value = response.json();
    body["id"].as_str().unwrap().to_string()
}

#[tokio::test]
async fn test_bridge_transfer() {
    let server = setup_test_server().await;
    let wallet_name = "bridge_wallet_ok";
    let _wallet_id = create_test_wallet(&server, wallet_name).await;

    // 鍋囪鐨勬ˉ鎺ョ鐐瑰拰杞借嵎
    let response = server
        .post("/api/bridge")
        .json(&json!({
            "from_wallet": wallet_name, // 淇锛氬瓧娈靛簲涓?from_wallet
            "from_chain": "eth",
            "to_chain": "solana",
            "token": "USDC",
            "amount": "100"
        }))
        .await;

    // The mock bridge handler doesn't check for wallet existence, so it should succeed.
    response.assert_status_ok();
    let body: Value = response.json();
    assert!(body["bridge_tx_id"].is_string());
}

#[tokio::test]
async fn test_bridge_invalid_chain() {
    let server = setup_test_server().await;
    let wallet_name = "bridge_wallet_invalid_chain";
    let _wallet_id = create_test_wallet(&server, wallet_name).await;

    let response = server
        .post("/api/bridge")
        .json(&json!({
            "from_wallet": wallet_name, // 淇锛氬瓧娈靛簲涓?from_wallet
            "from_chain": "invalid_chain", // 鏃犳晥鐨勬簮閾?            "to_chain": "solana",
            "token": "USDC",
            "amount": "100"
        }))
        .await;

    // 鏈熸湜涓€涓鎴风閿欒锛堜緥濡?400 Bad Request锛?    response.assert_status(StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_bridge_zero_amount() {
    let server = setup_test_server().await;
    let wallet_name = "bridge_wallet_zero_amount";
    let _wallet_id = create_test_wallet(&server, wallet_name).await;

    let response = server
        .post("/api/bridge")
        .json(&json!({ "from_wallet": wallet_name, "from_chain": "eth", "to_chain": "solana", "token": "USDC", "amount": "0" })) // 淇锛氬瓧娈靛簲涓?from_wallet
        .await;

    // 闆堕噾棰濇垨鏃犳晥閲戦搴斿鑷村鎴风閿欒
    response.assert_status(StatusCode::BAD_REQUEST);
}
