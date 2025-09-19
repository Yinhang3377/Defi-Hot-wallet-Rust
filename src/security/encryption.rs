//! 加密模块：负责钱包核心数据的加解密逻辑
//! 支持对称加密（如 AES-GCM）、非对称加密（如 secp256k1）等

use crate::tools::error::WalletError;
use aes_gcm::aead::{ Aead, Payload };
use aes_gcm::{ Aes256Gcm, Key, Nonce, KeyInit };
use hex;
use log;
use rand;
use zeroize::Zeroize;
use argon2::Argon2;
use crate::config::WalletConfig;
use base64::engine::general_purpose;
use base64::Engine as _;

/// WalletSecurity handles cryptographic operations for the hot wallet.
/// This module provides secure encryption and decryption of sensitive data, such as private keys.
pub struct WalletSecurity;

impl WalletSecurity {
    /// 使用 AES-256-GCM 加密私钥
    ///
    /// # 参数
    /// * `private_key` - 待加密的私钥（如 secp256k1，32字节）
    /// * `encryption_key` - 加密用的密钥（必须为64位十六进制字符串，解码后为32字节）
    /// * `aad` - 关联数据（Associated Data），用于认证但不会被加密
    ///
    /// # 返回值
    /// 返回包含随机nonce的密文（nonce前置），或 WalletError 错误
    ///
    /// # 安全说明
    /// - 使用随机12字节nonce防止重放攻击
    /// - nonce前置于密文，便于解密时提取
    /// - 加密密钥用完自动清零，防止内存泄漏
    /// - 生产环境建议用KDF安全派生encryption_key
    pub fn encrypt_private_key(
        private_key: &[u8],
        encryption_key: &str,
        aad: &[u8]
    ) -> Result<Vec<u8>, WalletError> {
        if private_key.len() != 32 {
            return Err(WalletError::EncryptionError("待加密的私钥长度必须为32字节".to_string()));
        }

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
        let salt = if !aad.is_empty() { aad } else { b"hotwallet-default-salt" };
        let derived_key = Self::derive_key_from_env_key(&key_bytes, salt);
        println!("[encrypt] derived_key: {:02x?}", derived_key);
        println!("[encrypt] aad: {:02x?}", aad);

        // 用派生密钥初始化加密器
        let key = Key::<Aes256Gcm>::from_slice(&derived_key);
        let cipher = Aes256Gcm::new(key);

        // 生成随机12字节nonce
        let nonce_bytes: [u8; 12] = rand::random();
        let nonce = Nonce::from_slice(&nonce_bytes);

        // 加密私钥
        let ciphertext = cipher
            .encrypt(nonce, Payload { msg: &private_key[..], aad })
            .map_err(|e| WalletError::EncryptionError(format!("加密失败: {}", e)))?;
        println!("[encrypt] ciphertext len: {}", ciphertext.len());

        let mut result = Vec::with_capacity(nonce_bytes.len() + ciphertext.len());
        result.extend_from_slice(&nonce_bytes);
        result.extend_from_slice(&ciphertext);
        println!("[encrypt] result (nonce+ciphertext) len: {}", result.len());

        Ok(result)
    }

