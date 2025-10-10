use base64::engine::general_purpose::STANDARD as BASE64_ENGINE;
use base64::Engine as _;
use ctor::ctor;
use std::env;

#[ctor]
fn init_test_env() {
    let key = vec![0u8; 32];
    let b64 = BASE64_ENGINE.encode(&key);
    env::set_var("WALLET_ENC_KEY", b64);
    env::set_var("TEST_SKIP_DECRYPT", "1");
    env::set_var("BRIDGE_MOCK_FORCE_SUCCESS", "1");
    env::set_var("BRIDGE_MOCK", "1");
}
