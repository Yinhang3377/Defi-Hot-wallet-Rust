//! tests/network_tests.rs
//!
//! 测试 `src/network/node_manager.rs` 的功能。
//! 覆盖：
//! - `select_node` 占位函数
//! - `NodeManager::new_infura` 构造函数
//! - `NodeManager::send_tx` 的成功和失败路径

use defi_hot_wallet::core::domain::{Tx, Wallet};
use defi_hot_wallet::network::node_manager::{select_node, NodeManager};
use httpmock::{Method, MockServer};
use serde_json::json;

#[test]
fn test_select_node_placeholder() {
    // 正常路径：测试占位函数是否返回预期的 URL
    let node_url = select_node();
    assert!(node_url.is_some());
    assert!(node_url.unwrap().contains("infura.io"));
}

#[tokio::test]
async fn test_node_manager_new_infura() {
    // 正常路径：测试构造函数
    let project_id = "test_project_id";
    let _manager = NodeManager::new_infura(project_id);
    // 仅验证构造函数是否成功，因为内部字段是私有的。
    // 可以在 NodeManager 中添加一个公共的 getter 方法来进一步测试 rpc_url。
}

#[tokio::test]
async fn test_send_tx_success() {
    // 正常路径：模拟成功的 RPC 调用
    let server = MockServer::start();

    let mock_tx_hash = "0xdeadbeefcafebabefeedface0000000000000000000000000000000000000000";

    let mock = server.mock(|when, then| {
        when.method(Method::POST)
            .path("/") // JSON-RPC endpoint
            .header("content-type", "application/json");
        then.status(200)
            .json_body(json!({
                "jsonrpc": "2.0",
                "id": 1,
                "result": mock_tx_hash
            }));
    });

    // 增加调试日志
    println!("Mock server is running at: {}", server.base_url());
 
    // 使用模拟服务器的 URL 创建 manager
    let manager = NodeManager::new(&server.base_url());
    let wallet = Wallet::from_mnemonic("test").unwrap();
    let tx = Tx::new(&wallet, "0xrecipient", 100);

    let result = manager.send_tx(tx).await;

    mock.assert();
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), mock_tx_hash);
}

#[tokio::test]
async fn test_send_tx_rpc_error() {
    // 错误路径：模拟 RPC 返回错误
    let server = MockServer::start();

    let _mock = server.mock(|when, then| {
        when.method(Method::POST).path("/");
        then.status(200) // RPC 错误通常 HTTP 状态码也是 200
            .header("content-type", "application/json")
            .json_body(json!({
                "jsonrpc": "2.0",
                "id": 1,
                "error": { "code": -32000, "message": "invalid sender" }
            }));
    });

    let manager = NodeManager::new(&server.base_url());
    let wallet = Wallet::from_mnemonic("test").unwrap();
    let tx = Tx::new(&wallet, "0xrecipient", 100);

    let result = manager.send_tx(tx).await;
    assert!(result.is_err());
}