//! tests/tools_error_tests.rs
//!
//! 娴嬭瘯 `src/tools/error.rs` 鐨勫姛鑳姐€?//! 瑕嗙洊锛?//! - 閿欒绫诲瀷鐨勫垱寤哄拰鏍煎紡鍖?//! - `is_critical` 鏂规硶鐨勫垎绫?//! - `is_retryable` 鏂规硶鐨勫垎绫?//! - `error_code` 鏂规硶鐨勬纭€?//! - `From<std::io::Error>` 鐨勮浆鎹?
use defi_hot_wallet::tools::error::WalletError;
use std::io;

#[test]
fn test_error_creation_and_display() {
    // 姝ｅ父璺緞锛氭祴璇曞悇绉嶉敊璇殑鍒涘缓鍜屽畠浠殑 Display 瀹炵幇
    let err = WalletError::InvalidInput("test input".to_string());
    assert_eq!(format!("{}", err), "Invalid input: test input");

    let err = WalletError::NetworkError("connection timed out".to_string());
    assert_eq!(format!("{}", err), "Network error: connection timed out");

    let err = WalletError::new("a generic error occurred");
    assert_eq!(format!("{}", err), "Generic error: a generic error occurred");
}

#[test]
fn test_is_critical_classification() {
    // 姝ｅ父璺緞锛氭祴璇曞摢浜涢敊璇褰掔被涓轰弗閲嶉敊璇?    assert!(WalletError::SecurityError("...".to_string()).is_critical());
    assert!(WalletError::MemoryError("...".to_string()).is_critical());
    assert!(WalletError::AuthenticationError("...".to_string()).is_critical());
    assert!(WalletError::ComplianceError("...".to_string()).is_critical());

    // 閿欒璺緞锛氭祴璇曢潪涓ラ噸閿欒
    assert!(!WalletError::NetworkError("...".to_string()).is_critical());
    assert!(!WalletError::InvalidInput("...".to_string()).is_critical());
    assert!(!WalletError::GenericError("...".to_string()).is_critical());
}

#[test]
fn test_is_retryable_classification() {
    // 姝ｅ父璺緞锛氭祴璇曞摢浜涢敊璇褰掔被涓哄彲閲嶈瘯閿欒
    assert!(WalletError::NetworkError("...".to_string()).is_retryable());
    assert!(WalletError::TimeoutError("...".to_string()).is_retryable());
    assert!(WalletError::RateLimitError("...".to_string()).is_retryable());

    // 閿欒璺緞锛氭祴璇曚笉鍙噸璇曢敊璇?    assert!(!WalletError::InvalidInput("...".to_string()).is_retryable());
    assert!(!WalletError::SecurityError("...".to_string()).is_retryable());
    assert!(!WalletError::GenericError("...".to_string()).is_retryable());
}

#[test]
fn test_error_code_mapping() {
    // 姝ｅ父璺緞锛氶獙璇佹瘡涓敊璇彉浣撴槸鍚︽槧灏勫埌姝ｇ‘鐨勯敊璇唬鐮佸瓧绗︿覆
    assert_eq!(
        WalletError::IoError(io::Error::new(io::ErrorKind::NotFound, "test")).error_code(),
        "IO_ERROR"
    );
    assert_eq!(
        WalletError::SerializationError("...".to_string()).error_code(),
        "SERIALIZATION_ERROR"
    );
    assert_eq!(WalletError::DecryptionError("...".to_string()).error_code(), "DECRYPTION_ERROR");
    assert_eq!(WalletError::InvalidInput("...".to_string()).error_code(), "INVALID_INPUT");
    assert_eq!(WalletError::NetworkError("...".to_string()).error_code(), "NETWORK_ERROR");
    assert_eq!(WalletError::DatabaseError("...".to_string()).error_code(), "DATABASE_ERROR");
    assert_eq!(WalletError::SecurityError("...".to_string()).error_code(), "SECURITY_ERROR");
    assert_eq!(WalletError::NotFoundError("...".to_string()).error_code(), "NOT_FOUND_ERROR");
    assert_eq!(WalletError::MnemonicError("...".to_string()).error_code(), "MNEMONIC_ERROR");
    assert_eq!(WalletError::GenericError("...".to_string()).error_code(), "GENERIC_ERROR");
}

#[test]
fn test_from_io_error_conversion() {
    // 姝ｅ父璺緞锛氭祴璇?`From<std::io::Error>` trait 瀹炵幇
    let io_error = io::Error::new(io::ErrorKind::PermissionDenied, "access denied");
    let wallet_error: WalletError = io_error.into();

    assert!(matches!(wallet_error, WalletError::IoError(_)));
    assert_eq!(format!("{}", wallet_error), "IO error: access denied");
}
