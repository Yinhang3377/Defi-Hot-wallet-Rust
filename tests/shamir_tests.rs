//! tests/shamir_tests.rs
//!
//! 娴嬭瘯 `src/crypto/shamir.rs` 鐨勫姛鑳姐€?//! 瑕嗙洊锛?//! - 绉樺瘑鐨勫垎鍓蹭笌缁勫悎
//! - 浣跨敤涓嶅悓浠介瀛愰泦杩涜缁勫悎
//! - 閿欒澶勭悊锛堜唤棰濅笉瓒炽€佸弬鏁版棤鏁堬級
//! - 杈圭晫鎯呭喌

use defi_hot_wallet::crypto::shamir::{combine_secret, split_secret};
use rand_core::{OsRng, RngCore};

#[test]
fn test_split_and_combine_basic_success() {
    let mut secret = [0u8; 32]; // 淇锛氭敼涓?32 瀛楄妭
    OsRng.fill_bytes(&mut secret);

    let threshold = 3;
    let shares_count = 5;

    // 1. 鍒嗗壊绉樺瘑
    let shares = split_secret(secret, threshold, shares_count).unwrap();
    assert_eq!(shares.len(), shares_count as usize);

    // 2. 浣跨敤鍓?`threshold` 涓唤棰濊繘琛岀粍鍚?    let combination: Vec<(u8, [u8; 32])> =
        shares.iter().take(threshold as usize).cloned().collect(); // 淇锛氭敼涓?32 瀛楄妭
    let recovered_secret = combine_secret(&combination).unwrap();

    // 3. 楠岃瘉鎭㈠鐨勭瀵嗕笌鍘熷绉樺瘑鐩稿悓
    assert_eq!(secret, recovered_secret);
}

#[test]
fn test_split_and_combine_with_different_subset() {
    let mut secret = [0u8; 32]; // 淇锛氭敼涓?32 瀛楄妭
    OsRng.fill_bytes(&mut secret);

    let threshold = 3;
    let shares_count = 5;

    let shares = split_secret(secret, threshold, shares_count).unwrap();

    // 浣跨敤涓€涓笉鍚岀殑浠介瀛愰泦杩涜缁勫悎
    let combination = vec![shares[1].clone(), shares[3].clone(), shares[4].clone()];
    let recovered_secret = combine_secret(&combination).unwrap();

    assert_eq!(secret, recovered_secret);
}

#[test]
fn test_combine_with_insufficient_shares_produces_wrong_secret() {
    let mut secret = [0u8; 32]; // 淇锛氭敼涓?32 瀛楄妭
    OsRng.fill_bytes(&mut secret);

    let threshold = 3;
    let shares_count = 5;

    let shares = split_secret(secret, threshold, shares_count).unwrap();

    // 灏濊瘯浣跨敤灏戜簬 `threshold` 鏁伴噺鐨勪唤棰濊繘琛岀粍鍚?    let combination: Vec<(u8, [u8; 32])> =
        shares.iter().take((threshold - 1) as usize).cloned().collect(); // 淇锛氭敼涓?32 瀛楄妭
                                                                         // Implementation returns an error when shares are insufficient
    let result = combine_secret(&combination);
    assert!(result.is_err());
}

#[test]
fn test_split_with_invalid_parameters() {
    let secret = [0u8; 32]; // 淇锛氭敼涓?32 瀛楄妭
                            // 闃堝€煎ぇ浜庢€讳唤棰濇暟锛屽簲璇ヨ繑鍥為敊璇?    let result = split_secret(secret, 4, 3);
    assert!(result.is_err());
}

#[test]
fn test_combine_with_no_shares() {
    let parts: Vec<(u8, [u8; 32])> = vec![]; // 淇锛氭敼涓?32 瀛楄妭
    let result = combine_secret(&parts);
    assert!(result.is_err());
}

// 鏂板娴嬭瘯浠ユ彁鍗囪鐩栫巼
#[test]
fn test_split_with_threshold_one() {
    let secret = [1u8; 32]; // 淇锛氭敼涓?32 瀛楄妭
    let shares = split_secret(secret, 1, 1).unwrap(); // 闃堝€?= 1锛屼唤棰濇暟 = 1
    assert_eq!(shares.len(), 1);
    let recovered = combine_secret(&shares).unwrap();
    assert_eq!(recovered, secret);
}

#[test]
fn test_split_with_large_secret() {
    let secret = [0u8; 32]; // 淇锛氫繚鎸?32 瀛楄妭锛堝嚱鏁颁笉鏀寔鏇村ぇ锛?    let shares = split_secret(secret, 2, 3).unwrap();
    let combination: Vec<(u8, [u8; 32])> = shares.iter().take(2).cloned().collect(); // 淇锛氭敼涓?32 瀛楄妭
    let recovered = combine_secret(&combination).unwrap();
    assert_eq!(recovered, secret);
}

#[test]
fn test_combine_with_duplicate_shares() {
    let secret = [2u8; 32]; // 淇锛氭敼涓?32 瀛楄妭
    let shares = split_secret(secret, 3, 5).unwrap();
    let combination = vec![shares[0].clone(), shares[0].clone(), shares[1].clone()]; // 閲嶅浠介
    let result = combine_secret(&combination);
    assert!(result.is_err()); // 鏂█杩斿洖閿欒锛堥噸澶嶄唤棰?ID锛?}
