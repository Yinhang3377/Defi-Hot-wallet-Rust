// filepath: c:\Users\plant\Desktop\Rust区块链\Defi-Hot-wallet-Rust\tests\ops_health_tests.rs

use defi_hot_wallet::ops::health::{health_check, HealthCheck};

#[test]
fn test_health_check_struct_new_and_is_healthy() {
    // 正常路径：测试 HealthCheck::new() 和 is_healthy() 方法
    let health = HealthCheck::new();
    assert!(health.is_healthy(), "HealthCheck::is_healthy should return true");
}

#[test]
fn test_health_check_struct_default() {
    // 正常路径：测试 HealthCheck 的 Default trait 实现
    let health = HealthCheck::default();
    assert!(health.is_healthy(), "Default HealthCheck instance should be healthy");
}

#[test]
fn test_standalone_health_check_function() {
    // 正常路径：测试独立的 health_check() 函数
    // 这个测试覆盖了 `health_check` 函数本身
    assert!(health_check(), "The standalone health_check function should return true");
}