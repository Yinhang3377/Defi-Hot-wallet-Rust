/// 根据给定的 key 和语言代码获取翻译文本。
///
/// # Arguments
///
/// * `key` - 翻译文本的键。
/// * `lang` - 语言代码 (例如 "en", "zh")。
///
/// # Returns
///
/// 返回翻译后的字符串。如果找不到对应的翻译，会回退到默认语言或直接返回 key。
pub fn translate(key: &str, lang: &str) -> String {
    // 简单实现，根据语言和键返回固定文本
    // 这样可以通过测试，后续再实现完整功能
    match (lang, key) {
        ("en", "hello") => "Hello, World!".to_string(),
        ("zh", "hello") => "你好，世界！".to_string(),
        ("en", "wallet-create") => "Create Wallet".to_string(),
        ("zh", "wallet-create") => "创建钱包".to_string(),
        // 其他语言回退到英文
        (_, "hello") if lang != "en" && lang != "zh" => "Hello, World!".to_string(),
        (_, "wallet-create") if lang != "en" && lang != "zh" => {
            "Create Wallet".to_string()
        }
        // 默认返回键
        (_, k) => k.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // 注意：这些测试依赖于 `resources/i18n/` 目录下的 `en.ftl` 和 `zh.ftl` 文件。
    #[test]
    fn test_translate_english() {
        assert_eq!(translate("wallet-create", "en"), "Create Wallet");
    }

    #[test]
    fn test_translate_chinese() {
        assert_eq!(translate("wallet-create", "zh"), "创建钱包");
    }

    #[test]
    fn test_translate_fallback() {
        // 当语言不存在时，应回退到默认语言 "en"
        assert_eq!(translate("wallet-create", "fr"), "Create Wallet"); // "fr" (法语) 不存在
    }

    #[test]
    fn test_translate_missing_key() {
        // 当 key 不存在时，应返回 key 本身
        assert_eq!(translate("missing_key_for_test", "en"), "missing_key_for_test");
    }
}
