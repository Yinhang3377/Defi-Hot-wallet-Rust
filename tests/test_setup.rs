use ctor::ctor;
use std::env;

#[ctor]
fn test_setup() {
    // 32 zero bytes base64 - 与 CI 保持一致
    env::set_var(
        "WALLET_ENC_KEY",
        env::var("WALLET_ENC_KEY")
            .unwrap_or_else(|_| "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=".into()),
    );
    env::set_var("TEST_SKIP_DECRYPT", env::var("TEST_SKIP_DECRYPT").unwrap_or_else(|_| "1".into()));
    env::set_var(
        "BRIDGE_MOCK_FORCE_SUCCESS",
        env::var("BRIDGE_MOCK_FORCE_SUCCESS").unwrap_or_else(|_| "1".into()),
    );
    env::set_var("BRIDGE_MOCK", env::var("BRIDGE_MOCK").unwrap_or_else(|_| "1".into()));
    // optional debug
    eprintln!(
        "test_setup: TEST_SKIP_DECRYPT={}",
        env::var("TEST_SKIP_DECRYPT").unwrap_or_default()
    );
}
