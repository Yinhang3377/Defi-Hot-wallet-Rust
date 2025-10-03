/// 浠ｈ〃涓€涓畨鍏ㄦ垨鎿嶄綔璀︽姤銆?#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Alert {
    pub message: String,
}

impl Alert {
    /// 鍒涘缓涓€涓柊鐨勮鎶ャ€?    pub fn new(message: &str) -> Self {
        Self { message: message.to_string() }
    }
}

pub fn send_alert(_alert: &Alert) { /* stub */
}
