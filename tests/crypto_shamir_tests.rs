use defi_hot_wallet::crypto::shamir::{combine_secret, split_secret};
use itertools::Itertools;

#[test]
fn test_shamir_secret_sharing_basic() {
    let threshold = 3;
    let shares_count = 5;
    // 鍒涘缓涓€涓纭殑[u8; 32]鏁扮粍鑰岄潪寮曠敤
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
    // 鍒涘缓涓€涓纭殑[u8; 32]鏁扮粍
    let mut secret = [0u8; 32];
    secret[0] = 42;

    let shares = split_secret(secret, threshold, shares_count).unwrap();

    let result = combine_secret(&shares[0..threshold as usize - 1]);
    assert!(result.is_err());
}

#[test]
fn test_shamir_invalid_threshold() {
    // 鍒涘缓涓€涓纭殑[u8; 32]鏁扮粍
    let secret = [0u8; 32];

    // 闃堝€煎ぇ浜庝唤棰濇暟閲?    let result = split_secret(secret, 5, 3);
    assert!(result.is_err());
}

#[test]
fn test_shamir_zero_threshold() {
    // 鍒涘缓涓€涓纭殑[u8; 32]鏁扮粍
    let secret = [0u8; 32];

    // 闆堕槇鍊?    let result = split_secret(secret, 0, 5);
    assert!(result.is_err());
}

#[test]
fn test_shamir_equal_threshold_and_shares() {
    // 鍒涘缓涓€涓纭殑[u8; 32]鏁扮粍
    let secret = [0u8; 32];

    // 闃堝€肩瓑浜庡叡浜暟
    let result = split_secret(secret, 3, 3);
    assert!(result.is_ok());
}

#[test]
fn test_shamir_reconstruct_exact() {
    // 鍒涘缓涓€涓纭殑[u8; 32]鏁扮粍
    let mut secret = [0u8; 32];
    for i in 0..32 {
        secret[i] = (i * 7) as u8; // 濉厖涓€浜涙暟鎹?    }

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
    // 鍒涘缓涓€涓纭殑[u8; 32]鏁扮粍
    let mut secret = [0u8; 32];
    // 濉厖鏁版嵁
    for i in 0..21 {
        secret[i] = (i * 13 + 7) as u8;
    }

    let shares = split_secret(secret, threshold, shares_count).unwrap();

    let recovered = combine_secret(&shares[0..threshold as usize]).unwrap();
    assert_eq!(recovered, secret);

    // 娴嬭瘯涓嶅悓鐨勫瓙闆嗙粍鍚?    let combination = vec![shares[0].clone(), shares[2].clone(), shares[4].clone()];

    let recovered2 = combine_secret(&combination).unwrap();
    assert_eq!(recovered2, secret);
}

#[test]
fn test_shamir_all_possible_combinations() {
    let threshold = 3;
    let shares_count = 5;
    // 鍒涘缓涓€涓纭殑[u8; 32]鏁扮粍
    let mut secret = [0u8; 32];
    for i in 0..18 {
        secret[i] = (i * 11) as u8;
    }

    let shares = split_secret(secret, threshold, shares_count).unwrap();

    // 娴嬭瘯鍙兘鐨勭粍鍚?- 娉ㄦ剰闇€瑕佸畨瑁卛tertools
    for combination in shares.iter().combinations(threshold as usize) {
        let selected_shares: Vec<(u8, [u8; 32])> =
            combination.iter().map(|&share| share.clone()).collect();
        let recovered = combine_secret(&selected_shares).unwrap();
        assert_eq!(recovered, secret);
    }
}

#[test]
fn test_shamir_tampered_share() {
    let threshold = 3;
    let shares_count = 5;
    // 鍒涘缓涓€涓纭殑[u8; 32]鏁扮粍
    let mut secret = [0u8; 32];
    for i in 0..21 {
        secret[i] = if i == 0 { 0xAA } else { (i * 5) as u8 };
    }

    let mut shares = split_secret(secret, threshold, shares_count).unwrap();

    // 绡℃敼涓€涓唤棰?- 淇敼绗簩涓唤棰濈殑绗竴涓瓧鑺?    shares[1].1[0] ^= 0xFF; // 浣跨敤.1璁块棶鍏冪粍鐨勭浜屼釜鍏冪礌锛岀劧鍚庝慨鏀圭涓€涓瓧鑺?
    let result = combine_secret(&shares[0..threshold as usize]);
    assert!(result.is_ok());
    assert_ne!(result.unwrap(), secret); // 搴旇涓嶅尮閰嶅師濮嬬瀵?}
