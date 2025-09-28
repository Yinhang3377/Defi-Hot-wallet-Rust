//! tests/network_rate_limit_tests.rs
//!
//! 针对 `src/network/rate_limit.rs` 的单元测试。
use defi_hot_wallet::network::rate_limit::RateLimiter;
use std::time::Duration;

#[test]
fn test_rate_limiter_new_and_initial_allow() {
    // 正常路径：创建一个新的速率限制器并允许第一个请求
    let limiter = RateLimiter::new(10, Duration::from_secs(1));
    assert!(limiter.allow(), "First request should be allowed");
}

#[test]
fn test_rate_limiter_exceeds_limit() {
    // 正常路径：测试超出速率限制
    let limiter = RateLimiter::new(1, Duration::from_millis(200));

    // 第一个请求应该被允许
    assert!(limiter.allow(), "The first request should be allowed");

    // 紧接着的第二个请求应该因为超出速率而被拒绝
    assert!(!limiter.allow(), "The second request should be denied as it exceeds the rate limit");
}

#[test]
fn test_rate_limiter_clone_shares_state() {
    // 正常路径：测试克隆的实例共享相同的速率限制状态
    let limiter1 = RateLimiter::new(1, Duration::from_millis(200));
    let limiter2 = limiter1.clone();

    // 使用第一个实例消耗掉许可
    assert!(limiter1.allow(), "First request on limiter1 should be allowed");

    // 第二个实例（克隆）的请求应该被拒绝，因为它们共享状态
    assert!(
        !limiter2.allow(),
        "Request on cloned limiter2 should be denied as the quota is used"
    );
}