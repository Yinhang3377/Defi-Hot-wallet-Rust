use defi_hot_wallet::audit::confirmation::*;

#[test]
fn test_confirmation_new() {
    let confirmation = Confirmation::new("tx_id");
    assert_eq!(confirmation.tx_id, "tx_id");
    assert!(!confirmation.is_confirmed());  // 覆盖初始 confirmed = false
}

#[test]
fn test_confirmation_confirm() {
    let mut confirmation = Confirmation::new("tx_id");
    confirmation.confirm();  // 覆盖 confirm 方法
    assert!(confirmation.is_confirmed());  // 覆盖 is_confirmed 返回 true
}

#[test]
fn test_require_confirmation() {
    assert!(require_confirmation("some_op"));  // 覆盖 require_confirmation 函数
}