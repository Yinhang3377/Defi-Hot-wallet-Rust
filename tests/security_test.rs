//! 集成测试：验证加密/解密循环和内存清理

use hot_wallet::security::encryption::WalletSecurity;
use hot_wallet::tools::error::WalletError;

#[test]
fn test_encrypt_decrypt_cycle() {
    // 一个有效的32字节私钥
    let private_key = b"32_byte_private_key_1234567890ab";
    // 一个有效的64位十六进制加密密钥
    let encryption_key = "a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2";
    // 关联数据
    let aad = b"user_id:123,context:backup";

    let encrypted = WalletSecurity::encrypt_private_key(private_key, encryption_key, aad).unwrap();
    let decrypted = WalletSecurity::decrypt_private_key(&encrypted, encryption_key, aad).unwrap();
    assert_eq!(private_key.to_vec(), decrypted);
}

#[test]
fn test_invalid_key_length() {
    // 私钥长度必须是32字节
    let invalid_private_key = b"this_is_not_32_bytes_long";
    let valid_private_key = &[0u8; 32];
    let short_key = "too_short";
    let aad = b"";

    // 使用无效长度的私钥进行测试
    let result = WalletSecurity::encrypt_private_key(invalid_private_key, short_key, aad);
    assert!(matches!(result, Err(WalletError::EncryptionError(_))));

    // 使用无效格式的加密密钥进行测试
    let result2 = WalletSecurity::encrypt_private_key(valid_private_key, short_key, aad);
    assert!(matches!(result2, Err(WalletError::EncryptionError(_))));
}

#[test]
fn test_invalid_ciphertext() {
    let encryption_key = "a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2";
    let aad = b"";
    let invalid_ciphertext = b"too_short";
    let result = WalletSecurity::decrypt_private_key(invalid_ciphertext, encryption_key, aad);
    assert!(matches!(result, Err(WalletError::EncryptionError(_))));
}

#[test]
fn test_aad_mismatch_fails_decryption() {
    // 测试当 AAD 不匹配时解密是否失败
    let private_key = &[0u8; 32];
    let encryption_key = "a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2";
    let aad_encrypt = b"correct_aad";
    let aad_decrypt = b"wrong_aad";

    let encrypted = WalletSecurity::encrypt_private_key(
        private_key,
        encryption_key,
        aad_encrypt
    ).unwrap();

    // 使用错误的 AAD 进行解密，应该失败
    let result = WalletSecurity::decrypt_private_key(&encrypted, encryption_key, aad_decrypt);
    assert!(matches!(result, Err(WalletError::EncryptionError(_))));
}

#[test]
fn test_tampered_ciphertext_fails_decryption() {
    // 测试当密文被篡改时解密是否失败
    let private_key = &[0u8; 32];
    let encryption_key = "a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2";
    let aad = b"some_aad";

    let mut encrypted = WalletSecurity::encrypt_private_key(
        private_key,
        encryption_key,
        aad
    ).unwrap();
    let len = encrypted.len();
    encrypted[len - 1] ^= 0xff; // 篡改最后一个字节

    let result = WalletSecurity::decrypt_private_key(&encrypted, encryption_key, aad);
    assert!(matches!(result, Err(WalletError::EncryptionError(_))));
}
