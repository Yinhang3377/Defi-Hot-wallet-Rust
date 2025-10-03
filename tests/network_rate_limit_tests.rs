//! tests/network_rate_limit_tests.rs
//!
//! 閽堝 `src/network/rate_limit.rs` 鐨勫崟鍏冩祴璇曘€?use defi_hot_wallet::network::rate_limit::RateLimiter;
use std::time::Duration;

#[test]
fn test_rate_limiter_new_and_initial_allow() {
    // 姝ｅ父璺緞锛氬垱寤轰竴涓柊鐨勯€熺巼闄愬埗鍣ㄥ苟鍏佽绗竴涓姹?    let limiter = RateLimiter::new(10, Duration::from_secs(1));
    assert!(limiter.allow(), "First request should be allowed");
}

#[test]
fn test_rate_limiter_exceeds_limit() {
    // 姝ｅ父璺緞锛氭祴璇曡秴鍑洪€熺巼闄愬埗
    let limiter = RateLimiter::new(1, Duration::from_millis(200));

    // 绗竴涓姹傚簲璇ヨ鍏佽
    assert!(limiter.allow(), "The first request should be allowed");

    // 绱ф帴鐫€鐨勭浜屼釜璇锋眰搴旇鍥犱负瓒呭嚭閫熺巼鑰岃鎷掔粷
    assert!(!limiter.allow(), "The second request should be denied as it exceeds the rate limit");
}

#[test]
fn test_rate_limiter_clone_shares_state() {
    // 姝ｅ父璺緞锛氭祴璇曞厠闅嗙殑瀹炰緥鍏变韩鐩稿悓鐨勯€熺巼闄愬埗鐘舵€?    let limiter1 = RateLimiter::new(1, Duration::from_millis(200));
    let limiter2 = limiter1.clone();

    // 浣跨敤绗竴涓疄渚嬫秷鑰楁帀璁稿彲
    assert!(limiter1.allow(), "First request on limiter1 should be allowed");

    // 绗簩涓疄渚嬶紙鍏嬮殕锛夌殑璇锋眰搴旇琚嫆缁濓紝鍥犱负瀹冧滑鍏变韩鐘舵€?    assert!(!limiter2.allow(), "Request on cloned limiter2 should be denied as the quota is used");
}
