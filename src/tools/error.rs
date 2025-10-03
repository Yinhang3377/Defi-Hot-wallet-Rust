// src/tools/error.rs
//! 閿欒绫诲瀷瀹氫箟
//! 涓烘暣涓」鐩彁渚涚粺涓€鐨勯敊璇鐞?
use thiserror::Error;

/// 椤圭洰缁熶竴鐨凴esult绫诲瀷
pub type Result<T> = std::result::Result<T, WalletError>;

/// 閽卞寘閿欒绫诲瀷
#[derive(Debug, Error)]
pub enum WalletError {
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("Deserialization error: {0}")]
    DeserializationError(String),

    #[error("Encryption error: {0}")]
    EncryptionError(String),

    #[error("Decryption error: {0}")]
    DecryptionError(String),

    #[error("Key derivation error: {0}")]
    KeyDerivationError(String),

    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("Authentication failed: {0}")]
    AuthenticationError(String),

    #[error("Authorization failed: {0}")]
    AuthorizationError(String),

    #[error("Network error: {0}")]
    NetworkError(String),

    #[error("Blockchain error: {0}")]
    BlockchainError(String),

    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("Memory error: {0}")]
    MemoryError(String),

    #[error("Database error: {0}")]
    DatabaseError(String),

    #[error("Storage error: {0}")]
    StorageError(String),

    #[error("Validation error: {0}")]
    ValidationError(String),

    #[error("Unsupported feature: {0}")]
    UnsupportedFeature(String),

    #[error("Rate limit exceeded: {0}")]
    RateLimitError(String),

    #[error("Compliance violation: {0}")]
    ComplianceError(String),

    #[error("Security violation: {0}")]
    SecurityError(String),

    #[error("Timeout error: {0}")]
    TimeoutError(String),

    #[error("Not found: {0}")]
    NotFoundError(String),

    #[error("Already exists: {0}")]
    AlreadyExistsError(String),

    #[error("Insufficient funds: {0}")]
    InsufficientFundsError(String),

    #[error("Invalid transaction: {0}")]
    InvalidTransactionError(String),

    #[error("Wallet locked: {0}")]
    WalletLockedError(String),

    #[error("Mnemonic error: {0}")]
    MnemonicError(String),

    #[error("Address error: {0}")]
    AddressError(String),

    #[error("Signature error: {0}")]
    SignatureError(String),

    #[error("Contract error: {0}")]
    ContractError(String),

    #[error("Bridge error: {0}")]
    BridgeError(String),

    #[error("Exchange error: {0}")]
    ExchangeError(String),

    #[error("Staking error: {0}")]
    StakingError(String),

    #[error("Monitoring error: {0}")]
    MonitoringError(String),

    #[error("Audit error: {0}")]
    AuditError(String),

    #[error("Plugin error: {0}")]
    PluginError(String),

    #[error("I18n error: {0}")]
    I18nError(String),

    #[error("Async error: {0}")]
    AsyncError(String),

    #[error("Generic error: {0}")]
    GenericError(String),
}

impl WalletError {
    /// 鍒涘缓涓€涓柊鐨勯€氱敤閿欒
    pub fn new(message: impl Into<String>) -> Self {
        Self::GenericError(message.into())
    }

    /// 妫€鏌ユ槸鍚︿负涓ラ噸閿欒
    pub fn is_critical(&self) -> bool {
        matches!(
            self,
            WalletError::SecurityError(_)
                | WalletError::MemoryError(_)
                | WalletError::AuthenticationError(_)
                | WalletError::ComplianceError(_)
        )
    }

