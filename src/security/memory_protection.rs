use std::time::{Duration, Instant};
use zeroize::Zeroize;

/// 敏感数据包装器，在销毁时自动清零
pub struct SensitiveData<T: Zeroize> {
    pub data: T,
}

impl<T: Zeroize> SensitiveData<T> {
    pub fn new(data: T) -> Self {
        Self { data }
    }
}

impl<T: Zeroize> Drop for SensitiveData<T> {
    fn drop(&mut self) {
        self.data.zeroize();
    }
}

// 可选的内存锁定接口，默认不启用，避免未使用产生的编译告警。
// 启用方式：在 Cargo.toml 增加 `features = ["memlock"]` 并在需要的模块使用。
#[cfg(feature = "memlock")]
#[allow(dead_code)]
pub trait MemoryLock {
    fn lock(&mut self) -> Result<(), std::io::Error>;
    fn unlock(&mut self) -> Result<(), std::io::Error>;
}

#[cfg(feature = "memlock")]
impl<T: AsMut<[u8]>> MemoryLock for T {
    fn lock(&mut self) -> Result<(), std::io::Error> {
        Ok(())
    }
    fn unlock(&mut self) -> Result<(), std::io::Error> {
        Ok(())
    }
}

pub struct MemoryProtector {
    last_clean: Instant,
    interval: Duration,
}

impl MemoryProtector {
    pub fn new() -> Self {
        Self { last_clean: Instant::now(), interval: Duration::from_secs(60) }
    }
    pub fn protect(&mut self, data: &mut [u8]) {
        if self.last_clean.elapsed() > self.interval {
            data.zeroize();
            self.last_clean = Instant::now();
            println!("[内存保护] 已定期清理敏感数据");
        }
    }
}

impl Default for MemoryProtector {
    fn default() -> Self {
        Self::new()
    }
}
