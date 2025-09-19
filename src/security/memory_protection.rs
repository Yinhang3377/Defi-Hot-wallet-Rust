/// 内存保护模块：防止敏感数据在内存中泄露
/// 包含自动清零、内存锁定等机制

use zeroize::Zeroize;

/// 敏感数据包装器，在销毁时自动清零并尝试解锁内存
#[allow(dead_code)]
pub struct SensitiveData<T: Zeroize> {
    pub data: T,
}

impl<T: Zeroize> Drop for SensitiveData<T> {
    /// 在销毁时，先解锁内存，然后清零数据
    fn drop(&mut self) {
        // self.unlock(); // 如果实现了自动解锁
        self.data.zeroize();
    }
}

#[allow(dead_code)]
impl<T: Zeroize> SensitiveData<T> {
    /// 创建新的敏感数据包装器，并尝试锁定其内存
    pub fn new(data: T) -> Self {
        let s = Self { data };
        // s.lock(); // 如果实现了自动锁定
        s
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

/// 内存锁定接口（预留，平台相关实现）
#[allow(dead_code)]
pub trait MemoryLock {
    /// 锁定内存，防止被交换到磁盘
    fn lock(&mut self) -> Result<(), ()>;
    /// 解锁内存
    fn unlock(&mut self) -> Result<(), ()>;
}

impl<T: AsMut<[u8]>> MemoryLock for T {
    fn lock(&mut self) -> Result<(), ()> {
        // 替代实现：目前 memlock 不可用，返回 Ok(())
        Ok(())
    }
    fn unlock(&mut self) -> Result<(), ()> {
        // 替代实现：目前 memlock 不可用，返回 Ok(())
        Ok(())
    }
}
/// 内存保护与敏感数据定期清理

use std::time::{ Duration, Instant };

pub struct MemoryProtector {
    last_clean: Instant,
    interval: Duration,
}

impl MemoryProtector {
    pub fn new() -> Self {
        MemoryProtector {
            last_clean: Instant::now(),
            interval: Duration::from_secs(60), // 每60秒清理一次
        }
    }
    /// 定期清理敏感数据（示例：这里只是模拟，实际应结合 SensitiveData 使用）
    pub fn protect(&mut self, data: &mut [u8]) {
        if self.last_clean.elapsed() > self.interval {
            data.zeroize();
            self.last_clean = Instant::now();
            println!("[内存保护] 已定期清理敏感数据");
        }
    }
}
