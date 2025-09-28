//! tests/ops_backup_tests.rs
//!
//! 针对 `src/ops/backup.rs` 的单元测试。

use defi_hot_wallet::ops::backup::*;

#[test]
fn test_backup_create() {
    // 正常路径：测试创建新的备份任务
    let backup = Backup::new("my_precious_wallet");
    assert_eq!(backup.wallet_name, "my_precious_wallet");
}

#[test]
fn test_perform_backup_function() {
    // 正常路径：测试占位函数总是成功
    let backup = Backup::new("any_wallet");
    assert_eq!(perform_backup(&backup), Ok(()));
}