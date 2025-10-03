use std::fmt;

/// Custom error type for wallet operations.
#[derive(Debug)]
pub enum WalletError {
    /// Configuration-related errors.
    ConfigError(String),
    /// Storage-related errors.
    StorageError(String),
    /// Blockchain interaction errors.
    BlockchainError(String),
    /// Encryption/decryption errors.
    CryptoError(String),
    /// Bridge operation errors.
    BridgeError(String),
    /// Validation errors.
    ValidationError(String),
    /// Network errors.
    NetworkError(String),
    /// Mnemonic generation/parsing errors.
    MnemonicError(String),
    /// Key derivation errors.
    KeyDerivationError(String),
    /// Address derivation errors.
    AddressError(String),
    /// Serialization/deserialization errors.
    SerializationError(String),
    /// Generic errors.
    Other(String),
}

impl fmt::Display for WalletError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            WalletError::ConfigError(msg) => write!(f, "Configuration error: {}", msg),
            WalletError::StorageError(msg) => write!(f, "Storage error: {}", msg),
            WalletError::BlockchainError(msg) => write!(f, "Blockchain error: {}", msg),
            WalletError::CryptoError(msg) => write!(f, "Crypto error: {}", msg),
            WalletError::BridgeError(msg) => write!(f, "Bridge error: {}", msg),
            WalletError::ValidationError(msg) => write!(f, "Validation error: {}", msg),
            WalletError::NetworkError(msg) => write!(f, "Network error: {}", msg),
            WalletError::MnemonicError(msg) => write!(f, "Mnemonic error: {}", msg),
            WalletError::KeyDerivationError(msg) => write!(f, "Key derivation error: {}", msg),
            WalletError::AddressError(msg) => write!(f, "Address error: {}", msg),
            WalletError::SerializationError(msg) => write!(f, "Serialization error: {}", msg),
            WalletError::Other(msg) => write!(f, "Error: {}", msg),
        }
    }
}

impl std::error::Error for WalletError {}

impl From<anyhow::Error> for WalletError {
    fn from(err: anyhow::Error) -> Self {
        WalletError::Other(err.to_string())
    }
}

impl From<std::io::Error> for WalletError {
    fn from(err: std::io::Error) -> Self {
        WalletError::StorageError(err.to_string())
    }
}

impl From<serde_json::Error> for WalletError {
    fn from(err: serde_json::Error) -> Self {
        WalletError::ValidationError(err.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_display_config_error() {
        let err = WalletError::ConfigError("Invalid config".to_string());
        assert_eq!(format!("{}", err), "Configuration error: Invalid config");
    }

    #[test]
    fn test_display_storage_error() {
        let err = WalletError::StorageError("DB failure".to_string());
        assert_eq!(format!("{}", err), "Storage error: DB failure");
    }

    #[test]
    fn test_from_anyhow() {
        let anyhow_err = anyhow::anyhow!("Test error");
        let wallet_err: WalletError = anyhow_err.into();
        match wallet_err {
            WalletError::Other(msg) => assert_eq!(msg, "Test error"),
            _ => panic!("Expected Other variant"),
        }
    }
}
