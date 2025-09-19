pub mod ethereum;
pub mod solana;
pub mod traits;

pub use ethereum::EthereumClient;
pub use solana::SolanaClient;
pub use traits::BlockchainClient;