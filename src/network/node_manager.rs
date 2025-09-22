use anyhow::{anyhow, Result};
use hex;
use reqwest::Client;
use serde_json::json;

use crate::core::domain::Tx;

/// 兼容保留的占位函数
pub fn select_node() -> Option<String> {
    Some("https://mainnet.infura.io/v3/".to_string())
}

pub struct NodeManager {
    client: Client,
    rpc_url: String,
}

impl NodeManager {
    /// 创建 Infura 主网客户端（传入 Project ID）
    pub fn new_infura(project_id: &str) -> Self {
        let rpc_url = format!("https://mainnet.infura.io/v3/{}", project_id);
        Self {
            client: Client::new(),
            rpc_url,
        }
    }

    /// 发送交易（eth_sendRawTransaction），返回交易哈希（0x...）
    pub async fn send_tx(&self, tx: Tx) -> Result<String> {
        // 假定 tx.serialize() 返回 RLP/原始交易字节
        let raw_hex = format!("0x{}", hex::encode(tx.serialize()));
        let payload = json!({
            "jsonrpc": "2.0",
            "method": "eth_sendRawTransaction",
            "params": [ raw_hex ],
            "id": 1
        });

        let resp = self
            .client
            .post(&self.rpc_url)
            .json(&payload)
            .send()
            .await?;
        let status = resp.status();
        let body: serde_json::Value = resp.json().await?;

        if !status.is_success() {
            return Err(anyhow!("rpc error status: {} body: {:?}", status, body));
        }
        if let Some(result) = body.get("result").and_then(|v| v.as_str()) {
            Ok(result.to_string())
        } else if let Some(err) = body.get("error") {
            Err(anyhow!("rpc returned error: {:?}", err))
        } else {
            Err(anyhow!("unexpected rpc response: {:?}", body))
        }
    }
}
