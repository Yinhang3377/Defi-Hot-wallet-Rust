pub mod bridge;
pub mod ethereum;
pub mod solana;
pub mod traits;

pub use bridge::{BridgeTransaction, BridgeTransactionStatus};
pub use traits::{BlockchainClient, Bridge};
