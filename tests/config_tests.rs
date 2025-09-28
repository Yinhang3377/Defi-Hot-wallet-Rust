use defi_hot_wallet::config::env_config;

#[test]
fn test_env_config_load_with_values() {
    // 为测试设置环境变量
    std::env::set_var("APP_SOME_FIELD", "test_value");
    std::env::set_var("APP_ANOTHER_FIELD", "123");

    let config = env_config::load().unwrap();
    assert_eq!(config.some_field, Some("test_value".to_string()));
    assert_eq!(config.another_field, Some(123));

    // 取消设置环境变量，以避免影响其他测试
    std::env::remove_var("APP_SOME_FIELD");
    std::env::remove_var("APP_ANOTHER_FIELD");
}

#[test]
#[serial_test::serial] // 添加此行以确保测试串行执行
fn test_env_config_load_defaults_no_env_vars() {
    // 确保环境变量未设置
    std::env::remove_var("APP_SOME_FIELD");
    std::env::remove_var("APP_ANOTHER_FIELD");

    let config = env_config::load().unwrap();
    // 验证字段是否为 None，使用 is_none() 更具可读性
    assert!(config.some_field.is_none());
    assert!(config.another_field.is_none());
}
