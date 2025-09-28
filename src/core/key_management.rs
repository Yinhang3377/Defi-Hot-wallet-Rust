use anyhow::Result;
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::sync::Mutex;
use uuid::Uuid;

/// 为了测试目的，使用一个简单的内存哈希映射来存储密钥
static KEY_STORAGE: Lazy<Mutex<HashMap<String, Vec<u8>>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

/// 生成一个新的密钥对。
/// 这是一个简化实现，使用 UUID 生成唯一密钥。
pub fn generate_key() -> Result<Vec<u8>> {
    Ok(Uuid::new_v4().as_bytes().to_vec())
}

/// 存储一个密钥并返回一个唯一的ID。
pub fn store_key(key: &[u8]) -> Result<String> {
    let id = Uuid::new_v4().to_string();
    let mut storage = KEY_STORAGE.lock().unwrap();
    storage.insert(id.clone(), key.to_vec());
    Ok(id)
}

/// 根据ID检索密钥。
pub fn retrieve_key(id: &str) -> Result<Vec<u8>> {
    let storage = KEY_STORAGE.lock().unwrap();
    storage.get(id).cloned().ok_or_else(|| anyhow::anyhow!("Key not found for id: {}", id))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn test_generate_key() {
        let key = generate_key().unwrap();
        assert!(!key.is_empty());
        assert_eq!(key.len(), 16); // 基于简化实现
    }

    #[test]
    fn test_store_and_retrieve_key() {
        let key = generate_key().unwrap();
        let id = store_key(&key).unwrap();
        let retrieved = retrieve_key(&id).unwrap();
        assert_eq!(key, retrieved);
    }

    #[test]
    fn test_retrieve_key_not_found() {
        let result = retrieve_key("nonexistent");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Key not found"));
    }

    #[test]
    fn test_store_empty_key() {
        let key = Vec::<u8>::new(); // 修复类型推断
        let id = store_key(&key).unwrap();
        let retrieved = retrieve_key(&id).unwrap();
        assert_eq!(retrieved, Vec::<u8>::new());
    }

    #[test]
    fn test_store_large_key() {
        let key = vec![0; 1000]; // 大密钥
        let id = store_key(&key).unwrap();
        let retrieved = retrieve_key(&id).unwrap();
        assert_eq!(retrieved, key);
    }

    #[test]
    fn test_concurrent_access() {
        let key1 = generate_key().unwrap();
        let key2 = generate_key().unwrap();

        let key1_clone = key1.clone(); // 克隆以避免移动
        let handle1 = thread::spawn(move || {
            let id = store_key(&key1_clone).unwrap();
            retrieve_key(&id).unwrap()
        });

        let key2_clone = key2.clone(); // 克隆以避免移动
        let handle2 = thread::spawn(move || {
            let id = store_key(&key2_clone).unwrap();
            retrieve_key(&id).unwrap()
        });

        let retrieved1 = handle1.join().unwrap();
        let retrieved2 = handle2.join().unwrap();

        assert!([&retrieved1, &retrieved2].contains(&&key1));
        assert!([&retrieved1, &retrieved2].contains(&&key2));
        assert_ne!(retrieved1, retrieved2);
    }

    #[test]
    fn test_multiple_keys() {
        let keys = vec![generate_key().unwrap(), generate_key().unwrap(), generate_key().unwrap()];

        let ids: Vec<String> = keys.iter().map(|k| store_key(k).unwrap()).collect();

        for (i, id) in ids.iter().enumerate() {
            let retrieved = retrieve_key(id).unwrap();
            assert_eq!(retrieved, keys[i]);
        }
    }

    #[test]
    fn test_retrieve_after_clear() {
        // 注意：这个测试假设没有清空功能，但测试边缘情况
        let key = generate_key().unwrap();
        let id = store_key(&key).unwrap();
        let retrieved = retrieve_key(&id).unwrap();
        assert_eq!(retrieved, key);

        // 模拟清空（如果有清空功能，可以添加）
        // 但当前实现不支持，所以跳过
    }
}
