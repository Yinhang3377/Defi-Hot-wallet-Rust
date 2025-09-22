use anyhow::Result;
use zeroize::Zeroize;

/// Wallet domain entity (placeholder)
#[derive(Debug, Clone)]
pub struct Wallet {
    pub id: String,
}

impl Wallet {
    pub fn from_mnemonic(_mnemonic: &str) -> Result<Self> {
        Ok(Self {
            id: "wallet-placeholder".to_string(),
        })
    }
}

/// Transaction domain entity (placeholder)
#[derive(Debug, Clone)]
pub struct Tx {
    pub id: String,
    pub to: String,
    pub amount: u64,
}

impl Tx {
    pub fn new(_w: &Wallet, to: &str, amount: u64) -> Self {
        Self {
            id: "tx-placeholder".to_string(),
            to: to.to_string(),
            amount,
        }
    }
    pub fn serialize(&self) -> Vec<u8> {
        format!("tx:{}:{}:{}", self.id, self.to, self.amount).into_bytes()
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
