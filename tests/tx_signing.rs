use defi_hot_wallet::mvp::{
    create_transaction, derive_public_key, generate_private_key, is_signature_valid,
    sign_transaction, verify_signature,
};

#[test]
fn tx_signing_roundtrip() {
    let tx = create_transaction();
    let private_key = generate_private_key();
    let public_key = derive_public_key(&private_key);
    let signature = sign_transaction(&tx, &private_key);

    assert!(verify_signature(&tx, &signature, &public_key));
    assert!(is_signature_valid(&signature, &public_key));
}
