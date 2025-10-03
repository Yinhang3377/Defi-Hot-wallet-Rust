/// 浠ｈ〃涓€涓晱鎰熸搷浣滅殑纭璇锋眰銆?#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Confirmation {
    pub tx_id: String,
    confirmed: bool,
}

impl Confirmation {
    /// 涓轰竴涓粰瀹氱殑浜ゆ槗ID鍒涘缓涓€涓柊鐨勭‘璁よ姹傘€?    pub fn new(tx_id: &str) -> Self {
        Self { tx_id: tx_id.to_string(), confirmed: false }
    }

    /// 纭姝ゆ搷浣溿€?    pub fn confirm(&mut self) {
        self.confirmed = true;
    }

    /// 妫€鏌ユ鎿嶄綔鏄惁宸茬‘璁ゃ€?    pub fn is_confirmed(&self) -> bool {
        self.confirmed
    }
}

pub fn require_confirmation(_op: &str) -> bool {
    true
}
