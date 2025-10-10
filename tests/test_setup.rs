use base64::engine::general_purpose::STANDARD as BASE64_ENGINE;
use base64::Engine as _;
use ctor::ctor;
use std::env;

#[ctor]
fn test_setup() {
    // Skip decrypt variants used in repo
    env::set_var("TEST_SKIP_DECRYPT", "1");
    env::set_var("SKIP_DECRYPT", "1");
    env::set_var("WALLET_SKIP_DECRYPT", "1");
    env::set_var("BRIDGE_SKIP_DECRYPT", "1");

    // Force bridge mock variants
    env::set_var("BRIDGE_MOCK_FORCE_SUCCESS", "1");
    env::set_var("BRIDGE_MOCK", "1");
    env::set_var("MOCK_BRIDGE_FORCE_SUCCESS", "1");
    env::set_var("FORCE_MOCK_BRIDGE", "1");

    // Valid base64 32-byte key to avoid AES errors where code checks WALLET_ENC_KEY
    let key = vec![0u8; 32];
    let b64 = BASE64_ENGINE.encode(&key);
    env::set_var("WALLET_ENC_KEY", b64);

    eprintln!("test_setup: applied skip-decrypt & bridge-mock envs");
}