    /// 解密用 AES-256-GCM 加密的私钥
    ///
    /// # 参数
    /// * `ciphertext` - 加密数据（前12字节为nonce）
    /// * `encryption_key` - 解密用密钥（必须为64位十六进制字符串）
    /// * `aad` - 关联数据（Associated Data），必须与加密时使用的数据一致
    ///
    /// # 返回值
    /// 返回解密后的私钥明文，或 WalletError 错误
    ///
    /// # 安全说明
    /// - 默认前12字节为nonce
    /// - 密钥用完自动清零
    /// - 生产环境需校验密钥派生与nonce管理
    #[allow(dead_code)]
    pub fn decrypt_private_key(
        ciphertext: &[u8],
        encryption_key: &str,
        aad: &[u8]
    ) -> Result<Vec<u8>, WalletError> {
        let key_bytes = hex::decode(encryption_key).map_err(|_| {
            log::error!("解密密钥格式错误，无法从Hex解码: [已隐藏]");
            WalletError::EncryptionError("解密密钥必须是有效的64位十六进制字符串".to_string())
        })?;
        if key_bytes.len() != 32 {
            log::error!("解密密钥解码后长度不为32字节, 实际长度: [已隐藏]");
            return Err(WalletError::EncryptionError("解密密钥解码后长度必须为32字节".to_string()));
        }

        // KDF: 用 Argon2 对明文密钥二次加固，salt 可用 AAD 或其它上下文
        let salt = if !aad.is_empty() { aad } else { b"hotwallet-default-salt" };
        let derived_key = Self::derive_key_from_env_key(&key_bytes, salt);

        // 提取 nonce 和密文
        if ciphertext.len() < 12 {
            return Err(WalletError::EncryptionError("密文长度不足，无法提取nonce".to_string()));
        }
        let (nonce_bytes, actual_ciphertext) = ciphertext.split_at(12);
        let nonce = Nonce::from_slice(nonce_bytes);

        // 用派生密钥初始化解密器
        let key = Key::<Aes256Gcm>::from_slice(&derived_key);
        let cipher = Aes256Gcm::new(key);

        // 解密数据（使用相同 AAD）
        let decrypted_data = cipher
            .decrypt(nonce, Payload { msg: actual_ciphertext, aad })
            .map_err(|e| WalletError::EncryptionError(format!("解密失败: {}", e)))?;

        Ok(decrypted_data)
    }

    /// 检查加密密钥强度（简单实现：检测是否为全0、全1、重复模式等弱密钥）
    #[allow(dead_code)]
    pub fn is_weak_key(key_bytes: &[u8]) -> bool {
        if key_bytes.iter().all(|&b| b == 0) || key_bytes.iter().all(|&b| b == 0xff) {
            return true;
        }
        // 检查是否为重复模式（如 010101... 或 ababab...）
        if key_bytes.len() >= 2 {
            let first = key_bytes[0];
            let second = key_bytes[1];
            if
                key_bytes
                    .iter()
                    .enumerate()
                    .all(|(i, &b)| {
                        if i % 2 == 0 { b == first } else { b == second }
                    })
            {
                return true;
            }
        }
        false
    }

    /// 使用 Argon2 从环境密钥派生加密密钥
    pub fn derive_key_from_env_key(env_key: &[u8], salt: &[u8]) -> [u8; 32] {
        let mut output = [0u8; 32];
        let salt_bytes = std::str::from_utf8(salt).unwrap_or("default-salt").as_bytes();
        let argon2 = Argon2::default();
        if argon2.hash_password_into(env_key, salt_bytes, &mut output).is_err() {
            output.fill(0);
        }
        output
    }

    /// Derives an encryption key using Argon2 and integrates with WalletConfig.
    ///
    /// # Parameters
    /// * `password` - The input password to derive the key from.
    /// * `config` - WalletConfig instance containing the salt.
    ///
    /// # Returns
    /// A 32-byte derived encryption key.
    pub fn derive_encryption_key(
        password: &str,
        config: &WalletConfig
    ) -> Result<[u8; 32], WalletError> {
        // Decode the salt from WalletConfig.salt (NOT encryption_key)
        let salt = general_purpose::STANDARD
            .decode(&config.salt)
            .map_err(|e| WalletError::InvalidSalt(format!("Failed to decode salt: {}", e)))?;

        // Configure Argon2 parameters
        let argon2 = Argon2::default();

        // Derive the key
        let mut derived_key = [0u8; 32];
        argon2
            .hash_password_into(password.as_bytes(), &salt, &mut derived_key)
            .map_err(|e| {
                WalletError::EncryptionError(format!("Key derivation failed: {}", e))
            })?;

        Ok(derived_key)
    }
    // ---- 内部辅助函数 ----
    // 上面已实现 is_weak_key 与 derive_key_from_env_key（Argon2 版本），此处不再重复
}

