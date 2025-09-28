/// 代表一个安全或操作警报。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Alert {
    pub message: String,
}

impl Alert {
    /// 创建一个新的警报。
    pub fn new(message: &str) -> Self {
        Self {
            message: message.to_string(),
        }
    }
}

pub fn send_alert(_alert: &Alert) { /* stub */
}
