use std::sync::Mutex;

// 为了测试目的，使用一个简单的内存存储
// 在实际应用中，这会是一个安全的、持久化的存储机制
static KEY_STORAGE: Mutex<Option<Vec<u8>>> = Mutex::new(None);

/// 密钥管理相关的错误类型
#[derive(Debug, thiserror::Error)]
pub enum KeyManagementError {
    #[error("Key generation failed")]
    KeyGenerationFailed,
    #[error("Key storage failed: {0}")]
    KeyStorageFailed(String),
    #[error("Key not found")]
    KeyNotFound,
    #[error("Invalid key: {0}")]
    InvalidKey(String),
}

/// 生成一个新的密钥。
/// 在实际应用中，这会使用一个密码学安全的随机数生成器。
pub fn generate_key() -> Result<Vec<u8>, KeyManagementError> {
    // 示例：生成一个16字节的密钥
    // 实际应用中应使用 `rand::Rng` 和 `rand::thread_rng()`
    Ok(vec![
        0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0A, 0x0B, 0x0C, 0x0D, 0x0E, 0x0F,
        0x10,
    ])
}

/// 存储一个密钥。
/// 在实际应用中，这会将密钥加密并持久化存储。
pub fn store_key(key: &[u8]) -> Result<(), KeyManagementError> {
    if key.is_empty() {
        return Err(KeyManagementError::InvalidKey("Key cannot be empty".to_string()));
    }
    let mut storage = KEY_STORAGE
        .lock()
        .map_err(|e| KeyManagementError::KeyStorageFailed(e.to_string()))?;
    *storage = Some(key.to_vec());
    Ok(())
}

/// 检索存储的密钥。
/// 在实际应用中，这会从持久化存储中读取并解密密钥。
pub fn retrieve_key() -> Result<Vec<u8>, KeyManagementError> {
    let storage = KEY_STORAGE
        .lock()
        .map_err(|e| KeyManagementError::KeyStorageFailed(e.to_string()))?;
    storage.clone().ok_or(KeyManagementError::KeyNotFound)
}

/// 清除所有存储的密钥。
/// 在实际应用中，这会安全地擦除持久化存储中的密钥。
pub fn clear_keys() -> Result<(), KeyManagementError> {
    let mut storage = KEY_STORAGE
        .lock()
        .map_err(|e| KeyManagementError::KeyStorageFailed(e.to_string()))?;
    *storage = None;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_key() {
        let key = generate_key().unwrap();
        assert!(!key.is_empty());
        assert_eq!(key.len(), 16); // 假设生成16字节密钥
    }

    #[test]
    fn test_store_key() {
        clear_keys().unwrap(); // 确保测试前状态干净
        let key = vec![1, 2, 3];
        store_key(&key).unwrap();
        let retrieved = retrieve_key().unwrap();
        assert_eq!(retrieved, key);
    }

    #[test]
    fn test_store_key_empty() {
        clear_keys().unwrap(); // 确保测试前状态干净
        assert!(store_key(&[]).is_err());
    }

    #[test]
    fn test_retrieve_key_not_found() {
        clear_keys().unwrap(); // 确保没有密钥
        assert!(retrieve_key().is_err());
    }
}