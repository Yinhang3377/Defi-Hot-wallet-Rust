// src/security/memory_protection.rs
//! 内存保护模块
//! 用于安全处理敏感数据，防止内存泄露

use crate::tools::error::WalletError;
use std::alloc::{alloc, dealloc, Layout};
use std::ptr;

/// 安全内存缓冲区
/// 在Drop时自动清除内容
pub struct SecureBuffer {
    ptr: *mut u8,
    len: usize,
    layout: Layout,
}

impl SecureBuffer {
    /// 创建新的安全缓冲区
    pub fn new(size: usize) -> Result<Self, WalletError> {
        if size == 0 {
            return Err(WalletError::InvalidInput("Buffer size cannot be zero".to_string()));
        }

        let layout = Layout::array::<u8>(size)
            .map_err(|_| WalletError::InvalidInput("Invalid buffer layout".to_string()))?;

        let ptr = unsafe { alloc(layout) };
        if ptr.is_null() {
            return Err(WalletError::MemoryError("Failed to allocate secure memory".to_string()));
        }

        Ok(Self { ptr, len: size, layout })
    }

    /// 获取缓冲区长度
    pub fn len(&self) -> usize {
        self.len
    }

    /// 检查缓冲区是否为空
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// 安全地写入数据
    pub fn write(&mut self, data: &[u8]) -> Result<(), WalletError> {
        if data.len() > self.len {
            return Err(WalletError::InvalidInput("Data too large for buffer".to_string()));
        }

        unsafe {
            ptr::copy_nonoverlapping(data.as_ptr(), self.ptr, data.len());
        }

        Ok(())
    }

    /// 安全地读取数据
    pub fn read(&self, dest: &mut [u8]) -> Result<usize, WalletError> {
        let read_len = dest.len().min(self.len);
        unsafe {
            ptr::copy_nonoverlapping(self.ptr, dest.as_mut_ptr(), read_len);
        }
        Ok(read_len)
    }

    /// 获取只读访问
    pub fn as_slice(&self) -> &[u8] {
        unsafe { std::slice::from_raw_parts(self.ptr, self.len) }
    }

    /// 获取可写访问（不安全）
    ///
    /// # Safety
    ///
    /// 调用者必须确保:
    /// - 不会有其他引用同时访问相同内存
    /// - 不会越界写入数据
    pub unsafe fn as_mut_slice(&mut self) -> &mut [u8] {
        std::slice::from_raw_parts_mut(self.ptr, self.len)
    }
}

impl Drop for SecureBuffer {
    fn drop(&mut self) {
        // 在释放前清除内存内容
        unsafe {
            clear_sensitive_data(self.ptr, self.len);
            dealloc(self.ptr, self.layout);
        }
    }
}

impl Clone for SecureBuffer {
    fn clone(&self) -> Self {
        let new_buf = Self::new(self.len).expect("Failed to clone SecureBuffer");
        unsafe {
            ptr::copy_nonoverlapping(self.ptr, new_buf.ptr, self.len);
        }
        new_buf
    }
}

/// 清除敏感数据
/// 使用多种方法确保数据被覆盖
///
/// # Safety
///
/// 此函数需要一个有效的指针和长度。调用者必须确保:
/// - `ptr` 指向有效的内存区域，并且可写入
/// - `len` 不超过分配给 `ptr` 的内存大小
/// - 操作期间 `ptr` 不会被其他代码访问
pub unsafe fn clear_sensitive_data(ptr: *mut u8, len: usize) {
    if ptr.is_null() || len == 0 {
        return;
    }

    // 方法1: 用零覆盖
    ptr::write_bytes(ptr, 0, len);

    // 方法2: 用伪随机（示例）数据覆盖
    for i in 0..len {
        *ptr.add(i) = (i % 256) as u8;
    }

    // 方法3: 再次用零覆盖
    ptr::write_bytes(ptr, 0, len);

    // 方法4: 用0xFF覆盖
    ptr::write_bytes(ptr, 0xFF, len);

    // 最终用零覆盖
    ptr::write_bytes(ptr, 0, len);
}

