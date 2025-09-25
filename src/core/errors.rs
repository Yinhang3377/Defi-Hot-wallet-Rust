use thiserror::Error;

#[derive(Debug, Error)]
pub enum CoreError {
    #[error("Wallet error: {0}")]
    WalletError(String),

    #[error("Encryption error: {0}")]
    EncryptionError(String),

    #[error("Authentication error: {0}")]
    AuthError(String),
    
    #[error("Invalid parameter: {0}")]
    InvalidParam(String),
}
