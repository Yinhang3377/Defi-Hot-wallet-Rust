//! tests/audit_rollback_tests.rs
//!
//! 针对 `src/audit/rollback.rs` 的单元测试。

use defi_hot_wallet::audit::rollback::*;

#[test]
fn test_rollback_new() {
    // 正常路径：测试创建新的回滚请求
    let rollback = Rollback::new("tx_id_to_revert");
    assert_eq!(rollback.tx_id, "tx_id_to_revert");
}

/// 测试 `rollback_tx` 占位函数。
/// 这个测试验证了占位函数当前总是返回成功 (`Ok(())`)，
/// 确保了即使在模拟实现下，其行为也是可预测的。
#[test]
fn test_rollback_tx_function() {
    assert_eq!(rollback_tx("any_tx_id"), Ok(()));
}