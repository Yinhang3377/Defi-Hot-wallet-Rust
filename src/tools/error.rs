use thiserror::Error;

/// 钱包全局错误类型定义模块
/// 用于集中管理所有与钱包相关的错误，便于统一处理和扩展

/// 钱包通用错误类型
#[derive(Debug, Error)]
pub enum WalletError {
    /// 加密相关错误，携带详细信息
    #[error("Encryption error: {0}")]
    EncryptionError(String),
    #[allow(dead_code)]
    /// 解密相关错误，携带详细信息
    #[error("Decryption error: {0}")]
    DecryptionError(String),
    /// 盐值无效错误
    #[error("Invalid salt: {0}")]
    InvalidSalt(String),
    /// 其他未知错误
    #[error("Other error: {0}")]
    #[allow(dead_code)]
    Other(String),
}

// The Display implementation has been removed to avoid conflict with derive(Error).
// ...existing code from error.rs...
