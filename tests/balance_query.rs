use defi_hot_wallet::mvp::query_balance;

#[test]
fn balance_query() {
    let account = "0x0000000000000000000000000000000000000000";
    let actual_balance = query_balance(account);
    assert_eq!(actual_balance, 0); // 固定桩值，避免无效比较
}
