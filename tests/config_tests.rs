use defi_hot_wallet::config::env_config;

#[test]
fn test_env_config_load_with_values() {
    // 涓烘祴璇曡缃幆澧冨彉閲?    std::env::set_var("APP_SOME_FIELD", "test_value");
    std::env::set_var("APP_ANOTHER_FIELD", "123");

    let config = env_config::load().unwrap();
    assert_eq!(config.some_field, Some("test_value".to_string()));
    assert_eq!(config.another_field, Some(123));

    // 鍙栨秷璁剧疆鐜鍙橀噺锛屼互閬垮厤褰卞搷鍏朵粬娴嬭瘯
    std::env::remove_var("APP_SOME_FIELD");
    std::env::remove_var("APP_ANOTHER_FIELD");
}

#[test]
#[serial_test::serial] // 娣诲姞姝よ浠ョ‘淇濇祴璇曚覆琛屾墽琛?fn test_env_config_load_defaults_no_env_vars() {
    // 纭繚鐜鍙橀噺鏈缃?    std::env::remove_var("APP_SOME_FIELD");
    std::env::remove_var("APP_ANOTHER_FIELD");

    let config = env_config::load().unwrap();
    // 楠岃瘉瀛楁鏄惁涓?None锛屼娇鐢?is_none() 鏇村叿鍙鎬?    assert!(config.some_field.is_none());
    assert!(config.another_field.is_none());
}
