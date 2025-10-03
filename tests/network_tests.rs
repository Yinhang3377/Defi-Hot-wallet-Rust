//! tests/network_tests.rs
//!
//! 娴嬭瘯 `src/network/node_manager.rs` 鐨勫姛鑳姐€?//! 瑕嗙洊锛?//! - `select_node` 鍗犱綅鍑芥暟
//! - `NodeManager::new_infura` 鏋勯€犲嚱鏁?//! - `NodeManager::send_tx` 鐨勬垚鍔熷拰澶辫触璺緞

use defi_hot_wallet::core::domain::{Tx, Wallet};
use defi_hot_wallet::network::node_manager::{select_node, NodeManager};
use httpmock::{Method, MockServer};
use serde_json::json;

#[test]
fn test_select_node_placeholder() {
    // 姝ｅ父璺緞锛氭祴璇曞崰浣嶅嚱鏁版槸鍚﹁繑鍥為鏈熺殑 URL
    let node_url = select_node();
    assert!(node_url.is_some());
    assert!(node_url.unwrap().contains("infura.io"));
}

#[tokio::test]
async fn test_node_manager_new_infura() {
    // 姝ｅ父璺緞锛氭祴璇曟瀯閫犲嚱鏁?    let project_id = "test_project_id";
    let _manager = NodeManager::new_infura(project_id);
    // 浠呴獙璇佹瀯閫犲嚱鏁版槸鍚︽垚鍔燂紝鍥犱负鍐呴儴瀛楁鏄鏈夌殑銆?    // 鍙互鍦?NodeManager 涓坊鍔犱竴涓叕鍏辩殑 getter 鏂规硶鏉ヨ繘涓€姝ユ祴璇?rpc_url銆?}

#[tokio::test]
async fn test_send_tx_success() {
    // 姝ｅ父璺緞锛氭ā鎷熸垚鍔熺殑 RPC 璋冪敤
    let server = MockServer::start();

    let mock_tx_hash = "0xdeadbeefcafebabefeedface0000000000000000000000000000000000000000";

    let mock = server.mock(|when, then| {
        when.method(Method::POST)
            .path("/") // JSON-RPC endpoint
            .header("content-type", "application/json");
        then.status(200).json_body(json!({
            "jsonrpc": "2.0",
            "id": 1,
            "result": mock_tx_hash
        }));
    });

    // 澧炲姞璋冭瘯鏃ュ織
    println!("Mock server is running at: {}", server.base_url());

    // 浣跨敤妯℃嫙鏈嶅姟鍣ㄧ殑 URL 鍒涘缓 manager
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
    // 閿欒璺緞锛氭ā鎷?RPC 杩斿洖閿欒
    let server = MockServer::start();

    let _mock = server.mock(|when, then| {
        when.method(Method::POST).path("/");
        then.status(200) // RPC 閿欒閫氬父 HTTP 鐘舵€佺爜涔熸槸 200
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
