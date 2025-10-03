use anyhow::Result;
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::sync::Mutex;
use uuid::Uuid;

/// 涓轰簡娴嬭瘯鐩殑锛屼娇鐢ㄤ竴涓畝鍗曠殑鍐呭瓨鍝堝笇鏄犲皠鏉ュ瓨鍌ㄥ瘑閽?static KEY_STORAGE: Lazy<Mutex<HashMap<String, Vec<u8>>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

/// 鐢熸垚涓€涓柊鐨勫瘑閽ュ銆?/// 杩欐槸涓€涓畝鍖栧疄鐜帮紝浣跨敤 UUID 鐢熸垚鍞竴瀵嗛挜銆?pub fn generate_key() -> Result<Vec<u8>> {
    Ok(Uuid::new_v4().as_bytes().to_vec())
}

/// 瀛樺偍涓€涓瘑閽ュ苟杩斿洖涓€涓敮涓€鐨処D銆?pub fn store_key(key: &[u8]) -> Result<String> {
    let id = Uuid::new_v4().to_string();
    let mut storage = KEY_STORAGE.lock().unwrap();
    storage.insert(id.clone(), key.to_vec());
    Ok(id)
}

/// 鏍规嵁ID妫€绱㈠瘑閽ャ€?pub fn retrieve_key(id: &str) -> Result<Vec<u8>> {
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
        assert_eq!(key.len(), 16); // 鍩轰簬绠€鍖栧疄鐜?    }

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
        let key = Vec::<u8>::new(); // 淇绫诲瀷鎺ㄦ柇
        let id = store_key(&key).unwrap();
        let retrieved = retrieve_key(&id).unwrap();
        assert_eq!(retrieved, Vec::<u8>::new());
    }

    #[test]
    fn test_store_large_key() {
        let key = vec![0; 1000]; // 澶у瘑閽?        let id = store_key(&key).unwrap();
        let retrieved = retrieve_key(&id).unwrap();
        assert_eq!(retrieved, key);
    }

    #[test]
    fn test_concurrent_access() {
        let key1 = generate_key().unwrap();
        let key2 = generate_key().unwrap();

        let key1_clone = key1.clone(); // 鍏嬮殕浠ラ伩鍏嶇Щ鍔?        let handle1 = thread::spawn(move || {
            let id = store_key(&key1_clone).unwrap();
            retrieve_key(&id).unwrap()
        });

        let key2_clone = key2.clone(); // 鍏嬮殕浠ラ伩鍏嶇Щ鍔?        let handle2 = thread::spawn(move || {
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
        // 娉ㄦ剰锛氳繖涓祴璇曞亣璁炬病鏈夋竻绌哄姛鑳斤紝浣嗘祴璇曡竟缂樻儏鍐?        let key = generate_key().unwrap();
        let id = store_key(&key).unwrap();
        let retrieved = retrieve_key(&id).unwrap();
        assert_eq!(retrieved, key);

        // 妯℃嫙娓呯┖锛堝鏋滄湁娓呯┖鍔熻兘锛屽彲浠ユ坊鍔狅級
        // 浣嗗綋鍓嶅疄鐜颁笉鏀寔锛屾墍浠ヨ烦杩?    }
}
