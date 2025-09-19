use zeroize::Zeroize;
#[cfg(all(feature = "memlock", unix))]
use libc::{ mlock, munlock };
#[cfg(all(feature = "memlock", target_os = "windows"))]
use windows::Win32::System::Memory::{ VirtualLock, VirtualUnlock };
#[cfg(feature = "memlock")]
use std::{ any::TypeId, io };

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
    /// 在启用 memlock feature 时尝试锁定内存，否则退化为普通 new。
    pub fn secure_new(data: T) -> Self {
        let mut s = Self::new(data);
        #[cfg(feature = "memlock")]
        if let Err(e) = s.try_lock_memory() {
            log::warn!("[memlock] 内存锁定失败: {} (继续运行)", e);
        }
        s
    }

    #[cfg(feature = "memlock")]
    fn try_lock_memory(&mut self) -> io::Result<()> {
        if self.locked {
            return Ok(());
        }
        if let Some(slice) = (unsafe { self.bytes_slice_mut() }) {
            let ptr = slice.as_ptr();
            let len = slice.len();
            if len == 0 {
                return Ok(());
            }
            unsafe {
                #[cfg(unix)]
                if mlock(ptr as *const _, len) != 0 {
                    return Err(io::Error::last_os_error());
                }
                #[cfg(target_os = "windows")]
                if VirtualLock(ptr as *const _, len) == false.into() {
                    return Err(io::Error::last_os_error());
                }
            }
            self.locked = true;
        }
        Ok(())
    }

    #[cfg(feature = "memlock")]
    fn try_unlock_memory(&mut self) {
        if !self.locked {
            return;
        }
        if let Some(slice) = (unsafe { self.bytes_slice_mut() }) {
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
                if VirtualUnlock(ptr as *const _, len) == false.into() {
                    log::warn!("[memlock] VirtualUnlock 失败: {:?}", io::Error::last_os_error());
                }
            }
            self.locked = false;
        }
    }

    #[cfg(feature = "memlock")]
    unsafe fn bytes_slice_mut(&mut self) -> Option<&mut [u8]> {
        // 仅支持常用敏感类型：[u8;32] 与 Vec<u8>。若需要更多类型可扩展。
        let type_id = TypeId::of::<T>();
        if type_id == TypeId::of::<[u8; 32]>() {
            let ptr = self as *mut SensitiveData<T> as *mut SensitiveData<[u8; 32]>;
            return Some((&mut *ptr).data.as_mut());
        }
        if type_id == TypeId::of::<Vec<u8>>() {
            let ptr = self as *mut SensitiveData<T> as *mut SensitiveData<Vec<u8>>;
            return Some((&mut *ptr).data.as_mut());
        }
        None
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
