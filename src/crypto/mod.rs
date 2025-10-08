pub mod hsm;
pub mod kdf;
pub mod multisig;
pub mod quantum;
pub mod shamir;

pub use self::hsm::HSMManager;
pub use self::kdf::KeyDerivation;
pub use self::multisig::MultiSignature;
pub use self::quantum::QuantumSafeEncryption;
// Fix: export shamir symbols from the crypto::shamir module (not from security::shamir)
pub use self::shamir::{combine_secret, combine_shares, split_secret};
