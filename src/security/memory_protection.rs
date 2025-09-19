use std::time::{ Duration, Instant };
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

#[allow(dead_code)]
impl<T: Zeroize> SensitiveData<T> {
    /// 创建新的敏感数据包装器（当前不执行实际 mlock）
    pub fn secure_new(data: T) -> Self {
        Self { data }
    }
}

impl<T: Zeroize + AsMut<[u8]>> AsMut<[u8]> for SensitiveData<T> {
    fn as_mut(&mut self) -> &mut [u8] {
        self.data.as_mut()
    }
}
impl<T: Zeroize + AsRef<[u8]>> AsRef<[u8]> for SensitiveData<T> {
    fn as_ref(&self) -> &[u8] {
        self.data.as_ref()
    }
}

/// 内存锁定接口（占位）
#[allow(dead_code)]
pub trait MemoryLock {
    fn lock(&mut self) -> Result<(), ()>;
    fn unlock(&mut self) -> Result<(), ()>;
}

impl<T: AsMut<[u8]>> MemoryLock for T {
    fn lock(&mut self) -> Result<(), ()> {
        Ok(())
    }
    fn unlock(&mut self) -> Result<(), ()> {
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
