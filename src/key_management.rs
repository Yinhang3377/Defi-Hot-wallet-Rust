use std::sync::Mutex;

// 涓轰簡娴嬭瘯鐩殑锛屼娇鐢ㄤ竴涓畝鍗曠殑鍐呭瓨瀛樺偍
// 鍦ㄥ疄闄呭簲鐢ㄤ腑锛岃繖浼氭槸涓€涓畨鍏ㄧ殑銆佹寔涔呭寲鐨勫瓨鍌ㄦ満鍒?
static KEY_STORAGE: Mutex<Option<Vec<u8>>> = Mutex::new(None);

/// 瀵嗛挜绠＄悊鐩稿叧鐨勯敊璇被鍨?
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

/// 鐢熸垚涓€涓柊鐨勫瘑閽ャ€?
/// 鍦ㄥ疄闄呭簲鐢ㄤ腑锛岃繖浼氫娇鐢ㄤ竴涓瘑鐮佸瀹夊叏鐨勯殢鏈烘暟鐢熸垚鍣ㄣ€?
pub fn generate_key() -> Result<Vec<u8>, KeyManagementError> {
    // 绀轰緥锛氱敓鎴愪竴涓?6瀛楄妭鐨勫瘑閽?
    // 瀹為檯搴旂敤涓簲浣跨敤 `rand::Rng` 鍜?`rand::thread_rng()`
    Ok(vec![
        0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0A, 0x0B, 0x0C, 0x0D, 0x0E, 0x0F,
        0x10,
    ])
}

/// 瀛樺偍涓€涓瘑閽ャ€?
/// 鍦ㄥ疄闄呭簲鐢ㄤ腑锛岃繖浼氬皢瀵嗛挜鍔犲瘑骞舵寔涔呭寲瀛樺偍銆?
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

/// 妫€绱㈠瓨鍌ㄧ殑瀵嗛挜銆?
/// 鍦ㄥ疄闄呭簲鐢ㄤ腑锛岃繖浼氫粠鎸佷箙鍖栧瓨鍌ㄤ腑璇诲彇骞惰В瀵嗗瘑閽ャ€?
pub fn retrieve_key() -> Result<Vec<u8>, KeyManagementError> {
    let storage = KEY_STORAGE
        .lock()
        .map_err(|e| KeyManagementError::KeyStorageFailed(e.to_string()))?;
    storage.clone().ok_or(KeyManagementError::KeyNotFound)
}

/// 娓呴櫎鎵€鏈夊瓨鍌ㄧ殑瀵嗛挜銆?
/// 鍦ㄥ疄闄呭簲鐢ㄤ腑锛岃繖浼氬畨鍏ㄥ湴鎿﹂櫎鎸佷箙鍖栧瓨鍌ㄤ腑鐨勫瘑閽ャ€?
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
        assert_eq!(key.len(), 16); // 鍋囪鐢熸垚16瀛楄妭瀵嗛挜
    }

    #[test]
    fn test_store_key() {
        clear_keys().unwrap(); // 纭繚娴嬭瘯鍓嶇姸鎬佸共鍑€
        let key = vec![1, 2, 3];
        store_key(&key).unwrap();
        let retrieved = retrieve_key().unwrap();
        assert_eq!(retrieved, key);
    }

    #[test]
    fn test_store_key_empty() {
        clear_keys().unwrap(); // 纭繚娴嬭瘯鍓嶇姸鎬佸共鍑€
        assert!(store_key(&[]).is_err());
    }

    #[test]
    fn test_retrieve_key_not_found() {
        clear_keys().unwrap(); // 纭繚娌℃湁瀵嗛挜
        assert!(retrieve_key().is_err());
    }
}
