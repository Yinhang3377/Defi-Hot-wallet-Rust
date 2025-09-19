use zeroize::Zeroize;
use std::time::{ Instant, Duration };
#[cfg(all(feature = "memlock", unix))]
use libc::{ mlock, munlock };
#[cfg(all(feature = "memlock", target_os = "windows"))]
use windows::Win32::System::Memory::{ VirtualLock, VirtualUnlock };
#[cfg(feature = "memlock")]
use std::io;

/// 敏感数据包装器：在 Drop 时自动清零；开启 feature "memlock" 时尝试锁定内存。
pub struct SensitiveData<T: Zeroize> {
    pub data: T,
    #[cfg(feature = "memlock")]
    locked: bool,
}

impl<T: Zeroize> SensitiveData<T> {
    pub fn new(data: T) -> Self {
        Self {
            data,
            #[cfg(feature = "memlock")]
            locked: false,
        }
    }
}

impl<T: Zeroize> Drop for SensitiveData<T> {
    fn drop(&mut self) {
        #[cfg(feature = "memlock")]
        {
            self.data.zeroize();
            self.try_unlock_memory();
            return;
        }
        #[cfg(not(feature = "memlock"))]
        {
            self.data.zeroize();
        }
    }
}

impl<T: Zeroize> SensitiveData<T> {
    /// 在启用 memlock feature 时尝试锁定内存；失败仅警告。
    #[allow(unused_mut)]
    pub fn secure_new(data: T) -> Self {
        let mut s = Self::new(data);
        #[cfg(feature = "memlock")]
        {
            // 仅当底层类型可当作字节切片时再尝试锁定（通过 trait 限制另一个 impl 区分）
        }
        s
    }
}

#[cfg(feature = "memlock")]
impl<T: Zeroize + AsRef<[u8]> + AsMut<[u8]>> SensitiveData<T> {
    pub fn secure_new(data: T) -> Self {
        let mut s = Self::new(data);
        if let Err(e) = s.try_lock_memory() {
            log::warn!("[memlock] 内存锁定失败: {} (继续运行)", e);
        }
        s
    }

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
            if !VirtualLock(ptr as *const _, len).as_bool() {
                return Err(io::Error::last_os_error());
            }
        }
        self.locked = true;
        Ok(())
    }

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
            if !VirtualUnlock(ptr as *const _, len).as_bool() {
                log::warn!("[memlock] VirtualUnlock 失败: {:?}", io::Error::last_os_error());
            }
        }
        self.locked = false;
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

/// 内存锁定接口：在未启用 feature 时为 no-op。
pub trait MemoryLock {
    fn lock(&mut self) -> Result<(), String>;
    fn unlock(&mut self) -> Result<(), String>;
}

impl<T: Zeroize> MemoryLock for SensitiveData<T> {
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
