pub mod hsm;
pub mod kdf;
pub mod multisig;
pub mod quantum;
pub mod shamir;

pub use self::hsm::HSMManager;
pub use self::kdf::KeyDerivation;
pub use self::multisig::MultiSignature;
pub use self::quantum::QuantumSafeEncryption;
pub use self::shamir::ShamirSecretSharing;
// 从这里导出 split_secret 和 combine_secret 函数
pub use shamir::{combine_secret, split_secret};
