// Re-export security-layer Shamır API so callers/tests can use crate::crypto::shamir::*
pub use crate::security::shamir::{combine_secret, combine_shares, split_secret, ShamirError};
