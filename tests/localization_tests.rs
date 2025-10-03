//! tests/localization_tests.rs
//!
//! 娴嬭瘯 localization.rs 鐨勫姛鑳?//! 瑕嗙洊锛氬熀鏈炕璇戙€佸洖閫€鏈哄埗銆佽竟鐣屾儏鍐?
use defi_hot_wallet::i18n::localization::translate;

// 娉ㄦ剰锛氳繖浜涙祴璇曟槸闆嗘垚娴嬭瘯锛屽畠浠緷璧栦簬 `resources/i18n/` 鐩綍涓嬬殑 `en.ftl` 鍜?`zh.ftl` 鏂囦欢銆?// 鍋囪 en.ftl 鍖呭惈: hello = Hello, World!
// 鍋囪 zh.ftl 鍖呭惈: hello = 浣犲ソ锛屼笘鐣岋紒

#[test]
fn test_translate_english() {
    // 娴嬭瘯鍩烘湰鑻辨枃缈昏瘧
    let result = translate("hello", "en");
    assert_eq!(result, "Hello, World!");
}

#[test]
fn test_translate_chinese() {
    // 娴嬭瘯鍩烘湰涓枃缈昏瘧
    let result = translate("hello", "zh");
    assert_eq!(result, "浣犲ソ锛屼笘鐣岋紒");
}

#[test]
fn test_translate_fallback_to_default_language() {
    // 娴嬭瘯褰撹瑷€涓嶅瓨鍦ㄦ椂锛屽洖閫€鍒伴粯璁よ瑷€ "en"
    let result = translate("hello", "fr"); // "fr" (娉曡) 涓嶅瓨鍦?    assert_eq!(result, "Hello, World!");
}

#[test]
fn test_translate_missing_key() {
    // 娴嬭瘯褰?key 涓嶅瓨鍦ㄦ椂锛屽簲杩斿洖 key 鏈韩
    let result = translate("missing_key_for_test", "en");
    assert_eq!(result, "missing_key_for_test");
}

#[test]
fn test_translate_empty_key() {
    // 杈圭紭鎯呭喌锛氭祴璇曠┖ key
    let result = translate("", "en");
    assert_eq!(result, "");
}
