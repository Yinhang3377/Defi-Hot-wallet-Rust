use axum::http::StatusCode;
use axum_test::{TestServer, TestServerConfig};
use defi_hot_wallet::api::server::WalletServer;
use defi_hot_wallet::core::config::{BlockchainConfig, StorageConfig, WalletConfig};
use serde_json::{json, Value};
use tokio;

/// 创建测试配置，使用内存数据库
fn create_test_config() -> WalletConfig {
    WalletConfig {
        storage: StorageConfig {
            database_url: "sqlite::memory:".to_string(), // 使用内存数据库以避免文件系统问题
            max_connections: Some(1),
            connection_timeout_seconds: Some(5),
        },
        blockchain: BlockchainConfig {
            networks: WalletConfig::default().blockchain.networks, // 保留默认网络配置
            default_network: Some("eth".to_string()),
        },
        quantum_safe: false,
        multi_sig_threshold: 2,
    }
}

/// 辅助函数：设置并返回一个测试服务器实例
async fn setup_test_server() -> TestServer {
    let config = create_test_config();
    let server = WalletServer::new("127.0.0.1".to_string(), 0, config, None)
        .await
        .expect("Failed to create server");
    let app = server.create_router().await;
    let config = TestServerConfig::default();
    TestServer::new_with_config(app, config).unwrap()
}

/// 辅助函数：在测试服务器上创建一个钱包以供测试使用
async fn create_test_wallet(server: &TestServer, name: &str) -> String {
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

    // 假设的桥接端点和载荷
    let response = server
        .post("/api/bridge")
        .json(&json!({
            "from_wallet": wallet_name, // 修正：字段应为 from_wallet
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
            "from_wallet": wallet_name, // 修正：字段应为 from_wallet
            "from_chain": "invalid_chain", // 无效的源链
            "to_chain": "solana",
            "token": "USDC",
            "amount": "100"
        }))
        .await;

    // 期望一个客户端错误（例如 400 Bad Request）
    response.assert_status(StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_bridge_zero_amount() {
    let server = setup_test_server().await;
    let wallet_name = "bridge_wallet_zero_amount";
    let _wallet_id = create_test_wallet(&server, wallet_name).await;

    let response = server
        .post("/api/bridge")
        .json(&json!({ "from_wallet": wallet_name, "from_chain": "eth", "to_chain": "solana", "token": "USDC", "amount": "0" })) // 修正：字段应为 from_wallet
        .await;

    // 零金额或无效金额应导致客户端错误
    response.assert_status(StatusCode::BAD_REQUEST);
}
