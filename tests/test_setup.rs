use ctor::ctor;
use std::env;

#[ctor]
fn test_setup() {
    // 如果环境变量未设置，则提供一个默认值，以确保本地测试和 CI 的行为一致。
    // 这个函数会在测试开始前运行。

    // 与 CI 保持一致的确定性 key（32 bytes zeros base64）
    if env::var("WALLET_ENC_KEY").is_err() {
        env::set_var(
            "WALLET_ENC_KEY",
            "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=",
        );
    }

    // 让测试在需要时跳过实际解密（与 CI 同步）
    if env::var("TEST_SKIP_DECRYPT").is_err() {
        env::set_var("TEST_SKIP_DECRYPT", "1");
    }

    // Mock bridge behavior for deterministic tests
    if env::var("BRIDGE_MOCK_FORCE_SUCCESS").is_err() {
        env::set_var("BRIDGE_MOCK_FORCE_SUCCESS", "1");
    }
    if env::var("BRIDGE_MOCK").is_err() {
        env::set_var("BRIDGE_MOCK", "1");
    }

    // Debug helper to confirm setup ran
    eprintln!("test_setup: WALLET_ENC_KEY set; TEST_SKIP_DECRYPT={}", env::var("TEST_SKIP_DECRYPT").unwrap_or_default());
}