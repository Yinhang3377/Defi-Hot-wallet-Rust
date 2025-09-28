//! tests/tools_error_tests.rs
//!
//! 测试 `src/tools/error.rs` 的功能。
//! 覆盖：
//! - 错误类型的创建和格式化
//! - `is_critical` 方法的分类
//! - `is_retryable` 方法的分类
//! - `error_code` 方法的正确性
//! - `From<std::io::Error>` 的转换

use defi_hot_wallet::tools::error::WalletError;
use std::io;

#[test]
fn test_error_creation_and_display() {
    // 正常路径：测试各种错误的创建和它们的 Display 实现
    let err = WalletError::InvalidInput("test input".to_string());
    assert_eq!(format!("{}", err), "Invalid input: test input");

    let err = WalletError::NetworkError("connection timed out".to_string());
    assert_eq!(format!("{}", err), "Network error: connection timed out");

    let err = WalletError::new("a generic error occurred");
    assert_eq!(format!("{}", err), "Generic error: a generic error occurred");
}

#[test]
fn test_is_critical_classification() {
    // 正常路径：测试哪些错误被归类为严重错误
    assert!(WalletError::SecurityError("...".to_string()).is_critical());
    assert!(WalletError::MemoryError("...".to_string()).is_critical());
    assert!(WalletError::AuthenticationError("...".to_string()).is_critical());
    assert!(WalletError::ComplianceError("...".to_string()).is_critical());

    // 错误路径：测试非严重错误
    assert!(!WalletError::NetworkError("...".to_string()).is_critical());
    assert!(!WalletError::InvalidInput("...".to_string()).is_critical());
    assert!(!WalletError::GenericError("...".to_string()).is_critical());
}

#[test]
fn test_is_retryable_classification() {
    // 正常路径：测试哪些错误被归类为可重试错误
    assert!(WalletError::NetworkError("...".to_string()).is_retryable());
    assert!(WalletError::TimeoutError("...".to_string()).is_retryable());
    assert!(WalletError::RateLimitError("...".to_string()).is_retryable());

    // 错误路径：测试不可重试错误
    assert!(!WalletError::InvalidInput("...".to_string()).is_retryable());
    assert!(!WalletError::SecurityError("...".to_string()).is_retryable());
    assert!(!WalletError::GenericError("...".to_string()).is_retryable());
}

#[test]
fn test_error_code_mapping() {
    // 正常路径：验证每个错误变体是否映射到正确的错误代码字符串
    assert_eq!(
        WalletError::IoError(io::Error::new(io::ErrorKind::NotFound, "test")).error_code(),
        "IO_ERROR"
    );
    assert_eq!(
        WalletError::SerializationError("...".to_string()).error_code(),
        "SERIALIZATION_ERROR"
    );
    assert_eq!(
        WalletError::DecryptionError("...".to_string()).error_code(),
        "DECRYPTION_ERROR"
    );
    assert_eq!(
        WalletError::InvalidInput("...".to_string()).error_code(),
        "INVALID_INPUT"
    );
    assert_eq!(
        WalletError::NetworkError("...".to_string()).error_code(),
        "NETWORK_ERROR"
    );
    assert_eq!(
        WalletError::DatabaseError("...".to_string()).error_code(),
        "DATABASE_ERROR"
    );
    assert_eq!(
        WalletError::SecurityError("...".to_string()).error_code(),
        "SECURITY_ERROR"
    );
    assert_eq!(
        WalletError::NotFoundError("...".to_string()).error_code(),
        "NOT_FOUND_ERROR"
    );
    assert_eq!(
        WalletError::MnemonicError("...".to_string()).error_code(),
        "MNEMONIC_ERROR"
    );
    assert_eq!(
        WalletError::GenericError("...".to_string()).error_code(),
        "GENERIC_ERROR"
    );
}

#[test]
fn test_from_io_error_conversion() {
    // 正常路径：测试 `From<std::io::Error>` trait 实现
    let io_error = io::Error::new(io::ErrorKind::PermissionDenied, "access denied");
    let wallet_error: WalletError = io_error.into();

    assert!(matches!(wallet_error, WalletError::IoError(_)));
    assert_eq!(
        format!("{}", wallet_error),
        "IO error: access denied"
    );
}