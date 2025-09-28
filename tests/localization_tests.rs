//! tests/localization_tests.rs
//!
//! 测试 localization.rs 的功能
//! 覆盖：基本翻译、回退机制、边界情况

use defi_hot_wallet::i18n::localization::translate;

// 注意：这些测试是集成测试，它们依赖于 `resources/i18n/` 目录下的 `en.ftl` 和 `zh.ftl` 文件。
// 假设 en.ftl 包含: hello = Hello, World!
// 假设 zh.ftl 包含: hello = 你好，世界！

#[test]
fn test_translate_english() {
    // 测试基本英文翻译
    let result = translate("hello", "en");
    assert_eq!(result, "Hello, World!");
}

#[test]
fn test_translate_chinese() {
    // 测试基本中文翻译
    let result = translate("hello", "zh");
    assert_eq!(result, "你好，世界！");
}

#[test]
fn test_translate_fallback_to_default_language() {
    // 测试当语言不存在时，回退到默认语言 "en"
    let result = translate("hello", "fr"); // "fr" (法语) 不存在
    assert_eq!(result, "Hello, World!");
}

#[test]
fn test_translate_missing_key() {
    // 测试当 key 不存在时，应返回 key 本身
    let result = translate("missing_key_for_test", "en");
    assert_eq!(result, "missing_key_for_test");
}

#[test]
fn test_translate_empty_key() {
    // 边缘情况：测试空 key
    let result = translate("", "en");
    assert_eq!(result, "");
}