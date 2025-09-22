use defi_hot_wallet::mvp::create_wallet;

#[test]
fn wallet_generation() {
    let wallet = create_wallet("test_password");
    assert!(wallet.address.starts_with("0x"));
    assert_eq!(wallet.address.len(), 42);
}
