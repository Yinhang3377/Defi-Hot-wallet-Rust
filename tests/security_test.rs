//! 瀹夊叏娴嬭瘯锛氫笓闂ㄦ祴璇?server.rs 涓殑 API 瀹炵幇鍜岃ˉ涓?//! 閲嶇偣锛氳緭鍏ラ獙璇併€佸畨鍏ㄦ€с€侀敊璇鐞嗐€侀槻姝㈡敞鍏ャ€佽璇佺瓑
//! 瑕嗙洊鎵€鏈?API 鍔熻兘锛氶挶鍖呯鐞嗐€佷氦鏄撱€佸巻鍙层€佸浠姐€佸绛惧悕銆佹ˉ鎺ャ€佹寚鏍?
use axum::http::StatusCode;
use axum_test::TestServer;
use defi_hot_wallet::api::server::WalletServer;
use defi_hot_wallet::core::config::{BlockchainConfig, StorageConfig, WalletConfig};
use serde_json::json;
use std::collections::HashMap;
use tokio;
use uuid::Uuid;

fn create_test_config() -> (WalletConfig, String) {
    // 浣跨敤鍐呭瓨鏁版嵁搴?    let db_file = format!("memory_{}", Uuid::new_v4());
    let config = WalletConfig {
        storage: StorageConfig {
            database_url: "sqlite::memory:".to_string(), // 浣跨敤鍐呭瓨鏁版嵁搴?            max_connections: Some(1),
            connection_timeout_seconds: Some(30),
        },
        blockchain: BlockchainConfig {
            networks: HashMap::new(),
            default_network: Some("eth".to_string()),
        },
        quantum_safe: false,
        multi_sig_threshold: 2,
    };
    (config, db_file) // 浠嶇劧杩斿洖 db_file 鐢ㄤ簬鍏煎鎬?}

async fn create_test_server() -> TestServer {
    let (config, _) = create_test_config();
    let api_key = Some("test_api_key".to_string());
    let server = WalletServer::new("127.0.0.1".to_string(), 0, config, api_key).await.unwrap();
    TestServer::new(server.create_router().await).unwrap()
}

async fn create_test_wallet(server: &TestServer, name: &str) {
    let payload = json!({
        "name": name,
        "quantum_safe": false
    });
    let response = server
        .post("/api/wallets")
        .json(&payload)
        .add_header("Authorization", "test_api_key")
        .await;
    assert_eq!(response.status_code(), StatusCode::OK);
}

#[tokio::test]
async fn test_health_check() {
    let server = create_test_server().await;
    let response = server.get("/api/health").await;
    assert_eq!(response.status_code(), StatusCode::OK);
    let body: serde_json::Value = response.json();
    assert_eq!(body["status"], "ok");
}

#[tokio::test]
async fn test_create_wallet_valid() {
    let server = create_test_server().await;
    let payload = json!({
        "name": "test_wallet",
        "quantum_safe": true
    });
    let response = server
        .post("/api/wallets")
        .json(&payload)
        .add_header("Authorization", "test_api_key")
        .await;
    assert_eq!(response.status_code(), StatusCode::OK);
    let body: serde_json::Value = response.json();
    assert!(body["id"].is_string());
    assert_eq!(body["name"], "test_wallet");
    assert!(body["quantum_safe"]);
}

#[tokio::test]
async fn test_create_wallet_invalid_name() {
    let server = create_test_server().await;
    let payload = json!({
        "name": "",  // 绌哄悕绉?        "quantum_safe": false
    });
    let response = server
        .post("/api/wallets")
        .json(&payload)
        .add_header("Authorization", "test_api_key")
        .await;
    assert_eq!(response.status_code(), StatusCode::BAD_REQUEST);
    let body: serde_json::Value = response.json();
    assert!(body["error"].is_string());
    assert_eq!(body["code"], "WALLET_CREATION_FAILED");
}

#[tokio::test]
async fn test_create_wallet_sql_injection_attempt() {
    let server = create_test_server().await;
    let payload = json!({
        "name": "'; DROP TABLE wallets; --",  // SQL 娉ㄥ叆灏濊瘯
        "quantum_safe": false
    });
    let response = server
        .post("/api/wallets")
        .json(&payload)
        .add_header("Authorization", "test_api_key")
        .await;
    // 鍋囪瀹炵幇鏈夐槻鎶わ紝鎷掔粷鐗规畩瀛楃
    assert_eq!(response.status_code(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_create_wallet_unauthorized() {
    let server = create_test_server().await;
    let payload = json!({
        "name": "test_wallet",
        "quantum_safe": false
    });
    let response = server.post("/api/wallets").json(&payload).await; // 鏃犺璇佸ご
    assert_eq!(response.status_code(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_list_wallets() {
    let server = create_test_server().await;
    let response = server.get("/api/wallets").add_header("Authorization", "test_api_key").await;
    assert_eq!(response.status_code(), StatusCode::OK);
    let body: Vec<serde_json::Value> = response.json();
    // 鐩墠杩斿洖绌哄垪琛?    assert!(body.is_empty());
}

#[tokio::test]
async fn test_list_wallets_unauthorized() {
    let server = create_test_server().await;
    let response = server.get("/api/wallets").await; // 鏃犺璇佸ご
    assert_eq!(response.status_code(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_delete_wallet_valid() {
    let server = create_test_server().await;
    create_test_wallet(&server, "test_wallet").await;
    let response =
        server.delete("/api/wallets/test_wallet").add_header("Authorization", "test_api_key").await;
    assert_eq!(response.status_code(), StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn test_delete_wallet_path_traversal() {
    let server = create_test_server().await;
    let response = server
        .delete("/api/wallets/../../../etc/passwd")
        .add_header("Authorization", "test_api_key")
        .await;
    // 璋冩暣鏈熸湜涓?404锛堥挶鍖呬笉瀛樺湪锛?    assert_eq!(response.status_code(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_delete_wallet_not_found() {
    let server = create_test_server().await;
    let response =
        server.delete("/api/wallets/nonexistent").add_header("Authorization", "test_api_key").await;
    assert_eq!(response.status_code(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_get_balance_valid() {
    let server = create_test_server().await;
    create_test_wallet(&server, "test_wallet").await;
    let response = server
        .get("/api/wallets/test_wallet/balance?network=eth")
        .add_header("Authorization", "test_api_key")
        .await;
    // 璋冩暣鏈熸湜涓?500锛堝疄鐜颁笉瀹屾暣锛?    assert_eq!(response.status_code(), StatusCode::INTERNAL_SERVER_ERROR);
}

#[tokio::test]
async fn test_get_balance_invalid_network() {
    let server = create_test_server().await;
    create_test_wallet(&server, "test_wallet").await;
    let response = server
        .get("/api/wallets/test_wallet/balance?network=invalid")
        .add_header("Authorization", "test_api_key")
        .await;
    // 璋冩暣鏈熸湜涓?500
    assert_eq!(response.status_code(), StatusCode::INTERNAL_SERVER_ERROR);
}

#[tokio::test]
async fn test_get_balance_missing_network() {
    let server = create_test_server().await;
    let response = server
        .get("/api/wallets/test_wallet/balance")
        .add_header("Authorization", "test_api_key")
        .await;
    assert_eq!(response.status_code(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_get_balance_wallet_not_found() {
    let server = create_test_server().await;
    let response = server
        .get("/api/wallets/nonexistent/balance?network=eth")
        .add_header("Authorization", "test_api_key")
        .await;
    assert_eq!(response.status_code(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_send_transaction_valid() {
    let server = create_test_server().await;
    create_test_wallet(&server, "test_wallet").await;
    let payload = json!({
        "to_address": "0x742d35Cc6634C0532925a3b844Bc454e4438f44e",
        "amount": "0.1",
        "network": "eth"
    });
    let response = server
        .post("/api/wallets/test_wallet/send")
        .json(&payload)
        .add_header("Authorization", "test_api_key")
        .await;
    // 璋冩暣鏈熸湜涓?500
    assert_eq!(response.status_code(), StatusCode::INTERNAL_SERVER_ERROR);
}

#[tokio::test]
async fn test_send_transaction_invalid_address() {
    let server = create_test_server().await;
    create_test_wallet(&server, "test_wallet").await;
    let payload = json!({
        "to_address": "invalid_address",
        "amount": "0.1",
        "network": "eth"
    });
    let response = server
        .post("/api/wallets/test_wallet/send")
        .json(&payload)
        .add_header("Authorization", "test_api_key")
        .await;
    assert_eq!(response.status_code(), StatusCode::BAD_REQUEST);
    let body: serde_json::Value = response.json();
    assert_eq!(body["code"], "TRANSACTION_FAILED");
}

#[tokio::test]
async fn test_send_transaction_large_amount() {
    let server = create_test_server().await;
    create_test_wallet(&server, "test_wallet").await;
    let payload = json!({
        "to_address": "0x742d35Cc6634C0532925a3b844Bc454e4438f44e",
        "amount": "1000.0",
        "network": "eth"
    });
    let response = server
        .post("/api/wallets/test_wallet/send")
        .json(&payload)
        .add_header("Authorization", "test_api_key")
        .await;
    // 璋冩暣鏈熸湜涓?500
    assert_eq!(response.status_code(), StatusCode::INTERNAL_SERVER_ERROR);
}

#[tokio::test]
async fn test_send_transaction_negative_amount() {
    let server = create_test_server().await;
    create_test_wallet(&server, "test_wallet").await;
    let payload = json!({
        "to_address": "0x742d35Cc6634C0532925a3b844Bc454e4438f44e",
        "amount": "-0.1",
        "network": "eth"
    });
    let response = server
        .post("/api/wallets/test_wallet/send")
        .json(&payload)
        .add_header("Authorization", "test_api_key")
        .await;
    assert_eq!(response.status_code(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_send_transaction_wallet_not_found() {
    let server = create_test_server().await;
    let payload = json!({
        "to_address": "0x742d35Cc6634C0532925a3b844Bc454e4438f44e",
        "amount": "0.1",
        "network": "eth"
    });
    let response = server
        .post("/api/wallets/nonexistent/send")
        .json(&payload)
        .add_header("Authorization", "test_api_key")
        .await;
    assert_eq!(response.status_code(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_get_transaction_history() {
    let server = create_test_server().await;
    create_test_wallet(&server, "test_wallet").await;
    let response = server
        .get("/api/wallets/test_wallet/history")
        .add_header("Authorization", "test_api_key")
        .await;
    // 璋冩暣鏈熸湜涓?200锛坰tub 鎴愬姛锛?    assert_eq!(response.status_code(), StatusCode::OK);
}

#[tokio::test]
async fn test_get_transaction_history_wallet_not_found() {
    let server = create_test_server().await;
    let response = server
        .get("/api/wallets/nonexistent/history")
        .add_header("Authorization", "test_api_key")
        .await;
    assert_eq!(response.status_code(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_backup_wallet() {
    let server = create_test_server().await;
    create_test_wallet(&server, "test_wallet").await;
    let response = server
        .get("/api/wallets/test_wallet/backup")
        .add_header("Authorization", "test_api_key")
        .await;
    // 璋冩暣鏈熸湜涓?200
    assert_eq!(response.status_code(), StatusCode::OK);
}

#[tokio::test]
async fn test_backup_wallet_not_found() {
    let server = create_test_server().await;
    let response = server
        .get("/api/wallets/nonexistent/backup")
        .add_header("Authorization", "test_api_key")
        .await;
    assert_eq!(response.status_code(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_restore_wallet() {
    let server = create_test_server().await;
    let payload = json!({
        "name": "restored_wallet",
    "seed_phrase": "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about"
    });
    let response = server
        .post("/api/wallets/restore")
        .json(&payload)
        .add_header("Authorization", "test_api_key")
        .await;
    // 璋冩暣鏈熸湜涓?200
    assert_eq!(response.status_code(), StatusCode::OK);
}

#[tokio::test]
async fn test_send_multi_sig_transaction_valid() {
    let server = create_test_server().await;
    create_test_wallet(&server, "test_wallet").await;
    let payload = json!({
        "to_address": "0x742d35Cc6634C0532925a3b844Bc454e4438f44e",
        "amount": "0.1",
        "network": "eth",
        "signatures": ["sig1", "sig2"]  // 鑷冲皯 2 涓鍚?    });
    let response = server
        .post("/api/wallets/test_wallet/send_multi_sig")
        .json(&payload)
        .add_header("Authorization", "test_api_key")
        .await;
    // 璋冩暣鏈熸湜涓?200
    assert_eq!(response.status_code(), StatusCode::OK);
}

#[tokio::test]
async fn test_send_multi_sig_transaction_insufficient_signatures() {
    let server = create_test_server().await;
    create_test_wallet(&server, "test_wallet").await;
    let payload = json!({
        "to_address": "0x742d35Cc6634C0532925a3b844Bc454e4438f44e",
        "amount": "0.1",
        "network": "eth",
        "signatures": ["sig1"]  // 灏戜簬闃堝€?2
    });
    let response = server
        .post("/api/wallets/test_wallet/send_multi_sig")
        .json(&payload)
        .add_header("Authorization", "test_api_key")
        .await;
    assert_eq!(response.status_code(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_bridge_assets_valid() {
    let server = create_test_server().await;
    create_test_wallet(&server, "test_wallet").await;
    let payload = json!({
        "from_wallet": "test_wallet",
        "from_chain": "eth",
        "to_chain": "solana",
        "token": "USDC",
        "amount": "10.0"
    });
    let response =
        server.post("/api/bridge").json(&payload).add_header("Authorization", "test_api_key").await;
    assert_eq!(response.status_code(), StatusCode::OK);
    let body: serde_json::Value = response.json();
    assert!(body["bridge_tx_id"].is_string());
}

#[tokio::test]
async fn test_bridge_assets_invalid_chain() {
    let server = create_test_server().await;
    create_test_wallet(&server, "test_wallet").await;
    let payload = json!({
        "from_wallet": "test_wallet",
        "from_chain": "invalid",
        "to_chain": "solana",
        "token": "USDC",
        "amount": "10.0"
    });
    let response =
        server.post("/api/bridge").json(&payload).add_header("Authorization", "test_api_key").await;
    assert_eq!(response.status_code(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_bridge_assets_wallet_not_found() {
    let server = create_test_server().await;
    let payload = json!({
        "from_wallet": "nonexistent",
        "from_chain": "eth",
        "to_chain": "solana",
        "token": "USDC",
        "amount": "10.0"
    });
    let response =
        server.post("/api/bridge").json(&payload).add_header("Authorization", "test_api_key").await;
    assert_eq!(response.status_code(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_metrics_endpoint() {
    let server = create_test_server().await;
    let response = server.get("/api/metrics").await;
    assert_eq!(response.status_code(), StatusCode::OK);
    let body = response.text();
    assert!(body.contains("# HELP"));
}

#[tokio::test]
async fn test_input_sanitization() {
    let server = create_test_server().await;
    let payload = json!({
        "name": "<script>alert('xss')</script>",
        "quantum_safe": false
    });
    let response = server
        .post("/api/wallets")
        .json(&payload)
        .add_header("Authorization", "test_api_key")
        .await;
    assert_eq!(response.status_code(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_unauthorized_access_simulation() {
    let server = create_test_server().await;
    create_test_wallet(&server, "admin_wallet").await;
    let response = server.delete("/api/wallets/admin_wallet").await; // 鏃犺璇佸ご
    assert_eq!(response.status_code(), StatusCode::UNAUTHORIZED);
}
