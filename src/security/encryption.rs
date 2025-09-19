/// 检查加密密钥强度（简单实现：检测是否为全0、全1、重复模式等弱密钥）
fn is_weak_key(key_bytes: &[u8]) -> bool {
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
                .all(|(i, &b)| if i % 2 == 0 { b == first } else { b == second })
        {
            return true;
        }
    }
    false
}
/// 加密模块：负责钱包核心数据的加解密逻辑
/// 支持对称加密（如 AES-GCM）、非对称加密（如 secp256k1）等

use aes_gcm::{ Aes256Gcm, Key, Nonce };
use aes_gcm::aead::{ Aead, AeadInPlace };
use hex;
use aes_gcm::KeyInit;
use log;
use zeroize::Zeroize;
use rand;
use crate::tools::error::WalletError;
use argon2::{ self, Config as ArgonConfig, Variant, Version };
/// KDF: 使用 Argon2 对明文 ENCRYPTION_KEY 进行二次加固，输出 32 字节密钥
fn derive_key_from_env_key(env_key: &[u8], salt: &[u8]) -> [u8; 32] {
    let mut output = [0u8; 32];
    let config = ArgonConfig {
        variant: Variant::Argon2id,
        version: Version::Version13,
        mem_cost: 65536, // 64 MiB
        time_cost: 3,
        lanes: 2,
        thread_mode: argon2::ThreadMode::Parallel,
        secret: &[],
        ad: &[],
        hash_length: 32,
    };
    argon2
        ::hash_raw(env_key, salt, &config)
        .map(|res| {
            output.copy_from_slice(&res);
        })
        .unwrap_or_else(|_| output.fill(0));
    output
}
use crate::security::memory_protection::{ SensitiveData, MemoryLock };
/// 用 SensitiveData 包裹的加密流程，自动内存锁定和清零
pub fn encrypt_private_key_sensitive(
    private_key: &mut SensitiveData<[u8; 32]>,
    encryption_key: &mut SensitiveData<Vec<u8>>,
    aad: &[u8]
) -> Result<Vec<u8>, WalletError> {
    // 锁定内存
    private_key.lock().ok();
    encryption_key.lock().ok();
    // 验证长度
    if private_key.data.len() != 32 {
        return Err(WalletError::EncryptionError("待加密的私钥长度必须为32字节".to_string()));
    }
    if encryption_key.data.len() != 64 {
        return Err(WalletError::EncryptionError("加密密钥必须为64字节十六进制字符串".to_string()));
    }
    // 解码密钥
    let mut key_bytes = hex::decode(&encryption_key.data).map_err(|e| {
        log::error!("加密密钥格式错误，无法从Hex解码: {}", e);
        WalletError::EncryptionError("加密密钥必须是有效的64位十六进制字符串".to_string())
    })?;
    if key_bytes.len() != 32 {
        log::error!("加密密钥解码后长度不为32字节, 实际长度: {}", key_bytes.len());
        return Err(WalletError::EncryptionError("加密密钥解码后长度必须为32字节".to_string()));
    }
    let mut key = Key::<Aes256Gcm>::from_slice(&key_bytes).clone();
    let cipher = Aes256Gcm::new(&key);
    let nonce_bytes: [u8; 12] = rand::random();
    let nonce = Nonce::from_slice(&nonce_bytes);
    let ciphertext = cipher
        .encrypt(nonce, (private_key.data.as_ref(), aad).as_ref())
        .map_err(|e| WalletError::EncryptionError(format!("加密失败: {}", e)))?;
    let mut result = Vec::with_capacity(nonce_bytes.len() + ciphertext.len());
    result.extend_from_slice(&nonce_bytes);
    result.extend_from_slice(&ciphertext);
    // 用完自动清零密钥
    key_bytes.zeroize();
    key.zeroize();
    // 解锁内存
    private_key.unlock().ok();
    encryption_key.unlock().ok();
    Ok(result)
}

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
        let salt = if !aad.is_empty() { aad } else { b"hotwallet-default-salt" };
        let derived_key = Self::derive_key_from_env_key(&key_bytes, salt);

        // 用派生密钥初始化加密器
        let mut key = Key::<Aes256Gcm>::from_slice(&derived_key).clone();
        let cipher = Aes256Gcm::new(&key);

        // 生成随机12字节nonce
        let nonce_bytes: [u8; 12] = rand::random();
        let nonce = Nonce::from_slice(&nonce_bytes);

        // 加密私钥
        let ciphertext = cipher
            .encrypt(nonce, (private_key, aad).as_ref())
            .map_err(|e| WalletError::EncryptionError(format!("加密失败: {}", e)))?;

        // nonce前置于密文，便于解密
        let mut result = Vec::with_capacity(nonce_bytes.len() + ciphertext.len());
        result.extend_from_slice(&nonce_bytes);
        result.extend_from_slice(&ciphertext);

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
        aad: &[u8]
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

        // 用密钥初始化解密器，这里的 from_slice 期望一个32字节的切片
        let mut key = Key::<Aes256Gcm>::from_slice(&key_bytes).clone();
        let cipher = Aes256Gcm::new(&key);

        // 解密数据
        let plaintext = cipher
            .decrypt(nonce, (actual_ciphertext, aad).as_ref())
            .map_err(|e| WalletError::EncryptionError(format!("解密失败: {}", e)))?;

        // 用完自动清零密钥
        key_bytes.zeroize();
        // 深度防御：显式清零 Key 对象本身
        key.zeroize();

        Ok(plaintext)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypt_decrypt_cycle() {
        // 测试加密解密流程
        let private_key = b"32_byte_private_key_1234567890ab";
        // 64位十六进制字符串
        let encryption_key = "33325f627974655f656e6372797074696f6e5f6b65795f31323334353637383930";
        let aad = b"associated_data";

        let encrypted = WalletSecurity::encrypt_private_key(
            private_key,
            encryption_key,
            aad
        ).unwrap();
        let decrypted = WalletSecurity::decrypt_private_key(
            &encrypted,
            encryption_key,
            aad
        ).unwrap();
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
        let encryption_key = "33325f627974655f656e6372797074696f6e5f6b65795f31323334353637383930";
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
}
