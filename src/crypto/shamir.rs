use anyhow::{anyhow, Result};
use rand_core::{OsRng, RngCore};
use std::collections::HashSet;

/// Split a 32-byte secret into `shares` parts with a reconstruction `threshold`.
/// Each returned tuple is (encoded_id, share_bytes).
/// encoded_id: high nibble = threshold, low nibble = share index (1..=15)
pub fn split_secret(secret: [u8; 32], threshold: u8, shares: u8) -> Result<Vec<(u8, [u8; 32])>> {
    if threshold == 0 || threshold > shares {
        return Err(anyhow!("Threshold must be between 1 and shares"));
    }
    if shares == 0 || shares > 15 {
        return Err(anyhow!("Shares must be between 1 and 15"));
    }

    let mut rng = OsRng;
    // coefficients[0] = secret (a0), coefficients[1..] = random coefficients
    let mut coefficients: Vec<[u8; 32]> = Vec::with_capacity(threshold as usize);
    coefficients.push(secret);
    for _ in 1..threshold {
        let mut coef = [0u8; 32];
        rng.fill_bytes(&mut coef);
        coefficients.push(coef);
    }

    let mut result = Vec::with_capacity(shares as usize);
    for id in 1u8..=shares {
        let mut share_value = [0u8; 32];
        for (byte_idx, share_byte) in share_value.iter_mut().enumerate() {
            // Evaluate polynomial at x = id in GF(256)
            let mut y = coefficients[0][byte_idx];
            let mut x_pow = id;
            for coef in coefficients.iter().skip(1) {
                let term = gf256_mul(coef[byte_idx], x_pow);
                y = gf256_add(y, term);
                x_pow = gf256_mul(x_pow, id);
            }
            *share_byte = y;
        }
        let encoded_id = (threshold << 4) | (id & 0x0F);
        result.push((encoded_id, share_value));
    }

    Ok(result)
}

/// Combine shares to recover the 32-byte secret.
/// parts: slice of (encoded_id, share_bytes)
pub fn combine_secret(parts: &[(u8, [u8; 32])]) -> Result<[u8; 32]> {
    if parts.is_empty() {
        return Err(anyhow!("No shares provided"));
    }

    // decode threshold from high nibble
    let first_enc = parts[0].0;
    let threshold = first_enc >> 4;
    if threshold == 0 {
        return Err(anyhow!("Invalid encoded threshold in share ID"));
    }

    // validate and collect x coordinates (low nibble)
    let mut ids = HashSet::new();
    for (enc_id, _) in parts.iter() {
        let t = enc_id >> 4;
        if t != threshold {
            return Err(anyhow!("Mismatched thresholds in shares: {} vs {}", t, threshold));
        }
        let id = enc_id & 0x0F;
        if id == 0 {
            return Err(anyhow!("Share index cannot be zero"));
        }
        if !ids.insert(id) {
            return Err(anyhow!("Duplicate share index: {}", id));
        }
    }

    if (parts.len() as u8) < threshold {
        return Err(anyhow!("Insufficient shares: need {} got {}", threshold, parts.len()));
    }

    // Use exactly `threshold` shares for reconstruction (take first threshold parts)
    let use_parts = &parts[..(threshold as usize)];

    let mut result = [0u8; 32];

    // Reconstruct each byte independently using Lagrange interpolation at x=0
    for byte_idx in 0..32 {
        let mut secret_byte = 0u8;

        for (j, (enc_x_j, share_j)) in use_parts.iter().enumerate() {
            let x_j = enc_x_j & 0x0F;
            let y_j = share_j[byte_idx];

            // compute numerator = product_{m != j} x_m
            // compute denominator = product_{m != j} (x_m - x_j) in GF(256)
            let mut numerator = 1u8;
            let mut denominator = 1u8;

            for (m, (enc_x_m, _)) in use_parts.iter().enumerate() {
                if m == j {
                    continue;
                }
                let x_m = enc_x_m & 0x0F;
                numerator = gf256_mul(numerator, x_m);
                let diff = gf256_sub(x_m, x_j);
                if diff == 0 {
                    return Err(anyhow!(
                        "Failed to calculate Lagrange basis: division by zero in GF(256)"
                    ));
                }
                denominator = gf256_mul(denominator, diff);
            }

            let lagrange_basis = gf256_div(numerator, denominator)
                .map_err(|e| anyhow!("Failed to calculate Lagrange basis: {}", e))?;
            secret_byte ^= gf256_mul(y_j, lagrange_basis);
        }

        result[byte_idx] = secret_byte;
    }

    Ok(result)
}

// GF(256) addition/subtraction = XOR
fn gf256_add(a: u8, b: u8) -> u8 {
    a ^ b
}
fn gf256_sub(a: u8, b: u8) -> u8 {
    a ^ b
}

// GF(256) multiplication (AES polynomial x^8 + x^4 + x^3 + x + 1)
fn gf256_mul(mut a: u8, mut b: u8) -> u8 {
    let mut result: u8 = 0;
    while b != 0 {
        if (b & 1) != 0 {
            result ^= a;
        }
        let carry = (a & 0x80) != 0;
        a <<= 1;
        if carry {
            a ^= 0x1B;
        }
        b >>= 1;
    }
    result
}

