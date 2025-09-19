use zeroize::Zeroize;
use std::time::{ Instant, Duration };
#[cfg(all(feature = "memlock", unix))]
use libc::{ mlock, munlock };
#[cfg(all(feature = "memlock", target_os = "windows"))]
use windows::Win32::System::Memory::{ VirtualLock, VirtualUnlock };
#[cfg(feature = "memlock")]
use std::io;

/// 敏感数据包装器：在 Drop 时自动清零；开启 feature "memlock" 时尝试锁定内存。
/// 为保证实现简单并避免重复方法冲突，这里约束 T 必须能映射到字节切片。
pub struct SensitiveData<T: Zeroize + AsRef<[u8]> + AsMut<[u8]>> {
    pub data: T,
    #[cfg(feature = "memlock")]
    locked: bool,
}

impl<T: Zeroize + AsRef<[u8]> + AsMut<[u8]>> SensitiveData<T> {
    pub fn new(data: T) -> Self {
        Self { data, #[cfg(feature = "memlock")] locked: false }
    }

    /// 创建并在启用 feature 时尝试加锁。
    #[cfg_attr(not(feature = "memlock"), allow(unused_mut))]
    pub fn secure_new(data: T) -> Self {
        let mut s = Self::new(data);
        #[cfg(feature = "memlock")]
        {
            if let Err(e) = s.try_lock_memory() {
                log::warn!("[memlock] 内存锁定失败: {} (继续运行)", e);
            }
        }
        s
    }

    #[cfg(feature = "memlock")]
    fn try_lock_memory(&mut self) -> io::Result<()> {
        if self.locked {
            return Ok(());
        }
        let slice = self.data.as_mut();
        if slice.is_empty() {
            return Ok(());
        }
        let ptr = slice.as_ptr();
        let len = slice.len();
        unsafe {
            #[cfg(unix)]
            if mlock(ptr as *const _, len) != 0 {
                return Err(io::Error::last_os_error());
            }
            #[cfg(target_os = "windows")]
            if let Err(e) = VirtualLock(ptr as *const _, len) {
                return Err(io::Error::other(format!("VirtualLock 失败: {e}")));
            }
        }
        self.locked = true;
        Ok(())
    }

    #[cfg(feature = "memlock")]
    fn try_unlock_memory(&mut self) {
        if !self.locked {
            return;
        }
        let slice = self.data.as_ref();
        let ptr = slice.as_ptr();
        let len = slice.len();
        if len == 0 {
            return;
        }
        unsafe {
            #[cfg(unix)]
            if munlock(ptr as *const _, len) != 0 {
                log::warn!("[memlock] munlock 失败: {:?}", io::Error::last_os_error());
            }
            #[cfg(target_os = "windows")]
            if let Err(e) = VirtualUnlock(ptr as *const _, len) {
                log::warn!("[memlock] VirtualUnlock 失败: {e}");
            }
        }
        self.locked = false;
    }
}

impl<T: Zeroize + AsRef<[u8]> + AsMut<[u8]>> Drop for SensitiveData<T> {
    fn drop(&mut self) {
        self.data.zeroize();
        #[cfg(feature = "memlock")]
        self.try_unlock_memory();
    }
}

impl<T: Zeroize + AsRef<[u8]> + AsMut<[u8]>> AsRef<[u8]> for SensitiveData<T> {
    fn as_ref(&self) -> &[u8] {
        self.data.as_ref()
    }
}
impl<T: Zeroize + AsRef<[u8]> + AsMut<[u8]>> AsMut<[u8]> for SensitiveData<T> {
    fn as_mut(&mut self) -> &mut [u8] {
        self.data.as_mut()
    }
}

/// 内存锁定接口（显式调用，可选）。
pub trait MemoryLock {
    fn lock(&mut self) -> Result<(), String>;
    fn unlock(&mut self) -> Result<(), String>;
}

impl<T: Zeroize + AsRef<[u8]> + AsMut<[u8]>> MemoryLock for SensitiveData<T> {
    fn lock(&mut self) -> Result<(), String> {
        #[cfg(feature = "memlock")]
        self.try_lock_memory().map_err(|e| e.to_string())?;
        Ok(())
    }
    fn unlock(&mut self) -> Result<(), String> {
        #[cfg(feature = "memlock")]
        self.try_unlock_memory();
        Ok(())
    }
}

#[allow(dead_code)]
pub struct MemoryProtector {
    last_clean: Instant,
    interval: Duration,
}

#[allow(dead_code)]
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
