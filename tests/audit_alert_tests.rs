// filepath: c:\Users\plant\Desktop\Rust鍖哄潡閾綷Defi-Hot-wallet-Rust\tests\audit_alert_tests.rs

use defi_hot_wallet::audit::alert::*;

#[test]
fn test_alert_basic() {
    let alert = Alert::new("test message");
    assert_eq!(alert.message, "test message");
}
