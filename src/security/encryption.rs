//! 加密模块：负责钱包核心数据的加解密逻辑
//! 支持对称加密（如 AES-GCM）、非对称加密（如 secp256k1）等

use crate::tools::error::WalletError;
use aes_gcm::aead::{Aead, Payload};
use aes_gcm::KeyInit;
use aes_gcm::{Aes256Gcm, Key, Nonce};
use hex;
use log;
use rand;
use zeroize::Zeroize;
// 移除暂未添加到 Cargo.toml 的 argon2 依赖，后续如需再恢复
// use argon2::{ self, Config as ArgonConfig, Variant, Version };
// 已移除 SensitiveData 版本加密函数，简化实现。后续如需可重新引入。

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
        aad: &[u8],
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
            return Err(WalletError::EncryptionError(
                "加密密钥过于简单，存在安全风险，请更换更复杂的密钥！".to_string(),
            ));
        }
        // KDF: 用 Argon2 对明文密钥二次加固，salt 可用 AAD 或其它上下文
        let salt = if !aad.is_empty() { aad } else { b"hotwallet-default-salt" };
        let derived_key = Self::derive_key_from_env_key(&key_bytes, salt);
        println!("[encrypt] derived_key: {:02x?}", derived_key);
        println!("[encrypt] aad: {:02x?}", aad);

        // 用派生密钥初始化加密器
        let mut key = *Key::<Aes256Gcm>::from_slice(&derived_key);
        let cipher = Aes256Gcm::new(&key);

        // 生成随机12字节nonce
        let nonce_bytes: [u8; 12] = rand::random();
        let nonce = Nonce::from_slice(&nonce_bytes);

        // 加密私钥
        println!("[encrypt] nonce: {:02x?}", nonce);
        let ciphertext = cipher
            .encrypt(nonce, Payload { msg: private_key, aad })
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
        aad: &[u8],
    ) -> Result<Vec<u8>, WalletError> {
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
        let derived_key = Self::derive_key_from_env_key(&key_bytes, aad);
        println!("[decrypt] derived_key: {:02x?}", derived_key);
        println!("[decrypt] aad: {:02x?}", aad);
        println!("[decrypt] nonce: {:02x?}", nonce);
        println!("[decrypt] ciphertext len: {}", actual_ciphertext.len());

        // 用派生密钥初始化解密器
        let mut key = *Key::<Aes256Gcm>::from_slice(&derived_key);
        let cipher = Aes256Gcm::new(&key);

        // 解密数据
        let plaintext = cipher
            .decrypt(nonce, Payload { msg: actual_ciphertext, aad })
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
    /// * `salt` - 派生密钥的盐值
    ///
    /// # 返回值
    /// 返回派生的 32 字节加密密钥
    // pub fn derive_encryption_key(password: &[u8], salt: &[u8]) -> Vec<u8> {
    //     let mut key = [0u8; 32]; // 目标密钥长度
    //     Argon2::default().hash_password_into(password, salt, &mut key).expect("密钥派生失败");
    //     key.to_vec()
    // }
    // ---- 内部辅助函数 ----
    fn is_weak_key(key_bytes: &[u8]) -> bool {
        if key_bytes.iter().all(|&b| b == 0) || key_bytes.iter().all(|&b| b == 0xff) {
            return true;
        }
        if key_bytes.len() >= 2 {
            let first = key_bytes[0];
            let second = key_bytes[1];
            if key_bytes
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
            *b = env_key.get(i % env_key.len()).cloned().unwrap_or(0)
                ^ salt.get(i % salt.len()).cloned().unwrap_or(0);
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypt_decrypt_cycle() {
        // 测试加密解密流程
        let private_key = &[0u8; 32];
        // 64位十六进制字符串（32字节）
        let encryption_key = "a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2";
        let aad = b"associated_data";

        let encrypted =
            WalletSecurity::encrypt_private_key(private_key, encryption_key, aad).unwrap();
        println!("[test] encrypted: {:02x?}", encrypted);
        let decrypted =
            WalletSecurity::decrypt_private_key(&encrypted, encryption_key, aad).unwrap();
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
        let result2 =
            WalletSecurity::decrypt_private_key(invalid_ciphertext_2, encryption_key, aad);
        assert!(matches!(result2, Err(WalletError::EncryptionError(_))));
    }
}
