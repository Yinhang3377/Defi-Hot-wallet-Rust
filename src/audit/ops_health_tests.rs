// filepath: c:\Users\plant\Desktop\Rust区块链\Defi-Hot-wallet-Rust\tests\ops_health_tests.rs

use defi_hot_wallet::ops::health::*;

#[test]
fn test_health_check() {
    let health = HealthCheck::new();
    assert!(health.is_healthy());
}