use ctor::ctor;
use std::env;

#[ctor]
fn test_setup() {
    // 在测试进程最早阶段设置，避免 AES 解密/桥接 mock race
    env::set_var("TEST_SKIP_DECRYPT", "1");
    env::set_var("BRIDGE_MOCK_FORCE_SUCCESS", "1");
    // 如需调试可额外打印（测试运行时会显式输出）
    // eprintln!("test_setup: TEST_SKIP_DECRYPT and BRIDGE_MOCK_FORCE_SUCCESS set");
}
