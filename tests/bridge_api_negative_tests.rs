use axum::http::StatusCode;
use axum_test::TestServer;
use defi_hot_wallet::api::server::WalletServer;
use defi_hot_wallet::core::config::{BlockchainConfig, NetworkConfig, StorageConfig, WalletConfig};
use serde_json::{json, Value};
use std::collections::HashMap;

// 统一设置测试用环境变量：固定主密钥 + 共享内存库
fn set_test_env() {
    std::env::set_var(
        "WALLET_ENC_KEY",
        "000102030405060708090a0b0c0d0e0f000102030405060708090a0b0c0d0e0f",
    );
    std::env::set_var("DATABASE_URL", "sqlite://:memory:?cache=shared");
}

fn create_test_config() -> WalletConfig {
    WalletConfig {
        storage: StorageConfig {
            database_url: "sqlite://:memory:?cache=shared".to_string(),
            max_connections: Some(1),
            connection_timeout_seconds: Some(30),
        },
        blockchain: BlockchainConfig {
            networks: {
                let mut map = HashMap::new();
                map.insert(
                    "eth".to_string(),
                    NetworkConfig {
                        rpc_url: "http://localhost:8545".to_string(),
                        chain_id: Some(1),
                        native_token: "ETH".to_string(),
                        block_time_seconds: 12,
                    },
                );
                map.insert(
                    "solana".to_string(),
                    NetworkConfig {
                        rpc_url: "http://localhost:8899".to_string(),
                        chain_id: None,
                        native_token: "SOL".to_string(),
                        block_time_seconds: 1,
                    },
                );
                map
            },
            default_network: Some("eth".to_string()),
        },
        quantum_safe: false,
        multi_sig_threshold: 2,
    }
}

async fn create_test_server() -> TestServer {
    set_test_env();
    let config = create_test_config();
    let api_key = Some("test_api_key".to_string());
    let test_master_key = Some(vec![0u8; 32]); // 确定性 32 字节主密钥
    let server =
        WalletServer::new_for_test("127.0.0.1".to_string(), 0, config, api_key, test_master_key)
            .await
            .expect("server boot");
    TestServer::new(server.create_router().await).unwrap()
}

#[tokio::test]
async fn test_bridge_empty_parameters_each_field() {
    let server = create_test_server().await;

    let base = json!({
        "from_wallet": "w",
        "from_chain": "eth",
        "to_chain": "solana",
        "token": "USDC",
        "amount": "1.0"
    });

    let cases = vec![
        ("from_wallet", json!("")),
        ("from_chain", json!("")),
        ("to_chain", json!("")),
        ("token", json!("")),
        ("amount", json!("")),
    ];

    for (k, v) in cases {
        let mut p = base.clone();
        p.as_object_mut().unwrap().insert(k.to_string(), v);
        let r =
            server.post("/api/bridge").json(&p).add_header("Authorization", "test_api_key").await;
        assert_eq!(r.status_code(), StatusCode::BAD_REQUEST, "failed on field {}", k);
        let e: Value = r.json();
        assert_eq!(e["error"], "Invalid parameters", "failed on field {}", k);
    }
}

#[tokio::test]
async fn test_bridge_amount_zero_and_non_numeric() {
    let server = create_test_server().await;

    // zero amount
    let p0 = json!({
        "from_wallet": "w",
        "from_chain": "eth",
        "to_chain": "solana",
        "token": "USDC",
        "amount": "0"
    });
    let r0 = server.post("/api/bridge").json(&p0).add_header("Authorization", "test_api_key").await;
    assert_eq!(r0.status_code(), StatusCode::BAD_REQUEST);
    let e0: Value = r0.json();
    assert_eq!(e0["error"], "Invalid amount");

    // non-numeric amount
    let p1 = json!({
        "from_wallet": "w",
        "from_chain": "eth",
        "to_chain": "solana",
        "token": "USDC",
        "amount": "abc"
    });
    let r1 = server.post("/api/bridge").json(&p1).add_header("Authorization", "test_api_key").await;
    assert_eq!(r1.status_code(), StatusCode::BAD_REQUEST);
    let e1: Value = r1.json();
    assert_eq!(e1["error"], "Invalid amount");
}

#[tokio::test]
async fn test_bridge_unauthorized_when_api_key_set() {
    let server = create_test_server().await;
    let payload = json!({
        "from_wallet": "w",
        "from_chain": "eth",
        "to_chain": "solana",
        "token": "USDC",
        "amount": "1.0"
    });

    let r = server.post("/api/bridge").json(&payload).await;
    assert_eq!(r.status_code(), StatusCode::UNAUTHORIZED);
    let e: Value = r.json();
    assert_eq!(e["error"], "Unauthorized");
}

#[tokio::test]
async fn test_bridge_wallet_not_found_for_valid_request() {
    let server = create_test_server().await;

    let payload = json!({
        "from_wallet": "noexist_wallet_xxx",
        "from_chain": "eth",
        "to_chain": "solana",
        "token": "USDC",
        "amount": "1.0"
    });

    let r =
        server.post("/api/bridge").json(&payload).add_header("Authorization", "test_api_key").await;
    assert_eq!(r.status_code(), StatusCode::NOT_FOUND);
    let e: Value = r.json();
    assert_eq!(e["error"], "Wallet not found");
}

#[tokio::test]
async fn test_bridge_unsupported_chain() {
    let server = create_test_server().await;
    let payload = json!({
        "from_wallet": "w",
        "from_chain": "btc",
        "to_chain": "solana",
        "token": "USDC",
        "amount": "1.0"
    });
    let r =
        server.post("/api/bridge").json(&payload).add_header("Authorization", "test_api_key").await;
    assert_eq!(r.status_code(), StatusCode::BAD_REQUEST, "body: {}", r.text());
    let e: Value = r.json();
    assert_eq!(e["error"], "Unsupported chain");
}