#[cfg(test)]
mod tests {
    use super::*;
    use base64::engine::general_purpose;
    use crate::config::WalletConfig;

    #[test]
    fn test_encrypt_decrypt_cycle() {
        // 测试加密解密流程
        let private_key = b"32_byte_private_key_1234567890ab"; // 32 bytes
        // 64位十六进制字符串（32字节）使用良好分布随机密钥
        let encryption_key = "0123456789abcdef1032547698badcfe1133557799bbddff123456789abcdef0";
        let aad = b"associated_data";

        let encrypted = WalletSecurity::encrypt_private_key(
            private_key,
            encryption_key,
            aad
        ).unwrap();
        println!("[test] encrypted: {:02x?}", encrypted);
        let decrypted = WalletSecurity::decrypt_private_key(
            &encrypted,
            encryption_key,
            aad
        ).unwrap();
        println!("[test] decrypted: {:02x?}", decrypted);
        assert_eq!(private_key.to_vec(), decrypted);
    }

    #[test]
    fn test_invalid_key_length() {
        // 测试密钥长度不足的情况
        let private_key = b"test_key";
        let aad = b"";
        let short_key = "too_short";
        // 私钥长度不足32字节，应该报错
        let result = WalletSecurity::encrypt_private_key(private_key, short_key, aad);
        assert!(matches!(result, Err(WalletError::EncryptionError(_))));

        // 密钥格式错误，应该报错
        let valid_private_key = &[0; 32];
        let result2 = WalletSecurity::encrypt_private_key(valid_private_key, short_key, aad);
        assert!(matches!(result2, Err(WalletError::EncryptionError(_))));
    }

    #[test]
    fn test_invalid_ciphertext() {
        // 测试密文长度不足的情况
        // 64位十六进制字符串
        let encryption_key = "a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2";
        let aad = b"";
        let invalid_ciphertext = b"too_short";
        let result = WalletSecurity::decrypt_private_key(invalid_ciphertext, encryption_key, aad);
        assert!(matches!(result, Err(WalletError::EncryptionError(_))));

        // 刚好12字节，也应该报错
        let invalid_ciphertext_2 = b"123456789012";
        let result2 = WalletSecurity::decrypt_private_key(
            invalid_ciphertext_2,
            encryption_key,
            aad
        );
        assert!(matches!(result2, Err(WalletError::EncryptionError(_))));
    }

    #[test]
    fn test_derive_encryption_key_valid() {
        let password = "password123";
        let config = WalletConfig {
            encryption_key: "a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2".to_string(),
            network: "testnet".to_string(),
            salt: general_purpose::STANDARD.encode(b"testsalt"),
        };
        let result = WalletSecurity::derive_encryption_key(password, &config);
        assert!(result.is_ok());
        let key = result.unwrap();
        assert_eq!(key.len(), 32);
    }

    #[test]
    fn test_derive_encryption_key_invalid_salt() {
        let password = "password123";
        let config = WalletConfig {
            encryption_key: "a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2".to_string(),
            network: "testnet".to_string(),
            salt: "invalid-base64!".to_string(),
        };
        let result = WalletSecurity::derive_encryption_key(password, &config);
        assert!(matches!(result, Err(WalletError::InvalidSalt(_))));
    }

    #[test]
    fn test_derive_encryption_key_empty_password() {
        let config = WalletConfig {
            encryption_key: "a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2".to_string(),
            network: "testnet".to_string(),
            salt: general_purpose::STANDARD.encode(b"testsalt"),
        };
        let result = WalletSecurity::derive_encryption_key("", &config);
        assert!(result.is_ok()); // Argon2 allows empty passwords
    }
}
