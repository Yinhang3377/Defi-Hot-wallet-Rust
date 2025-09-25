pub mod client;
pub mod ethereum;
pub mod traits;
pub mod solana;
pub mod bridge;

pub use traits::BlockchainClient;
pub use bridge::Bridge;
