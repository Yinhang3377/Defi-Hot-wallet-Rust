use ctor::ctor;
use std::env;

#[ctor]
fn init_test_env() {
    // 32 zero bytes base64
    env::set_var("WALLET_ENC_KEY", "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=");
    env::set_var("TEST_SKIP_DECRYPT", "1");
    env::set_var("BRIDGE_MOCK_FORCE_SUCCESS", "1");
    env::set_var("BRIDGE_MOCK", "1");
    eprintln!("test-env feature active: test env variables set");
}
