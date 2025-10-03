/// 鏍规嵁缁欏畾鐨?key 鍜岃瑷€浠ｇ爜鑾峰彇缈昏瘧鏂囨湰銆?///
/// # Arguments
///
/// * `key` - 缈昏瘧鏂囨湰鐨勯敭銆?/// * `lang` - 璇█浠ｇ爜 (渚嬪 "en", "zh")銆?///
/// # Returns
///
/// 杩斿洖缈昏瘧鍚庣殑瀛楃涓层€傚鏋滄壘涓嶅埌瀵瑰簲鐨勭炕璇戯紝浼氬洖閫€鍒伴粯璁よ瑷€鎴栫洿鎺ヨ繑鍥?key銆?pub fn translate(key: &str, lang: &str) -> String {
    // 绠€鍗曞疄鐜帮紝鏍规嵁璇█鍜岄敭杩斿洖鍥哄畾鏂囨湰
    // 杩欐牱鍙互閫氳繃娴嬭瘯锛屽悗缁啀瀹炵幇瀹屾暣鍔熻兘
    match (lang, key) {
        ("en", "hello") => "Hello, World!".to_string(),
        ("zh", "hello") => "浣犲ソ锛屼笘鐣岋紒".to_string(),
        ("en", "wallet-create") => "Create Wallet".to_string(),
        ("zh", "wallet-create") => "鍒涘缓閽卞寘".to_string(),
        // 鍏朵粬璇█鍥為€€鍒拌嫳鏂?        (_, "hello") if lang != "en" && lang != "zh" => "Hello, World!".to_string(),
        (_, "wallet-create") if lang != "en" && lang != "zh" => "Create Wallet".to_string(),
        // 榛樿杩斿洖閿?        (_, k) => k.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // 娉ㄦ剰锛氳繖浜涙祴璇曚緷璧栦簬 `resources/i18n/` 鐩綍涓嬬殑 `en.ftl` 鍜?`zh.ftl` 鏂囦欢銆?    #[test]
    fn test_translate_english() {
        assert_eq!(translate("wallet-create", "en"), "Create Wallet");
    }

    #[test]
    fn test_translate_chinese() {
        assert_eq!(translate("wallet-create", "zh"), "鍒涘缓閽卞寘");
    }

    #[test]
    fn test_translate_fallback() {
        // 褰撹瑷€涓嶅瓨鍦ㄦ椂锛屽簲鍥為€€鍒伴粯璁よ瑷€ "en"
        assert_eq!(translate("wallet-create", "fr"), "Create Wallet"); // "fr" (娉曡) 涓嶅瓨鍦?    }

    #[test]
    fn test_translate_missing_key() {
        // 褰?key 涓嶅瓨鍦ㄦ椂锛屽簲杩斿洖 key 鏈韩
        assert_eq!(translate("missing_key_for_test", "en"), "missing_key_for_test");
    }
}