    /// 妫€鏌ユ槸鍚︿负鍙噸璇曢敊璇?    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            WalletError::NetworkError(_)
                | WalletError::TimeoutError(_)
                | WalletError::RateLimitError(_)
        )
    }

    /// 鑾峰彇閿欒浠ｇ爜
    pub fn error_code(&self) -> &'static str {
        match self {
            WalletError::IoError(_) => "IO_ERROR",
            WalletError::SerializationError(_) => "SERIALIZATION_ERROR",
            WalletError::DeserializationError(_) => "DESERIALIZATION_ERROR",
            WalletError::EncryptionError(_) => "ENCRYPTION_ERROR",
            WalletError::DecryptionError(_) => "DECRYPTION_ERROR",
            WalletError::KeyDerivationError(_) => "KEY_DERIVATION_ERROR",
            WalletError::InvalidInput(_) => "INVALID_INPUT",
            WalletError::AuthenticationError(_) => "AUTHENTICATION_ERROR",
            WalletError::AuthorizationError(_) => "AUTHORIZATION_ERROR",
            WalletError::NetworkError(_) => "NETWORK_ERROR",
            WalletError::BlockchainError(_) => "BLOCKCHAIN_ERROR",
            WalletError::ConfigError(_) => "CONFIG_ERROR",
            WalletError::MemoryError(_) => "MEMORY_ERROR",
            WalletError::DatabaseError(_) => "DATABASE_ERROR",
            WalletError::StorageError(_) => "STORAGE_ERROR",
            WalletError::ValidationError(_) => "VALIDATION_ERROR",
            WalletError::UnsupportedFeature(_) => "UNSUPPORTED_FEATURE",
            WalletError::RateLimitError(_) => "RATE_LIMIT_ERROR",
            WalletError::ComplianceError(_) => "COMPLIANCE_ERROR",
            WalletError::SecurityError(_) => "SECURITY_ERROR",
            WalletError::TimeoutError(_) => "TIMEOUT_ERROR",
            WalletError::NotFoundError(_) => "NOT_FOUND_ERROR",
            WalletError::AlreadyExistsError(_) => "ALREADY_EXISTS_ERROR",
            WalletError::InsufficientFundsError(_) => "INSUFFICIENT_FUNDS_ERROR",
            WalletError::InvalidTransactionError(_) => "INVALID_TRANSACTION_ERROR",
            WalletError::WalletLockedError(_) => "WALLET_LOCKED_ERROR",
            WalletError::MnemonicError(_) => "MNEMONIC_ERROR",
            WalletError::AddressError(_) => "ADDRESS_ERROR",
            WalletError::SignatureError(_) => "SIGNATURE_ERROR",
            WalletError::ContractError(_) => "CONTRACT_ERROR",
            WalletError::BridgeError(_) => "BRIDGE_ERROR",
            WalletError::ExchangeError(_) => "EXCHANGE_ERROR",
            WalletError::StakingError(_) => "STAKING_ERROR",
            WalletError::MonitoringError(_) => "MONITORING_ERROR",
            WalletError::AuditError(_) => "AUDIT_ERROR",
            WalletError::PluginError(_) => "PLUGIN_ERROR",
            WalletError::I18nError(_) => "I18N_ERROR",
            WalletError::AsyncError(_) => "ASYNC_ERROR",
            WalletError::GenericError(_) => "GENERIC_ERROR",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_creation() {
        let error = WalletError::new("Test error");
        assert!(matches!(error, WalletError::GenericError(_)));
        assert_eq!(error.error_code(), "GENERIC_ERROR");
    }

    #[test]
    fn test_error_classification() {
        let security_error = WalletError::SecurityError("Security breach".to_string());
        assert!(security_error.is_critical());
        assert!(!security_error.is_retryable());

        let network_error = WalletError::NetworkError("Connection failed".to_string());
        assert!(!network_error.is_critical());
        assert!(network_error.is_retryable());
    }

    #[test]
    fn test_error_codes() {
        assert_eq!(
            WalletError::IoError(std::io::Error::new(std::io::ErrorKind::NotFound, "test"))
                .error_code(),
            "IO_ERROR"
        );
        assert_eq!(
            WalletError::EncryptionError("test".to_string()).error_code(),
            "ENCRYPTION_ERROR"
        );
        assert_eq!(WalletError::SecurityError("test".to_string()).error_code(), "SECURITY_ERROR");
    }

    #[test]
    fn test_from_io_error() {
        let io_error = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "access denied");
        let wallet_error: WalletError = io_error.into();
        assert!(matches!(wallet_error, WalletError::IoError(_)));
    }
}