/// 清除敏感数据缓冲区（安全包装）
pub fn clear_sensitive(buf: &mut [u8]) {
    unsafe {
        clear_sensitive_data(buf.as_mut_ptr(), buf.len());
    }
}

/// 安全字符串类型
/// 在Drop时自动清除内容
pub struct SecureString {
    buffer: SecureBuffer,
}

impl SecureString {
    /// 创建新的安全字符串
    pub fn new(s: &str) -> Result<Self, WalletError> {
        let mut buffer = SecureBuffer::new(s.len())?;
        buffer.write(s.as_bytes())?;
        Ok(Self { buffer })
    }

    /// 获取字符串长度
    pub fn len(&self) -> usize {
        self.buffer.len()
    }

    /// 检查字符串是否为空
    pub fn is_empty(&self) -> bool {
        self.buffer.len() == 0
    }

    /// 安全地获取字符串内容
    pub fn reveal(&self) -> Result<String, WalletError> {
        let mut data = vec![0u8; self.len()];
        self.buffer.read(&mut data)?;
        String::from_utf8(data)
            .map_err(|e| WalletError::InvalidInput(format!("Invalid UTF-8: {}", e)))
    }
}

impl Drop for SecureString {
    fn drop(&mut self) {
        // SecureBuffer 的 Drop 会处理清除
    }
}

/// 内存锁定（防止页面交换）
/// 注意：这需要特定的系统权限
///
/// # Safety
///
/// 调用者必须确保:
/// - `ptr` 指向有效的、已分配的内存
/// - `len` 不超过分配的内存大小
/// - 锁定的内存不会过多消耗系统资源
pub unsafe fn lock_memory(ptr: *mut u8, len: usize) -> Result<(), WalletError> {
    #[cfg(unix)]
    {
        use libc::mlock;
        let result = mlock(ptr as *const std::ffi::c_void, len);
        if result != 0 {
            return Err(WalletError::MemoryError("Failed to lock memory".to_string()));
        }
    }

    #[cfg(windows)]
    {
        use winapi::um::memoryapi::VirtualLock;
        let result = VirtualLock(ptr as winapi::shared::minwindef::LPVOID, len);
        if result == 0 {
            return Err(WalletError::MemoryError("Failed to lock memory".to_string()));
        }
    }

    #[cfg(not(any(unix, windows)))]
    {
        return Err(WalletError::UnsupportedFeature(
            "Memory locking not supported on this platform".to_string(),
        ));
    }

    Ok(())
}

/// 内存解锁
///
/// # Safety
///
/// 调用者必须确保:
/// - `ptr` 指向之前通过 `lock_memory` 锁定的内存
/// - `len` 与锁定时使用的相同
pub unsafe fn unlock_memory(ptr: *mut u8, len: usize) -> Result<(), WalletError> {
    #[cfg(unix)]
    {
        use libc::munlock;
        let result = munlock(ptr as *const std::ffi::c_void, len);
        if result != 0 {
            return Err(WalletError::MemoryError("Failed to unlock memory".to_string()));
        }
    }

    #[cfg(windows)]
    {
        use winapi::um::memoryapi::VirtualUnlock;
        let result = VirtualUnlock(ptr as winapi::shared::minwindef::LPVOID, len);
        if result == 0 {
            return Err(WalletError::MemoryError("Failed to unlock memory".to_string()));
        }
    }

    #[cfg(not(any(unix, windows)))]
    {
        return Err(WalletError::UnsupportedFeature(
            "Memory unlocking not supported on this platform".to_string(),
        ));
    }

    Ok(())
}

/// 安全内存分配器
pub struct SecureAllocator {
    locked_pages: Vec<(usize, usize)>, // (ptr, size)
}

impl SecureAllocator {
    pub fn new() -> Self {
        Self { locked_pages: Vec::new() }
    }

