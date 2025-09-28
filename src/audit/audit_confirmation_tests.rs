//! tests/audit_confirmation_tests.rs
//!
//! 针对 `src/audit/confirmation.rs` 的单元测试。

use defi_hot_wallet::audit::confirmation::*;

#[test]
fn test_confirmation_new() {
    // 正常路径：测试新创建的确认请求
    let confirmation = Confirmation::new("tx_id_123");
    assert_eq!(confirmation.tx_id, "tx_id_123");
    // 验证初始状态为未确认
    assert!(!confirmation.is_confirmed());
}

#[test]
fn test_confirmation_confirm_and_check() {
    // 正常路径：测试确认流程
    let mut confirmation = Confirmation::new("tx_id_456");

    // 初始状态
    assert!(!confirmation.is_confirmed(), "Should not be confirmed initially");

    // 确认操作
    confirmation.confirm();

    // 验证最终状态
    assert!(confirmation.is_confirmed(), "Should be confirmed after calling confirm()");
}

#[test]
fn test_require_confirmation_placeholder() {
    // 正常路径：测试占位函数总是返回 true
    assert!(require_confirmation("any_operation"));
    assert!(require_confirmation(""));
}