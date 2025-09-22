use defi_hot_wallet::mvp::{confirm_transaction, create_transaction, send_transaction};

#[test]
fn tx_send_and_confirm() {
    let tx = create_transaction();
    let tx_hash = send_transaction(tx).expect("send ok");
    let confirmed = confirm_transaction(tx_hash).expect("confirm ok");
    assert!(confirmed);
}
