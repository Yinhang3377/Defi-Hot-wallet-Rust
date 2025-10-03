use thiserror::Error;

pub type Result<T> = std::result::Result<T, WalletError>;

#[derive(Error, Debug)]
pub enum WalletError {
    #[error("Crypto error: {0}")]
    Crypto(#[from] anyhow::Error),

    #[error("Storage error: {0}")]
    Storage(String),

    #[error("Network error: {0}")]
    Network(String),

    #[error("Blockchain error: {0}")]
    Blockchain(String),

    #[error("Invalid mnemonic: {0}")]
    InvalidMnemonic(String),

    #[error("Invalid address: {0}")]
    InvalidAddress(String),

    #[error("Insufficient balance")]
    InsufficientBalance,

    #[error("Wallet not found: {0}")]
    WalletNotFound(String),

    #[error("HSM error: {0}")]
    HSM(String),

    #[error("Multi-signature error: {0}")]
    MultiSig(String),

    #[error("Quantum crypto error: {0}")]
    QuantumCrypto(String),

    #[error("Configuration error: {0}")]
    Config(#[from] config::ConfigError),

    #[error("Serialization error: {0}")]
    Serialization(#[from] bincode::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}
