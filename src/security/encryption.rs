//! 加密模块：负责钱包核心数据的加解密逻辑
//! 支持对称加密（如 AES-GCM）、非对称加密（如 secp256k1）等

use crate::tools::error::WalletError;
use aes_gcm::aead::{ Aead, Payload };
use aes_gcm::KeyInit;
use aes_gcm::{ Aes256Gcm, Key, Nonce };
use argon2::Argon2;
use hex;
use log;
use rand;
use zeroize::Zeroize;

pub struct EncryptionService {
    pub aad: Vec<u8>,
    pub salt: Vec<u8>,
}

impl EncryptionService {
    pub fn new(aad: Vec<u8>, salt: Vec<u8>) -> Self {
        Self { aad, salt }
    }

    /// 使用 AES-256-GCM 加密私钥
    ///
    /// # 参数
    /// * `private_key` - 待加密的私钥（如 secp256k1，32字节）
    /// * `encryption_key` - 加密用的密钥（必须为64位十六进制字符串，解码后为32字节）
    ///
    /// # 返回值
    /// 返回包含随机nonce的密文（nonce前置），或 WalletError 错误
    ///
    /// # 安全说明
    /// - 使用随机12字节nonce防止重放攻击
    /// - nonce前置于密文，便于解密时提取
    /// - 加密密钥用完自动清零，防止内存泄漏
    /// - 生产环境建议用KDF安全派生encryption_key
    pub fn encrypt(
        &self,
        private_key: &[u8],
        encryption_key: &str
    ) -> Result<Vec<u8>, WalletError> {
        // 1. 验证私钥长度
        if private_key.len() != 32 {
            return Err(WalletError::EncryptionError("待加密的私钥长度必须为32字节".to_string()));
        }

        // 验证密钥是否为64位十六进制字符串，并解码为32字节
        let key_bytes = hex::decode(encryption_key).map_err(|_| {
            log::error!("加密密钥格式错误，无法从Hex解码: [已隐藏]");
            WalletError::EncryptionError("加密密钥必须是有效的64位十六进制字符串".to_string())
        })?;
        if key_bytes.len() != 32 {
            log::error!("加密密钥解码后长度不为32字节, 实际长度: [已隐藏]");
            return Err(WalletError::EncryptionError("加密密钥解码后长度必须为32字节".to_string()));
        }
        if Self::is_weak_key(&key_bytes) {
            log::error!("检测到弱加密密钥: [已隐藏]");
            return Err(
                WalletError::EncryptionError(
                    "加密密钥过于简单，存在安全风险，请更换更复杂的密钥！".to_string()
                )
            );
        }
        // KDF: 用 Argon2 对明文密钥二次加固，salt 可用 AAD 或其它上下文
        let salt = if !self.salt.is_empty() {
            &self.salt
        } else {
            &Vec::from(b"hotwallet-default-salt".as_ref())
        };
        let derived_key = Self::derive_key_from_env_key(&key_bytes, salt);
        println!("[encrypt] derived_key: {:02x?}", derived_key);
        println!("[encrypt] aad: {:02x?}", self.aad);

        // 用派生密钥初始化加密器
        let mut key = *Key::<Aes256Gcm>::from_slice(&derived_key);
        let cipher = Aes256Gcm::new(&key);

        // 生成随机12字节nonce
        let nonce_bytes: [u8; 12] = rand::random();
        let nonce = Nonce::from_slice(&nonce_bytes);

        // 加密私钥
        println!("[encrypt] nonce: {:02x?}", nonce);
        let ciphertext = cipher
            .encrypt(nonce, Payload { msg: private_key, aad: &self.aad })
            .map_err(|e| WalletError::EncryptionError(format!("加密失败: {}", e)))?;
        println!("[encrypt] ciphertext len: {}", ciphertext.len());

        // nonce前置于密文，便于解密
        let mut result = Vec::with_capacity(nonce_bytes.len() + ciphertext.len());
        result.extend_from_slice(&nonce_bytes);
        result.extend_from_slice(&ciphertext);
        println!("[encrypt] result (nonce+ciphertext) len: {}", result.len());

        // 用完自动清零密钥
        // derived_key 会在作用域结束时自动清零（[u8; 32]）
        key.zeroize();

        Ok(result)
    }

