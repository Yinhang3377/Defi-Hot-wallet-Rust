use anyhow::Result;
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::sync::Mutex;
use uuid::Uuid;

/// In-memory key storage used for tests and simple runtimes.
/// Maps id -> key bytes.
static KEY_STORAGE: Lazy<Mutex<HashMap<String, Vec<u8>>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

/// Generate a fresh key (here we use a UUID as a 16-byte placeholder).
pub fn generate_key() -> Result<Vec<u8>> {
    Ok(Uuid::new_v4().as_bytes().to_vec())
}

/// Store a key and return a generated id.
pub fn store_key(key: &[u8]) -> Result<String> {
    let id = Uuid::new_v4().to_string();
    let mut storage = KEY_STORAGE.lock().unwrap();
    storage.insert(id.clone(), key.to_vec());
    Ok(id)
}

/// Retrieve a key by id.
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
        assert_eq!(key.len(), 16); // UUID v4 is 16 bytes
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
        let key = Vec::<u8>::new();
        let id = store_key(&key).unwrap();
        let retrieved = retrieve_key(&id).unwrap();
        assert_eq!(retrieved, Vec::<u8>::new());
    }

    #[test]
    fn test_store_large_key() {
        let key = vec![0u8; 1000];
        let id = store_key(&key).unwrap();
        let retrieved = retrieve_key(&id).unwrap();
        assert_eq!(retrieved, key);
    }

    #[test]
    fn test_concurrent_access() {
        let key1 = generate_key().unwrap();
        let key2 = generate_key().unwrap();

        let key1_clone = key1.clone();
        let handle1 = thread::spawn(move || {
            let id = store_key(&key1_clone).unwrap();
            retrieve_key(&id).unwrap()
        });

        let key2_clone = key2.clone();
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
    fn test_retrieve_after_clear_behavior() {
        // Basic store & retrieve sanity
        let key = generate_key().unwrap();
        let id = store_key(&key).unwrap();
        let retrieved = retrieve_key(&id).unwrap();
        assert_eq!(retrieved, key);

        // Note: current in-memory storage does not expose a clear API.
        // If clear functionality is added later, tests should be updated accordingly.
    }
}
