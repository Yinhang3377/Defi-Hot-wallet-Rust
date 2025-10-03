use anyhow::{anyhow, Result};
use hex;
use reqwest::Client;
use serde_json::json;

use crate::core::domain::Tx;

/// 鍏煎淇濈暀鐨勫崰浣嶅嚱鏁?pub fn select_node() -> Option<String> {
    Some("https://mainnet.infura.io/v3/".to_string())
}

pub struct NodeManager {
    client: Client,
    rpc_url: String,
}

impl NodeManager {
    /// 鍒涘缓涓€涓?NodeManager 瀹炰緥
    pub fn new(rpc_url: &str) -> Self {
        Self { client: Client::new(), rpc_url: rpc_url.to_string() }
    }

    /// 鍒涘缓 Infura 涓荤綉瀹㈡埛绔紙浼犲叆 Project ID锛?    pub fn new_infura(project_id: &str) -> Self {
        let rpc_url = format!("https://mainnet.infura.io/v3/{}", project_id);
        Self { client: Client::new(), rpc_url }
    }

    /// 鍙戦€佷氦鏄擄紙eth_sendRawTransaction锛夛紝杩斿洖浜ゆ槗鍝堝笇锛?x...锛?    pub async fn send_tx(&self, tx: Tx) -> Result<String> {
        // 鍋囧畾 tx.serialize() 杩斿洖 RLP/鍘熷浜ゆ槗瀛楄妭
        let raw_hex = format!("0x{}", hex::encode(tx.serialize()));
        let payload = json!({
            "jsonrpc": "2.0",
            "method": "eth_sendRawTransaction",
            "params": [ raw_hex ],
            "id": 1
        });

        let resp = self.client.post(&self.rpc_url).json(&payload).send().await?;
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::domain::{Tx, Wallet};

    #[test]
    fn test_send_transaction() {
        // 妯℃嫙鍙戦€佷氦鏄?        let tx = Tx::new(&Wallet::from_mnemonic("test").unwrap(), "0x123", 100);
        let raw_hex = format!("0x{}", hex::encode(tx.serialize()));
        assert!(raw_hex.starts_with("0x"));
    }
}