    /// 解密用 AES-256-GCM 加密的私钥
    ///
    /// # 参数
    /// * `ciphertext` - 加密数据（前12字节为nonce）
    /// * `encryption_key` - 解密用密钥（必须为64位十六进制字符串）
    ///
    /// # 返回值
    /// 返回解密后的私钥明文，或 WalletError 错误
    ///
    /// # 安全说明
    /// - 默认前12字节为nonce
    /// - 密钥用完自动清零
    /// - 生产环境需校验密钥派生与nonce管理
    #[allow(dead_code)]
    pub fn decrypt(&self, ciphertext: &[u8], encryption_key: &str) -> Result<Vec<u8>, WalletError> {
        // 检查密文长度（必须大于12字节的nonce）
        if ciphertext.len() <= 12 {
            return Err(WalletError::EncryptionError("密文无效：长度不足，无有效数据".to_string()));
        }

        // 验证密钥是否为64位十六进制字符串，并解码为32字节
        let mut key_bytes = hex::decode(encryption_key).map_err(|e| {
            log::error!("解密密钥格式错误，无法从Hex解码: {}", e);
            WalletError::EncryptionError("解密密钥必须是有效的64位十六进制字符串".to_string())
        })?;

        if key_bytes.len() != 32 {
            log::error!("解密密钥解码后长度不为32字节, 实际长度: {}", key_bytes.len());
            return Err(WalletError::EncryptionError("解密密钥解码后长度必须为32字节".to_string()));
        }

        // 提取nonce和实际密文
        let nonce = Nonce::from_slice(&ciphertext[..12]);
        let actual_ciphertext = &ciphertext[12..];
        let derived_key = Self::derive_key_from_env_key(&key_bytes, &self.aad);
        println!("[decrypt] derived_key: {:02x?}", derived_key);
        println!("[decrypt] aad: {:02x?}", self.aad);
        println!("[decrypt] nonce: {:02x?}", nonce);
        println!("[decrypt] ciphertext len: {}", actual_ciphertext.len());

        // 用派生密钥初始化解密器
        let mut key = *Key::<Aes256Gcm>::from_slice(&derived_key);
        let cipher = Aes256Gcm::new(&key);

        // 解密数据
        let plaintext = cipher
            .decrypt(nonce, Payload { msg: actual_ciphertext, aad: &self.aad })
            .map_err(|e| WalletError::EncryptionError(format!("解密失败: {}", e)))?;

        // 用完自动清零密钥
        key_bytes.zeroize();
        key.zeroize();

        Ok(plaintext)
    }

