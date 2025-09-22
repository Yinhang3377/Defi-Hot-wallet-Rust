/// 加密与密钥派生（占位 + AES-256-GCM 加密）
pub struct Encryptor;
impl Encryptor {
    pub fn new() -> Self {
        Self
    }
    pub fn derive_key(&self, _password: &str) -> Vec<u8> {
        vec![0u8; 32]
    }
}

impl Default for Encryptor {
    fn default() -> Self {
        Self::new()
    }
}

use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Key, Nonce as AesNonce,
};
use ring::rand::{SecureRandom, SystemRandom};

/// 加密错误
#[derive(Debug)]
pub enum CryptoErr {
    AeadError,
    RngError,
}
impl From<aes_gcm::Error> for CryptoErr {
    fn from(_: aes_gcm::Error) -> Self {
        CryptoErr::AeadError
    }
}
impl core::fmt::Display for CryptoErr {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            CryptoErr::AeadError => write!(f, "AEAD encryption error"),
            CryptoErr::RngError => write!(f, "randomness error"),
        }
    }
}
impl std::error::Error for CryptoErr {}

/// 使用 AES-256-GCM 加密明文。返回: nonce(12字节) || ciphertext
pub fn encrypt_data(plaintext: Vec<u8>, key: &[u8; 32]) -> Result<Vec<u8>, CryptoErr> {
    let key = Key::<Aes256Gcm>::from_slice(key);
    let cipher = Aes256Gcm::new(key);

    // 96-bit nonce（用 ring 生成，避免额外引入 rand 依赖）
    let mut nonce_bytes = [0u8; 12];
    SystemRandom::new()
        .fill(&mut nonce_bytes)
        .map_err(|_| CryptoErr::RngError)?;
    let nonce = AesNonce::from_slice(&nonce_bytes);

    let ciphertext = cipher.encrypt(nonce, plaintext.as_ref())?;
    let mut out = Vec::with_capacity(12 + ciphertext.len());
    out.extend_from_slice(&nonce_bytes);
    out.extend_from_slice(&ciphertext);
    Ok(out)
}
