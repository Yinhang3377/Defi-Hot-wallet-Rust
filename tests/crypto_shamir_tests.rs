use defi_hot_wallet::crypto::shamir::{combine_secret, split_secret};
use itertools::Itertools;

#[test]
fn test_shamir_secret_sharing_basic() {
    let threshold = 3;
    let shares_count = 5;
    // 创建一个正确的[u8; 32]数组而非引用
    let mut secret = [0u8; 32];
    secret[0] = 42;
    secret[1] = 101;
    secret[2] = 53;

    let shares = split_secret(secret, threshold, shares_count).unwrap();
    assert_eq!(shares.len(), shares_count as usize);

    let recovered = combine_secret(&shares[0..threshold as usize]).unwrap();
    assert_eq!(recovered, secret);
}

#[test]
fn test_shamir_insufficient_shares() {
    let threshold = 3;
    let shares_count = 5;
    // 创建一个正确的[u8; 32]数组
    let mut secret = [0u8; 32];
    secret[0] = 42;

    let shares = split_secret(secret, threshold, shares_count).unwrap();
    
    let result = combine_secret(&shares[0..threshold as usize - 1]);
    assert!(result.is_err());
}

#[test]
fn test_shamir_invalid_threshold() {
    // 创建一个正确的[u8; 32]数组
    let secret = [0u8; 32];

    // 阈值大于份额数量
    let result = split_secret(secret, 5, 3);
    assert!(result.is_err());
}

#[test]
fn test_shamir_zero_threshold() {
    // 创建一个正确的[u8; 32]数组
    let secret = [0u8; 32];
    
    // 零阈值
    let result = split_secret(secret, 0, 5);
    assert!(result.is_err());
}

#[test]
fn test_shamir_equal_threshold_and_shares() {
    // 创建一个正确的[u8; 32]数组
    let secret = [0u8; 32];

    // 阈值等于共享数
    let result = split_secret(secret, 3, 3);
    assert!(result.is_ok());
}

#[test]
fn test_shamir_reconstruct_exact() {
    // 创建一个正确的[u8; 32]数组
    let mut secret = [0u8; 32];
    for i in 0..32 {
        secret[i] = (i * 7) as u8; // 填充一些数据
    }

    let result = split_secret(secret, 2, 3);
    assert!(result.is_ok());
    
    let shares = result.unwrap();
    let recovered = combine_secret(&shares[0..2]).unwrap();
    assert_eq!(recovered, secret);
}

#[test]
fn test_shamir_different_share_subsets() {
    let threshold = 3;
    let shares_count = 5;
    // 创建一个正确的[u8; 32]数组
    let mut secret = [0u8; 32];
    // 填充数据
    for i in 0..21 {
        secret[i] = (i * 13 + 7) as u8;
    }

    let shares = split_secret(secret, threshold, shares_count).unwrap();

    let recovered = combine_secret(&shares[0..threshold as usize]).unwrap();
    assert_eq!(recovered, secret);

    // 测试不同的子集组合
    let combination = vec![
        shares[0].clone(),
        shares[2].clone(),
        shares[4].clone()
    ];
    
    let recovered2 = combine_secret(&combination).unwrap();
    assert_eq!(recovered2, secret);
}

#[test]
fn test_shamir_all_possible_combinations() {
    let threshold = 3;
    let shares_count = 5;
    // 创建一个正确的[u8; 32]数组
    let mut secret = [0u8; 32];
    for i in 0..18 {
        secret[i] = (i * 11) as u8;
    }

    let shares = split_secret(secret, threshold, shares_count).unwrap();
    
    // 测试可能的组合 - 注意需要安装itertools
    for combination in shares.iter().combinations(threshold as usize) {
        let selected_shares: Vec<(u8, [u8; 32])> = combination.iter().map(|&share| share.clone()).collect();
        let recovered = combine_secret(&selected_shares).unwrap();
        assert_eq!(recovered, secret);
    }
}

#[test]
fn test_shamir_tampered_share() {
    let threshold = 3;
    let shares_count = 5;
    // 创建一个正确的[u8; 32]数组
    let mut secret = [0u8; 32];
    for i in 0..21 {
        secret[i] = if i == 0 { 0xAA } else { (i * 5) as u8 };
    }

    let mut shares = split_secret(secret, threshold, shares_count).unwrap();
    
    // 篡改一个份额 - 修改第二个份额的第一个字节
    shares[1].1[0] ^= 0xFF; // 使用.1访问元组的第二个元素，然后修改第一个字节

    let result = combine_secret(&shares[0..threshold as usize]);
    assert!(result.is_ok());
    assert_ne!(result.unwrap(), secret); // 应该不匹配原始秘密
}