// Division using multiplicative inverse
fn gf256_div(a: u8, b: u8) -> Result<u8> {
    if b == 0 {
        return Err(anyhow!("Division by zero in GF(256)"));
    }
    if a == 0 {
        return Ok(0);
    }
    let inv_b = gf256_inverse(b).ok_or_else(|| anyhow!("Division by zero in GF(256)"))?;
    Ok(gf256_mul(a, inv_b))
}

/// Multiplicative inverse in GF(256) using lookup table.
fn gf256_inverse(a: u8) -> Option<u8> {
    if a == 0 {
        return None;
    }
    const INVERSE_TABLE: [u8; 256] = [
        0x00, 0x01, 0x8d, 0xf6, 0xcb, 0x52, 0x7b, 0xd1, 0xe8, 0x4f, 0x29, 0xc0, 0xb0, 0xe1, 0xe5,
        0xc7, 0x74, 0xb4, 0xaa, 0x4b, 0x99, 0x2b, 0x60, 0x5f, 0x58, 0x3f, 0xfd, 0xcc, 0xff, 0x40,
        0xee, 0xb2, 0x3a, 0x6e, 0x5a, 0xf1, 0x55, 0x4d, 0xa8, 0xc9, 0xc1, 0x0a, 0x98, 0x15, 0x30,
        0x44, 0xa2, 0xc2, 0x2c, 0x45, 0x92, 0x6c, 0xf3, 0x39, 0x66, 0x42, 0xf2, 0x35, 0x20, 0x6f,
        0x77, 0xbb, 0x59, 0x19, 0x1d, 0xfe, 0x37, 0x67, 0x2d, 0x31, 0xf5, 0x69, 0xa7, 0x64, 0xab,
        0x13, 0x54, 0x25, 0xe9, 0x09, 0xed, 0x5c, 0x05, 0xca, 0x4c, 0x24, 0x87, 0xbf, 0x18, 0x3e,
        0x22, 0xf0, 0x51, 0xec, 0x61, 0x17, 0x16, 0x5e, 0xaf, 0xd3, 0x49, 0xa6, 0x36, 0x43, 0xf4,
        0x47, 0x91, 0xdf, 0x33, 0x93, 0x21, 0x3b, 0x79, 0xb7, 0x97, 0x85, 0x10, 0xb5, 0xba, 0x3c,
        0xb6, 0x70, 0xd0, 0x06, 0xa1, 0xfa, 0x81, 0x82, 0x83, 0x7e, 0x7f, 0x80, 0x96, 0x73, 0xbe,
        0x56, 0x9b, 0x9e, 0x95, 0xd9, 0xf7, 0x02, 0xb9, 0xa4, 0xde, 0x6a, 0x32, 0x6d, 0xd8, 0x8a,
        0x84, 0x72, 0x2a, 0x14, 0x9f, 0x88, 0xf9, 0xdc, 0x89, 0x9a, 0xfb, 0x7c, 0x2e, 0xc3, 0x8f,
        0xb8, 0x65, 0x48, 0x26, 0xc8, 0x12, 0x4a, 0xce, 0xe7, 0xd2, 0x62, 0x0c, 0xe0, 0x1f, 0xef,
        0x11, 0x75, 0x78, 0x71, 0xa5, 0x8e, 0x76, 0x3d, 0xbd, 0xbc, 0x86, 0x57, 0x0b, 0x28, 0x2f,
        0xa3, 0xda, 0xd4, 0xe4, 0x0f, 0xa9, 0x27, 0x53, 0x04, 0x1b, 0xfc, 0xac, 0xe6, 0x7a, 0x07,
        0xae, 0x63, 0xc5, 0xdb, 0xe2, 0xea, 0x94, 0x8b, 0xc4, 0xd5, 0x9d, 0xf8, 0x90, 0x6b, 0xb1,
        0x0d, 0xd6, 0xeb, 0xc6, 0x0e, 0xcf, 0xad, 0x08, 0x4e, 0xd7, 0xe3, 0x5d, 0x50, 0x1e, 0xb3,
        0x5b, 0x23, 0x38, 0x34, 0x68, 0x46, 0x03, 0x8c, 0xdd, 0x9c, 0x7d, 0xa0, 0xcd, 0x1a, 0x41,
        0x1c,
    ];
    Some(INVERSE_TABLE[a as usize])
}

/// Lightweight helper struct wrapping the functions above.
pub struct ShamirSecretSharing {
    pub threshold: u8,
    pub shares: u8,
}

impl ShamirSecretSharing {
    pub fn new(threshold: u8, shares: u8) -> Self {
        Self { threshold, shares }
    }

    pub fn split_secret(&self, secret: [u8; 32]) -> Result<Vec<(u8, [u8; 32])>> {
        split_secret(secret, self.threshold, self.shares)
    }

    pub fn combine_secret(parts: &[(u8, [u8; 32])]) -> Result<[u8; 32]> {
        combine_secret(parts)
    }
}
