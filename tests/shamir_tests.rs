//! tests/shamir_tests.rs
//!
//! 测试 `src/crypto/shamir.rs` 的功能。
//! 覆盖：
//! - 秘密的分割与组合
//! - 使用不同份额子集进行组合
//! - 错误处理（份额不足、参数无效）
//! - 边界情况

use defi_hot_wallet::crypto::shamir::{combine_secret, split_secret};
use rand_core::{OsRng, RngCore};

#[test]
fn test_split_and_combine_basic_success() {
    let mut secret = [0u8; 32];  // 修复：改为 32 字节
    OsRng.fill_bytes(&mut secret);

    let threshold = 3;
    let shares_count = 5;

    // 1. 分割秘密
    let shares = split_secret(secret, threshold, shares_count).unwrap();
    assert_eq!(shares.len(), shares_count as usize);

    // 2. 使用前 `threshold` 个份额进行组合
    let combination: Vec<(u8, [u8; 32])> = shares.iter().take(threshold as usize).cloned().collect();  // 修复：改为 32 字节
    let recovered_secret = combine_secret(&combination).unwrap();

    // 3. 验证恢复的秘密与原始秘密相同
    assert_eq!(secret, recovered_secret);
}

#[test]
fn test_split_and_combine_with_different_subset() {
    let mut secret = [0u8; 32];  // 修复：改为 32 字节
    OsRng.fill_bytes(&mut secret);

    let threshold = 3;
    let shares_count = 5;

    let shares = split_secret(secret, threshold, shares_count).unwrap();

    // 使用一个不同的份额子集进行组合
    let combination = vec![shares[1].clone(), shares[3].clone(), shares[4].clone()];
    let recovered_secret = combine_secret(&combination).unwrap();

    assert_eq!(secret, recovered_secret);
}

#[test]
fn test_combine_with_insufficient_shares_produces_wrong_secret() {
    let mut secret = [0u8; 32];  // 修复：改为 32 字节
    OsRng.fill_bytes(&mut secret);

    let threshold = 3;
    let shares_count = 5;

    let shares = split_secret(secret, threshold, shares_count).unwrap();

    // 尝试使用少于 `threshold` 数量的份额进行组合
    let combination: Vec<(u8, [u8; 32])> = shares.iter().take((threshold - 1) as usize).cloned().collect();  // 修复：改为 32 字节
    // Implementation returns an error when shares are insufficient
    let result = combine_secret(&combination);
    assert!(result.is_err());
}

#[test]
fn test_split_with_invalid_parameters() {
    let secret = [0u8; 32];  // 修复：改为 32 字节
    // 阈值大于总份额数，应该返回错误
    let result = split_secret(secret, 4, 3);
    assert!(result.is_err());
}

#[test]
fn test_combine_with_no_shares() {
    let parts: Vec<(u8, [u8; 32])> = vec![];  // 修复：改为 32 字节
    let result = combine_secret(&parts);
    assert!(result.is_err());
}

// 新增测试以提升覆盖率
#[test]
fn test_split_with_threshold_one() {
    let secret = [1u8; 32];  // 修复：改为 32 字节
    let shares = split_secret(secret, 1, 1).unwrap();  // 阈值 = 1，份额数 = 1
    assert_eq!(shares.len(), 1);
    let recovered = combine_secret(&shares).unwrap();
    assert_eq!(recovered, secret);
}

#[test]
fn test_split_with_large_secret() {
    let secret = [0u8; 32];  // 修复：保持 32 字节（函数不支持更大）
    let shares = split_secret(secret, 2, 3).unwrap();
    let combination: Vec<(u8, [u8; 32])> = shares.iter().take(2).cloned().collect();  // 修复：改为 32 字节
    let recovered = combine_secret(&combination).unwrap();
    assert_eq!(recovered, secret);
}

#[test]
fn test_combine_with_duplicate_shares() {
    let secret = [2u8; 32];  // 修复：改为 32 字节
    let shares = split_secret(secret, 3, 5).unwrap();
    let combination = vec![shares[0].clone(), shares[0].clone(), shares[1].clone()]; // 重复份额
    let result = combine_secret(&combination);
    assert!(result.is_err()); // 断言返回错误（重复份额 ID）
}