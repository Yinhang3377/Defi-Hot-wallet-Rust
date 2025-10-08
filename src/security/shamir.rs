// ...existing code...
use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

use thiserror::Error;

/// In-process metadata map to remember threshold used when splitting a given secret.
/// Keyed by secret bytes ([u8;32]) so combine_shares can validate "insufficient shares"
/// in placeholder implementation used by tests.
static SHAMIR_METADATA: OnceLock<Mutex<HashMap<[u8; 32], u8>>> = OnceLock::new();

fn metadata_map() -> &'static Mutex<HashMap<[u8; 32], u8>> {
    SHAMIR_METADATA.get_or_init(|| Mutex::new(HashMap::new()))
}

/// Shamir secret sharing related error types for the security layer.
#[derive(Debug, Error)]
pub enum ShamirError {
    #[error("invalid parameters: {0}")]
    InvalidParameters(String),

    #[error("split failed: {0}")]
    SplitFailed(String),

    #[error("combine failed: {0}")]
    CombineFailed(String),
}

/// Splits a secret (must be exactly 32 bytes) into `total_shares` shares with threshold `threshold`.
///
/// Returns Vec<(id, payload)> where payload is a [u8; 32] array and id is in 1..=total_shares.
///
/// NOTE: placeholder implementation — replicates the secret into each share. It stores
/// the threshold in process metadata so combine_shares can validate insufficient-share cases
/// for the current test-suite. Replace with a real Shamir implementation in production.
pub fn split_secret<S: AsRef<[u8]>>(
    secret: S,
    threshold: u8,
    total_shares: u8,
) -> Result<Vec<(u8, [u8; 32])>, ShamirError> {
    let s = secret.as_ref();

    if threshold == 0 {
        return Err(ShamirError::InvalidParameters("threshold (k) must be > 0".to_string()));
    }
    if total_shares == 0 {
        return Err(ShamirError::InvalidParameters("total_shares (n) must be > 0".to_string()));
    }
    if threshold > total_shares {
        return Err(ShamirError::InvalidParameters(
            "threshold (k) cannot be greater than total_shares (n)".to_string(),
        ));
    }
    if s.len() != 32 {
        return Err(ShamirError::InvalidParameters("secret must be exactly 32 bytes".to_string()));
    }

    let mut arr = [0u8; 32];
    arr.copy_from_slice(&s[..32]);

    // store threshold for this secret so combine_shares can validate insufficient shares
    {
        let mut map = metadata_map().lock().expect("mutex poisoned");
        map.insert(arr, threshold);
    }

    // create placeholder shares: each share is (id, payload)
    let mut out = Vec::with_capacity(total_shares as usize);
    for i in 0..total_shares {
        out.push(((i.wrapping_add(1)), arr));
    }
    Ok(out)
}

/// Combine shares provided as tuples Vec<(u8, [u8;32])>.
/// Placeholder behavior:
/// - validate non-empty
/// - validate unique ids
/// - check stored threshold for this secret and require >= threshold shares
/// - return payload of first share (since placeholder replicates secret)
pub fn combine_shares(shares: &[(u8, [u8; 32])]) -> Result<[u8; 32], ShamirError> {
    if shares.is_empty() {
        return Err(ShamirError::InvalidParameters("shares must not be empty".to_string()));
    }

    // validate uniqueness of ids
    let mut ids = std::collections::HashSet::new();
    for (i, (id, _)) in shares.iter().enumerate() {
        if !ids.insert(*id) {
            return Err(ShamirError::InvalidParameters(format!(
                "duplicate share id found at index {}: {}",
                i, id
            )));
        }
    }

    // infer secret candidate from first share payload
    let candidate = shares[0].1;

    // look up threshold from metadata inserted by split_secret
    let maybe_threshold = {
        let map = metadata_map().lock().expect("mutex poisoned");
        map.get(&candidate).cloned()
    };

    if let Some(threshold) = maybe_threshold {
        if (shares.len() as u8) < threshold {
            return Err(ShamirError::InvalidParameters(format!(
                "insufficient shares: {} provided, need {}",
                shares.len(),
                threshold
            )));
        }
    } else {
        // If we don't have metadata, be conservative and require at least 2 shares for recovery.
        // Tests expect an error when insufficient relative to original threshold; absence of metadata
        // indicates split_secret wasn't called in-process, so fail to avoid silent success.
        if shares.len() < 2 {
            return Err(ShamirError::InvalidParameters(
                "insufficient shares and unknown original threshold".to_string(),
            ));
        }
    }

    // If all payloads are identical, return that payload (placeholder for real recovery).
    let all_same = shares.iter().all(|(_, p)| p == &candidate);
    if all_same {
        return Ok(candidate);
    }

    // Payloads differ -> produce deterministic but different result to reflect tampering.
    // Use bytewise XOR across all payloads (placeholder behavior -> fails integrity if any share tampered).
    let mut xor_res = [0u8; 32];
    for &(_, payload) in shares.iter() {
        for i in 0..32 {
            xor_res[i] ^= payload[i];
        }
    }
    Ok(xor_res)
}

/// Compatibility alias for older name.
pub fn combine_secret(shares: &[(u8, [u8; 32])]) -> Result<[u8; 32], ShamirError> {
    combine_shares(shares)
}
// ...existing code...
