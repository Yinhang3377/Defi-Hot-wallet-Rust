// src/security/memory_protection.rs
//! 鍐呭瓨淇濇姢妯″潡
//! 鐢ㄤ簬瀹夊叏澶勭悊鏁忔劅鏁版嵁锛岄槻姝㈠唴瀛樻硠闇?
use crate::tools::error::WalletError;
use std::alloc::{alloc, dealloc, Layout};
use std::ptr;

/// 瀹夊叏鍐呭瓨缂撳啿鍖?/// 鍦―rop鏃惰嚜鍔ㄦ竻闄ゅ唴瀹?pub struct SecureBuffer {
    ptr: *mut u8,
    len: usize,
    layout: Layout,
}

impl SecureBuffer {
    /// 鍒涘缓鏂扮殑瀹夊叏缂撳啿鍖?    pub fn new(size: usize) -> Result<Self, WalletError> {
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

    /// 鑾峰彇缂撳啿鍖洪暱搴?    pub fn len(&self) -> usize {
        self.len
    }

    /// 妫€鏌ョ紦鍐插尯鏄惁涓虹┖
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// 瀹夊叏鍦板啓鍏ユ暟鎹?    pub fn write(&mut self, data: &[u8]) -> Result<(), WalletError> {
        if data.len() > self.len {
            return Err(WalletError::InvalidInput("Data too large for buffer".to_string()));
        }

        unsafe {
            ptr::copy_nonoverlapping(data.as_ptr(), self.ptr, data.len());
        }

        Ok(())
    }

    /// 瀹夊叏鍦拌鍙栨暟鎹?    pub fn read(&self, dest: &mut [u8]) -> Result<usize, WalletError> {
        let read_len = dest.len().min(self.len);
        unsafe {
            ptr::copy_nonoverlapping(self.ptr, dest.as_mut_ptr(), read_len);
        }
        Ok(read_len)
    }

    /// 鑾峰彇鍙璁块棶
    pub fn as_slice(&self) -> &[u8] {
        unsafe { std::slice::from_raw_parts(self.ptr, self.len) }
    }

    /// 鑾峰彇鍙啓璁块棶锛堜笉瀹夊叏锛?    ///
    /// # Safety
    ///
    /// 璋冪敤鑰呭繀椤荤‘淇?
    /// - 涓嶄細鏈夊叾浠栧紩鐢ㄥ悓鏃惰闂浉鍚屽唴瀛?    /// - 涓嶄細瓒婄晫鍐欏叆鏁版嵁
    pub unsafe fn as_mut_slice(&mut self) -> &mut [u8] {
        std::slice::from_raw_parts_mut(self.ptr, self.len)
    }
}

impl Drop for SecureBuffer {
    fn drop(&mut self) {
        // 鍦ㄩ噴鏀惧墠娓呴櫎鍐呭瓨鍐呭
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

/// 娓呴櫎鏁忔劅鏁版嵁
/// 浣跨敤澶氱鏂规硶纭繚鏁版嵁琚鐩?///
/// # Safety
///
/// 姝ゅ嚱鏁伴渶瑕佷竴涓湁鏁堢殑鎸囬拡鍜岄暱搴︺€傝皟鐢ㄨ€呭繀椤荤‘淇?
/// - `ptr` 鎸囧悜鏈夋晥鐨勫唴瀛樺尯鍩燂紝骞朵笖鍙啓鍏?/// - `len` 涓嶈秴杩囧垎閰嶇粰 `ptr` 鐨勫唴瀛樺ぇ灏?/// - 鎿嶄綔鏈熼棿 `ptr` 涓嶄細琚叾浠栦唬鐮佽闂?pub unsafe fn clear_sensitive_data(ptr: *mut u8, len: usize) {
    if ptr.is_null() || len == 0 {
        return;
    }

    // 鏂规硶1: 鐢ㄩ浂瑕嗙洊
    ptr::write_bytes(ptr, 0, len);

    // 鏂规硶2: 鐢ㄤ吉闅忔満锛堢ず渚嬶級鏁版嵁瑕嗙洊
    for i in 0..len {
        *ptr.add(i) = (i % 256) as u8;
    }

    // 鏂规硶3: 鍐嶆鐢ㄩ浂瑕嗙洊
    ptr::write_bytes(ptr, 0, len);

    // 鏂规硶4: 鐢?xFF瑕嗙洊
    ptr::write_bytes(ptr, 0xFF, len);

    // 鏈€缁堢敤闆惰鐩?    ptr::write_bytes(ptr, 0, len);
}

/// 娓呴櫎鏁忔劅鏁版嵁缂撳啿鍖猴紙瀹夊叏鍖呰锛?pub fn clear_sensitive(buf: &mut [u8]) {
    unsafe {
        clear_sensitive_data(buf.as_mut_ptr(), buf.len());
    }
}

