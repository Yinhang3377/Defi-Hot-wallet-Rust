/// 代表一个敏感操作的确认请求。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Confirmation {
    pub tx_id: String,
    confirmed: bool,
}

impl Confirmation {
    /// 为一个给定的交易ID创建一个新的确认请求。
    pub fn new(tx_id: &str) -> Self {
        Self {
            tx_id: tx_id.to_string(),
            confirmed: false,
        }
    }

    /// 确认此操作。
    pub fn confirm(&mut self) {
        self.confirmed = true;
    }

    /// 检查此操作是否已确认。
    pub fn is_confirmed(&self) -> bool {
        self.confirmed
    }
}

pub fn require_confirmation(_op: &str) -> bool {
    true
}
