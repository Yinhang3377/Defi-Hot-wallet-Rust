/// 浠ｈ〃涓€涓拡瀵圭壒瀹氫氦鏄撶殑鍥炴粴鎿嶄綔銆?#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Rollback {
    pub tx_id: String,
}

impl Rollback {
    /// 涓轰竴涓粰瀹氱殑浜ゆ槗ID鍒涘缓涓€涓柊鐨勫洖婊氳姹傘€?    pub fn new(tx_id: &str) -> Self {
        Self { tx_id: tx_id.to_string() }
    }
}

/// 鎵ц鍥炴粴鎿嶄綔鐨勫崰浣嶅嚱鏁般€?pub fn rollback_tx(_tx_id: &str) -> Result<(), &'static str> {
    Ok(())
}