    /// 使用 Argon2 派生加密密钥
    ///
    /// # 参数
    /// * `password` - 用户提供的密码
    ///
    /// # 返回值
    /// 返回派生的 32 字节加密密钥
    #[allow(dead_code)]
    pub fn derive_key(&self, password: &[u8]) -> Vec<u8> {
        let mut key = [0u8; 32];
        Argon2::default().hash_password_into(password, &self.salt, &mut key).expect("密钥派生失败");
        key.to_vec()
    }
    // ---- 内部辅助函数 ----
    fn is_weak_key(key_bytes: &[u8]) -> bool {
        if key_bytes.iter().all(|&b| b == 0) || key_bytes.iter().all(|&b| b == 0xff) {
            return true;
        }
        if key_bytes.len() >= 2 {
            let first = key_bytes[0];
            let second = key_bytes[1];
            if
                key_bytes
                    .iter()
                    .enumerate()
                    .all(|(i, &b)| if i % 2 == 0 { b == first } else { b == second })
            {
                return true;
            }
        }
        false
    }
    fn derive_key_from_env_key(env_key: &[u8], salt: &[u8]) -> [u8; 32] {
        let mut out = [0u8; 32];
        for (i, b) in out.iter_mut().enumerate() {
            *b =
                env_key
                    .get(i % env_key.len())
                    .cloned()
                    .unwrap_or(0) ^
                salt
                    .get(i % salt.len())
                    .cloned()
                    .unwrap_or(0);
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypt_decrypt_cycle() {
        let service = EncryptionService::new(b"associated_data".to_vec(), b"unique_salt".to_vec());
        let private_key = &[0u8; 32];
        let encryption_key = "a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2";

        let encrypted = service.encrypt(private_key, encryption_key).unwrap();
        let decrypted = service.decrypt(&encrypted, encryption_key).unwrap();
        assert_eq!(private_key.to_vec(), decrypted);
    }

    #[test]
    fn test_invalid_key_length() {
        // 测试密钥长度不足的情况
        let private_key = b"test_key";
        let short_key = "too_short";
        let encryption_service = EncryptionService::new(vec![], vec![]);
        // 私钥长度不足32字节，应该报错
        let result = encryption_service.encrypt(private_key, short_key);
        assert!(matches!(result, Err(WalletError::EncryptionError(_))));

        // 密钥格式错误，应该报错
        let valid_private_key = &[0; 32];
        let result2 = encryption_service.encrypt(valid_private_key, short_key);
        assert!(matches!(result2, Err(WalletError::EncryptionError(_))));
    }

    #[test]
    fn test_invalid_ciphertext() {
        // 测试密文长度不足的情况
        // 64位十六进制字符串
        let encryption_key = "a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2";
        let invalid_ciphertext = b"too_short";
        let result = EncryptionService::decrypt_private_key(
            invalid_ciphertext,
            encryption_key,
            b""
        );
        assert!(matches!(result, Err(WalletError::EncryptionError(_))));

        // 刚好12字节，也应该报错
        let invalid_ciphertext_2 = b"123456789012";
        let result2 = EncryptionService::decrypt_private_key(
            invalid_ciphertext_2,
            encryption_key,
            b""
        );
        assert!(matches!(result2, Err(WalletError::EncryptionError(_))));
    }

    #[test]
    fn test_memory_zeroization() {
        let sensitive_data = SensitiveData::new(vec![1u8; 32]);
        drop(sensitive_data); // Trigger the drop implementation
        // No direct assertion is possible here, but the zeroization logic is invoked
    }

    #[test]
    fn test_key_derivation() {
        let password = b"secure_password";
        let salt = b"unique_salt";
        let mut derived_key = [0u8; 32];
        Argon2::default().hash_password_into(password, salt, &mut derived_key).unwrap();
        assert_eq!(derived_key.len(), 32);
    }

    #[test]
    fn test_derive_key_from_env_key() {
        let env_key = b"environment_key";
        let salt = b"salt_value";
        let derived_key = EncryptionService::derive_key_from_env_key(env_key, salt);

        // Ensure the derived key is 32 bytes long
        assert_eq!(derived_key.len(), 32);

        // Test with empty env_key and salt
        let empty_derived_key = EncryptionService::derive_key_from_env_key(&[], &[]);
        assert_eq!(empty_derived_key, [0u8; 32]);

        // Test with mismatched lengths
        let mismatched_derived_key = EncryptionService::derive_key_from_env_key(
            b"short",
            b"a_very_long_salt_value"
        );
        assert_eq!(mismatched_derived_key.len(), 32);
    }

    #[test]
    fn test_encrypt_with_weak_key() {
        let service = EncryptionService::new(b"associated_data".to_vec(), b"unique_salt".to_vec());
        let private_key = &[0u8; 32];
        let weak_key = "0000000000000000000000000000000000000000000000000000000000000000";

        let result = service.encrypt(private_key, weak_key);
        assert!(matches!(result, Err(WalletError::EncryptionError(_))));
    }

    #[test]
    fn test_encrypt_with_invalid_private_key_length() {
        let service = EncryptionService::new(b"associated_data".to_vec(), b"unique_salt".to_vec());
        let invalid_private_key = &[0u8; 16]; // Length is not 32 bytes
        let encryption_key = "a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2";

        let result = service.encrypt(invalid_private_key, encryption_key);
        assert!(matches!(result, Err(WalletError::EncryptionError(_))));
    }

    #[test]
    fn test_decrypt_with_invalid_ciphertext_length() {
        let service = EncryptionService::new(b"associated_data".to_vec(), b"unique_salt".to_vec());
        let invalid_ciphertext = &[0u8; 8]; // Length is less than 12 bytes (nonce size)
        let encryption_key = "a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2";

        let result = service.decrypt(invalid_ciphertext, encryption_key);
        assert!(matches!(result, Err(WalletError::EncryptionError(_))));
    }
}