/// 瀹夊叏瀛楃涓茬被鍨?/// 鍦―rop鏃惰嚜鍔ㄦ竻闄ゅ唴瀹?pub struct SecureString {
    buffer: SecureBuffer,
}

impl SecureString {
    /// 鍒涘缓鏂扮殑瀹夊叏瀛楃涓?    pub fn new(s: &str) -> Result<Self, WalletError> {
        let mut buffer = SecureBuffer::new(s.len())?;
        buffer.write(s.as_bytes())?;
        Ok(Self { buffer })
    }

    /// 鑾峰彇瀛楃涓查暱搴?    pub fn len(&self) -> usize {
        self.buffer.len()
    }

    /// 妫€鏌ュ瓧绗︿覆鏄惁涓虹┖
    pub fn is_empty(&self) -> bool {
        self.buffer.len() == 0
    }

    /// 瀹夊叏鍦拌幏鍙栧瓧绗︿覆鍐呭
    pub fn reveal(&self) -> Result<String, WalletError> {
        let mut data = vec![0u8; self.len()];
        self.buffer.read(&mut data)?;
        String::from_utf8(data)
            .map_err(|e| WalletError::InvalidInput(format!("Invalid UTF-8: {}", e)))
    }
}

impl Drop for SecureString {
    fn drop(&mut self) {
        // SecureBuffer 鐨?Drop 浼氬鐞嗘竻闄?    }
}

/// 鍐呭瓨閿佸畾锛堥槻姝㈤〉闈氦鎹級
/// 娉ㄦ剰锛氳繖闇€瑕佺壒瀹氱殑绯荤粺鏉冮檺
///
/// # Safety
///
/// 璋冪敤鑰呭繀椤荤‘淇?
/// - `ptr` 鎸囧悜鏈夋晥鐨勩€佸凡鍒嗛厤鐨勫唴瀛?/// - `len` 涓嶈秴杩囧垎閰嶇殑鍐呭瓨澶у皬
/// - 閿佸畾鐨勫唴瀛樹笉浼氳繃澶氭秷鑰楃郴缁熻祫婧?pub unsafe fn lock_memory(ptr: *mut u8, len: usize) -> Result<(), WalletError> {
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

/// 鍐呭瓨瑙ｉ攣
///
/// # Safety
///
/// 璋冪敤鑰呭繀椤荤‘淇?
/// - `ptr` 鎸囧悜涔嬪墠閫氳繃 `lock_memory` 閿佸畾鐨勫唴瀛?/// - `len` 涓庨攣瀹氭椂浣跨敤鐨勭浉鍚?pub unsafe fn unlock_memory(ptr: *mut u8, len: usize) -> Result<(), WalletError> {
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

/// 瀹夊叏鍐呭瓨鍒嗛厤鍣?pub struct SecureAllocator {
    locked_pages: Vec<(usize, usize)>, // (ptr, size)
}

impl SecureAllocator {
    pub fn new() -> Self {
        Self { locked_pages: Vec::new() }
    }

    /// 鍒嗛厤骞堕攣瀹氬唴瀛?    pub fn alloc_locked(&mut self, size: usize) -> Result<SecureBuffer, WalletError> {
        let buffer = SecureBuffer::new(size)?;
        unsafe {
            lock_memory(buffer.ptr, buffer.len)?;
        }
        self.locked_pages.push((buffer.ptr as usize, buffer.len));
        Ok(buffer)
    }

    /// 瑙ｉ攣鎵€鏈夊垎閰嶇殑鍐呭瓨
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
        let _ = self.unlock_all(); // 蹇界暐閿欒锛屽洜涓烘垜浠鍦ㄦ竻鐞?    }
}

/// 涓存椂鏁忔劅鏁版嵁澶勭悊
/// 纭繚鍦ㄤ綔鐢ㄥ煙缁撴潫鏃舵竻闄ゆ暟鎹?pub struct TempSensitive<T, F>
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

        // 鍦ㄦ煇浜涚幆澧冧腑鍙兘闇€瑕佹潈闄愭墠鑳介攣瀹氬唴瀛?        let result = allocator.alloc_locked(64);
        if let Ok(buffer) = result {
            assert_eq!(buffer.len(), 64);
            // 瑙ｉ攣鎵€鏈夊唴瀛?            let _ = allocator.unlock_all();
        }
    }

    // 鎵╁睍鐢ㄤ緥

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
            Err(_) => assert!(true), // 骞冲彴/鏉冮檺涓嶆敮鎸佹椂鍏佽澶辫触
        }
    }

    #[test]
    fn test_secure_allocator_unlock_all_idempotent() {
        let mut allocator = SecureAllocator::new();
        let _ = allocator.alloc_locked(32); // 鍙兘鍥犳潈闄愬け璐?        let _ = allocator.unlock_all();
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