    /// 分配并锁定内存
    pub fn alloc_locked(&mut self, size: usize) -> Result<SecureBuffer, WalletError> {
        let buffer = SecureBuffer::new(size)?;
        unsafe {
            lock_memory(buffer.ptr, buffer.len)?;
        }
        self.locked_pages.push((buffer.ptr as usize, buffer.len));
        Ok(buffer)
    }

    /// 解锁所有分配的内存
    pub fn unlock_all(&mut self) -> Result<(), WalletError> {
        for (ptr, size) in &self.locked_pages {
            unsafe {
                unlock_memory(*ptr as *mut u8, *size)?;
            }
        }
        self.locked_pages.clear();
        Ok(())
    }
}

impl Default for SecureAllocator {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for SecureAllocator {
    fn drop(&mut self) {
        let _ = self.unlock_all(); // 忽略错误，因为我们正在清理
    }
}

/// 临时敏感数据处理
/// 确保在作用域结束时清除数据
pub struct TempSensitive<T, F>
where
    F: FnMut(&mut T),
{
    data: T,
    clear_fn: F,
}

impl<T, F> TempSensitive<T, F>
where
    F: FnMut(&mut T),
{
    pub fn new(data: T, clear_fn: F) -> Self {
        Self { data, clear_fn }
    }

    pub fn get(&self) -> &T {
        &self.data
    }

    pub fn get_mut(&mut self) -> &mut T {
        &mut self.data
    }
}

impl<T, F> Drop for TempSensitive<T, F>
where
    F: FnMut(&mut T),
{
    fn drop(&mut self) {
        (self.clear_fn)(&mut self.data);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tools::error::WalletError;

    #[test]
    fn test_secure_buffer() {
        let mut buffer = SecureBuffer::new(32).unwrap();
        assert_eq!(buffer.len(), 32);
        assert!(!buffer.is_empty());

        let data = b"Hello, Secure World!";
        buffer.write(data).unwrap();

        let mut read_data = vec![0u8; data.len()];
        buffer.read(&mut read_data).unwrap();
        assert_eq!(&read_data, data);
    }

    #[test]
    fn test_clear_sensitive() {
        let mut data = [1, 2, 3, 4, 5];
        clear_sensitive(&mut data);
        assert_eq!(data, [0, 0, 0, 0, 0]);
    }

    #[test]
    fn test_secure_string() {
        let secret = "my_secret_password";
        let secure_str = SecureString::new(secret).unwrap();
        assert_eq!(secure_str.len(), secret.len());
        assert!(!secure_str.is_empty());

        let revealed = secure_str.reveal().unwrap();
        assert_eq!(revealed, secret);
    }

    #[test]
    fn test_temp_sensitive() {
        let mut cleared = false;
        {
            let mut temp = TempSensitive::new(42, |x| {
                *x = 0;
                cleared = true;
            });
            assert_eq!(*temp.get(), 42);
            *temp.get_mut() = 100;
            assert_eq!(*temp.get(), 100);
        }
        assert!(cleared);
    }

    #[test]
    fn test_secure_allocator() {
        let mut allocator = SecureAllocator::new();

        // 在某些环境中可能需要权限才能锁定内存
        let result = allocator.alloc_locked(64);
        if let Ok(buffer) = result {
            assert_eq!(buffer.len(), 64);
            // 解锁所有内存
            let _ = allocator.unlock_all();
        }
    }

    // 扩展用例

    #[test]
    fn test_secure_buffer_new_zero_fails() {
        let res = SecureBuffer::new(0);
        assert!(matches!(res, Err(WalletError::InvalidInput(_))));
    }

    #[test]
    fn test_secure_buffer_write_too_large_fails() {
        let mut buffer = SecureBuffer::new(8).unwrap();
        let data = vec![1u8; 16];
        let res = buffer.write(&data);
        assert!(matches!(res, Err(WalletError::InvalidInput(_))));
    }

    #[test]
    fn test_secure_buffer_partial_read_smaller_dest() {
        let mut buffer = SecureBuffer::new(16).unwrap();
        let pattern: Vec<u8> = (0u8..16u8).collect();
        buffer.write(&pattern).unwrap();

        let mut dest = vec![0u8; 8];
        let read_len = buffer.read(&mut dest).unwrap();
        assert_eq!(read_len, 8);
        assert_eq!(dest, pattern[..8]);
    }

    #[test]
    fn test_secure_buffer_read_into_larger_dest() {
        let mut buffer = SecureBuffer::new(8).unwrap();
        let pattern: Vec<u8> = (1u8..=8u8).collect();
        buffer.write(&pattern).unwrap();

        let mut dest = vec![0u8; 16];
        let read_len = buffer.read(&mut dest).unwrap();
        assert_eq!(read_len, 8);
        assert_eq!(&dest[..8], &pattern[..]);
        assert!(dest[8..].iter().all(|&b| b == 0));
    }

    #[test]
    fn test_clear_sensitive_data_overwrites_to_zero() {
        let mut data = vec![0xAA; 32];
        unsafe {
            clear_sensitive_data(data.as_mut_ptr(), data.len());
        }
        assert!(data.iter().all(|&b| b == 0));
    }

    #[test]
    fn test_clear_sensitive_data_zero_len_noop() {
        let mut data: Vec<u8> = (0u8..16u8).collect();
        let original = data.clone();
        unsafe {
            clear_sensitive_data(data.as_mut_ptr(), 0);
        }
        assert_eq!(data, original);
    }

    #[test]
    fn test_secure_string_empty_fails() {
        let res = SecureString::new("");
        assert!(matches!(res, Err(WalletError::InvalidInput(_))));
    }

    #[test]
    fn test_as_mut_slice_mutation() {
        let mut buffer = SecureBuffer::new(10).unwrap();
        let initial = vec![1u8; 10];
        buffer.write(&initial).unwrap();

        unsafe {
            let s = buffer.as_mut_slice();
            for (i, b) in s.iter_mut().enumerate() {
                *b = (i as u8) + 10;
            }
        }

        let expected: Vec<u8> = (0..10u8).map(|i| i + 10).collect();
        assert_eq!(buffer.as_slice(), &expected[..]);
    }

    #[test]
    fn test_lock_and_unlock_memory_smoke() {
        let mut data = vec![0u8; 64];
        let ptr = data.as_mut_ptr();
        let len = data.len();

        let lock_res = unsafe { lock_memory(ptr, len) };
        match lock_res {
            Ok(()) => {
                let unlock_res = unsafe { unlock_memory(ptr, len) };
                assert!(unlock_res.is_ok());
            }
            Err(_) => assert!(true), // 平台/权限不支持时允许失败
        }
    }

    #[test]
    fn test_secure_allocator_unlock_all_idempotent() {
        let mut allocator = SecureAllocator::new();
        let _ = allocator.alloc_locked(32); // 可能因权限失败
        let _ = allocator.unlock_all();
        let second = allocator.unlock_all();
        assert!(second.is_ok());
    }

    #[test]
    fn test_secure_buffer_clone_independent() {
        let mut original = SecureBuffer::new(8).unwrap();
        let a = vec![0x11u8; 8];
        original.write(&a).unwrap();

        let mut cloned = original.clone();
        let b = vec![0x22u8; 8];
        cloned.write(&b).unwrap();

        assert_eq!(original.as_slice(), &a[..]);
        assert_eq!(cloned.as_slice(), &b[..]);
    }

    #[test]
    fn test_secure_buffer_clone_contents_equal_initially() {
        let mut original = SecureBuffer::new(6).unwrap();
        let data = [9u8, 8, 7, 6, 5, 4];
        original.write(&data).unwrap();

        let cloned = original.clone();
        assert_eq!(original.as_slice(), cloned.as_slice());
    }
}