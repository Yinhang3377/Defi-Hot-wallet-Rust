use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json;
use zeroize::Zeroize;

/// 领域模型

#[derive(Serialize, Deserialize)] // 添加 Serialize derive
pub struct Wallet {
    pub id: String,
    // 添加其他字段
}

impl Wallet {
    pub fn from_mnemonic(_mnemonic: &str) -> Result<Self> {
        // 实现
        Ok(Wallet { id: "test".to_string() })
    }
}

#[derive(Serialize, Deserialize)] // 添加 Serialize derive
pub struct Tx {
    // 添加字段
    pub to: String,
    pub amount: u64,
}

impl Tx {
    pub fn new(_w: &Wallet, to: &str, amount: u64) -> Self {
        Tx { to: to.to_string(), amount }
    }

    pub fn serialize(&self) -> Vec<u8> {
        serde_json::to_string(self).unwrap().into_bytes()
    }
}

/// Private key wrapper (32 bytes)
pub struct PrivateKey([u8; 32]);
impl PrivateKey {
    pub fn new(k: [u8; 32]) -> Self {
        Self(k)
    }
}
impl Zeroize for PrivateKey {
    fn zeroize(&mut self) {
        self.0.zeroize();
    }
}
impl Drop for PrivateKey {
    fn drop(&mut self) {
        self.zeroize();
    }
}

/// Public key wrapper (33 bytes)
pub struct PublicKey([u8; 33]);
impl PublicKey {
    pub fn new(k: [u8; 33]) -> Self {
        Self(k)
    }
}
impl Zeroize for PublicKey {
    fn zeroize(&mut self) {
        self.0.zeroize();
    }
}
impl Drop for PublicKey {
    fn drop(&mut self) {
        self.zeroize();
    }
}

/// Address wrapper (20 bytes)
pub struct Address([u8; 20]);
impl Address {
    pub fn new(a: [u8; 20]) -> Self {
        Self(a)
    }
}
impl Zeroize for Address {
    fn zeroize(&mut self) {
        self.0.zeroize();
    }
}
impl Drop for Address {
    fn drop(&mut self) {
        self.zeroize();
    }
}

/// Nonce wrapper (u64)
pub struct Nonce(u64);
impl Nonce {
    pub fn new(n: u64) -> Self {
        Self(n)
    }
    pub fn get(&self) -> u64 {
        self.0
    }
    pub fn set(&mut self, v: u64) {
        self.0 = v;
    }
}
impl Zeroize for Nonce {
    fn zeroize(&mut self) {
        self.0 = 0;
    }
}
impl Drop for Nonce {
    fn drop(&mut self) {
        self.zeroize();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wallet_from_mnemonic() {
        let wallet = Wallet::from_mnemonic("test mnemonic").unwrap();
        assert_eq!(wallet.id, "test");
    }

    #[test]
    fn test_tx_new() {
        let wallet = Wallet::from_mnemonic("test").unwrap();
        let tx = Tx::new(&wallet, "0x123", 100);
        assert_eq!(tx.to, "0x123");
        assert_eq!(tx.amount, 100);
    }

    #[test]
    fn test_tx_serialize() {
        let wallet = Wallet::from_mnemonic("test").unwrap();
        let tx = Tx::new(&wallet, "0x123", 100);
        let serialized = tx.serialize();
        assert!(!serialized.is_empty());
        // 验证可以反序列化
        let deserialized: Tx = serde_json::from_slice(&serialized).unwrap();
        assert_eq!(deserialized.to, tx.to);
        assert_eq!(deserialized.amount, tx.amount);
    }

    #[test]
    fn test_private_key_new() {
        let key = [1u8; 32];
        let pk = PrivateKey::new(key);
        assert_eq!(pk.0, key);
    }

    #[test]
    fn test_public_key_new() {
        let key = [2u8; 33];
        let pk = PublicKey::new(key);
        assert_eq!(pk.0, key);
    }

    #[test]
    fn test_address_new() {
        let addr = [3u8; 20];
        let address = Address::new(addr);
        assert_eq!(address.0, addr);
    }

    #[test]
    fn test_nonce_new() {
        let nonce = Nonce::new(42);
        assert_eq!(nonce.get(), 42);
    }

    #[test]
    fn test_nonce_set() {
        let mut nonce = Nonce::new(0);
        nonce.set(100);
        assert_eq!(nonce.get(), 100);
    }
}
