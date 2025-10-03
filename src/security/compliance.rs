// src/security/compliance.rs
//! 鍚堣鎬ф鏌ユā鍧?//! 鐢ㄤ簬纭繚閽卞寘鎿嶄綔绗﹀悎娉曡瑕佹眰

use crate::tools::error::WalletError;
use std::collections::HashMap;

/// 鍚堣妫€鏌ョ粨鏋?#[derive(Debug, Clone, PartialEq)]
pub enum ComplianceResult {
    Compliant,
    NonCompliant(String),
    RequiresApproval(String),
}

/// 浜ゆ槗绫诲瀷
#[derive(Debug, Clone, PartialEq)]
pub enum TransactionType {
    Transfer,
    Receive,
    Swap,
    Stake,
    Unstake,
    Bridge,
}

/// 椋庨櫓绛夌骇
#[derive(Debug, Clone, PartialEq)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
    Critical,
}

/// 鍚堣妫€鏌ュ櫒
pub struct ComplianceChecker {
    max_daily_limit: f64,
    max_transaction_limit: f64,
    restricted_countries: Vec<String>,
    sanctioned_addresses: Vec<String>,
    user_daily_totals: HashMap<String, f64>,
}

impl ComplianceChecker {
    /// 鍒涘缓鏂扮殑鍚堣妫€鏌ュ櫒
    pub fn new() -> Self {
        Self {
            max_daily_limit: 10000.0,      // 姣忔棩鏈€澶т氦鏄撻檺棰?            max_transaction_limit: 1000.0, // 鍗曠瑪鏈€澶т氦鏄撻檺棰?            restricted_countries: vec![
                "IR".to_string(), // 浼婃湕
                "KP".to_string(), // 鏈濋矞
                "CU".to_string(), // 鍙ゅ反
                "SY".to_string(), // 鍙欏埄浜?            ],
            sanctioned_addresses: vec![
                // 杩欓噷搴旇鍖呭惈鍒惰鍦板潃鍒楄〃
                // 鍦ㄥ疄闄呭簲鐢ㄤ腑锛岃繖搴旇浠庡閮ㄦ暟鎹簮鍔犺浇
            ],
            user_daily_totals: HashMap::new(),
        }
    }

    /// 妫€鏌ヤ氦鏄撳悎瑙勬€?    pub fn check_transaction(
        &mut self,
        user_id: &str,
        transaction_type: &TransactionType,
        amount: f64,
        recipient_address: &str,
        user_country: &str,
    ) -> Result<ComplianceResult, WalletError> {
        // 妫€鏌ュ浗瀹堕檺鍒?        if self.restricted_countries.contains(&user_country.to_string()) {
            return Ok(ComplianceResult::NonCompliant(format!(
                "Transactions from {} are restricted",
                user_country
            )));
        }

        // 妫€鏌ュ埗瑁佸湴鍧€
        if self.sanctioned_addresses.contains(&recipient_address.to_string()) {
            return Ok(ComplianceResult::NonCompliant(
                "Recipient address is sanctioned".to_string(),
            ));
        }

        // 妫€鏌ヤ氦鏄撻噾棰濋檺鍒?        if amount > self.max_transaction_limit {
            return Ok(ComplianceResult::RequiresApproval(format!(
                "Transaction amount {} exceeds limit {}",
                amount, self.max_transaction_limit
            )));
        }

        // 妫€鏌ユ瘡鏃ラ檺棰?        let current_daily = *self.user_daily_totals.get(user_id).unwrap_or(&0.0);
        if current_daily + amount > self.max_daily_limit {
            return Ok(ComplianceResult::RequiresApproval(format!(
                "Daily limit would be exceeded: current {}, adding {}, limit {}",
                current_daily, amount, self.max_daily_limit
            )));
        }

        // 妫€鏌ヤ氦鏄撶被鍨嬬壒瀹氳鍒?        match transaction_type {
            TransactionType::Bridge => {
                // 璺ㄩ摼妗ユ帴鍙兘闇€瑕侀澶栨鏌?                if amount > self.max_transaction_limit * 0.5 {
                    return Ok(ComplianceResult::RequiresApproval(
                        "Large bridge transactions require approval".to_string(),
                    ));
                }
            }
            TransactionType::Swap => {
                // 浜ゆ崲浜ゆ槗鐨勬鏌?                // 杩欓噷鍙互娣诲姞鍘讳腑蹇冨寲浜ゆ槗鎵€鐨勭壒瀹氳鍒?            }
            _ => {}
        }

        // 鏇存柊姣忔棩鎬婚
        let new_total = current_daily + amount; // current_daily is already a f64
        self.user_daily_totals.insert(user_id.to_string(), new_total);

        Ok(ComplianceResult::Compliant)
    }

    /// 璇勪及浜ゆ槗椋庨櫓绛夌骇
    pub fn assess_risk(
        &self,
        transaction_type: &TransactionType,
        amount: f64,
        recipient_address: &str,
        user_history: usize, // 鐢ㄦ埛鍘嗗彶浜ゆ槗娆℃暟
    ) -> RiskLevel {
        let mut risk_score: u32 = 0;

        // 鍩轰簬閲戦鐨勯闄?        if amount > self.max_transaction_limit * 5.0 {
            // 鏋侀珮閲戦
            risk_score += 5; // 璋冩暣鏉冮噸浠ユ洿濂藉湴鍖哄垎椋庨櫓
        } else if amount > self.max_transaction_limit {
            risk_score += 3; // 楂橀闄?        }

        // 鍩轰簬浜ゆ槗绫诲瀷鐨勯闄?        match transaction_type {
            TransactionType::Bridge => risk_score += 2,
            TransactionType::Swap => risk_score += 1,
            TransactionType::Stake | TransactionType::Unstake => {} // Staking is often lower risk
            _ => {}
        }

        // 鍩轰簬鐢ㄦ埛鍘嗗彶鐨勯闄?        if user_history < 5 {
            risk_score += 2; // 鏂扮敤鎴烽闄╂洿楂?        }

        // 鍩轰簬鎺ユ敹鍦板潃鐨勯闄╋紙绠€鍖栨鏌ワ級
        // 鍦ㄧ湡瀹炰笘鐣屼腑锛岃繖閲屼細妫€鏌ュ湴鍧€鏄惁鍦ㄩ粦鍚嶅崟銆佹槸鍚︿笌娣峰竵鍣ㄤ氦浜掔瓑
        if recipient_address.len() < 20 {
            risk_score += 3; // 鍙枒鍦板潃锛屾彁楂樻潈閲?        }

        match risk_score {
            0..=2 => RiskLevel::Low,
            3..=5 => RiskLevel::Medium,
            6..=8 => RiskLevel::High,
            _ => RiskLevel::Critical, // 9+ is critical
        }
    }

    /// 閲嶇疆姣忔棩闄愰锛堥€氬父鍦ㄦ瘡鏃ラ噸缃换鍔′腑璋冪敤锛?    pub fn reset_daily_limits(&mut self) {
        self.user_daily_totals.clear();
    }

    /// 娣诲姞鍒惰鍦板潃
    pub fn add_sanctioned_address(&mut self, address: String) {
        if !self.sanctioned_addresses.contains(&address) {
            self.sanctioned_addresses.push(address);
        }
    }

    /// 绉婚櫎鍒惰鍦板潃
    pub fn remove_sanctioned_address(&mut self, address: &str) {
        self.sanctioned_addresses.retain(|a| a != address);
    }

    /// 鑾峰彇鐢ㄦ埛姣忔棩浣跨敤棰濆害
    pub fn get_user_daily_usage(&self, user_id: &str) -> f64 {
        *self.user_daily_totals.get(user_id).unwrap_or(&0.0)
    }

    /// 妫€鏌ュ湴鍧€鏄惁琚埗瑁?    pub fn is_address_sanctioned(&self, address: &str) -> bool {
        self.sanctioned_addresses.contains(&address.to_string())
    }
}

impl Default for ComplianceChecker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compliance_check() {
        let mut checker = ComplianceChecker::new();
        let user_id = "user123";
        let recipient = "0x1234567890abcdef";
        let country = "US";

        // 姝ｅ父浜ゆ槗搴旇鍚堣
        let result = checker
            .check_transaction(user_id, &TransactionType::Transfer, 100.0, recipient, country)
            .unwrap();
        assert_eq!(result, ComplianceResult::Compliant);

        // 妫€鏌ユ瘡鏃ユ€婚宸叉洿鏂?        assert_eq!(checker.get_user_daily_usage(user_id), 100.0);
    }

    #[test]
    fn test_transaction_limit() {
        let mut checker = ComplianceChecker::new();
        let user_id = "user123";

        // 瓒呰繃鍗曠瑪闄愰鐨勪氦鏄撻渶瑕佹壒鍑?        let result = checker
            .check_transaction(
                user_id,
                &TransactionType::Transfer,
                2000.0, // 瓒呰繃1000鐨勯檺棰?                "0x1234567890abcdef",
                "US",
            )
            .unwrap();

        match result {
            ComplianceResult::RequiresApproval(_) => {}
            _ => panic!("Expected approval required"),
        }
    }

    #[test]
    fn test_restricted_country() {
        let mut checker = ComplianceChecker::new();
        let user_id = "user123";

        // 鍙楅檺鍥藉鐨勪氦鏄撲笉鍚堣
        let result = checker
            .check_transaction(
                user_id,
                &TransactionType::Transfer,
                100.0,
                "0x1234567890abcdef",
                "IR", // 浼婃湕
            )
            .unwrap();

        match result {
            ComplianceResult::NonCompliant(_) => {}
            _ => panic!("Expected non-compliant"),
        }
    }

    #[test]
    fn test_risk_assessment() {
        let checker = ComplianceChecker::new();

        // 浣庨闄╀氦鏄?        let risk = checker.assess_risk(
            &TransactionType::Transfer,
            100.0,
            "0x1234567890abcdef1234567890abcdef",
            10, // 鏈夌粡楠岀殑鐢ㄦ埛
        );
        assert_eq!(risk, RiskLevel::Low);

        // 楂橀闄╀氦鏄?        // 璋冩暣娴嬭瘯鏁版嵁浠ヨ揪鍒?Critical
        let risk = checker.assess_risk(
            &TransactionType::Bridge,
            6000.0,  // 瓒呰繃 5000, +5
            "short", // len < 20, +3
            1,       // 鏂扮敤鎴? +2
        );
        assert_eq!(risk, RiskLevel::Critical); // 5 + 2 + 3 + 2 = 12, Critical
    }

    #[test]
    fn test_sanctioned_addresses() {
        let mut checker = ComplianceChecker::new();
        let sanctioned_addr = "0x1111111111111111111111111111111111111111";

        checker.add_sanctioned_address(sanctioned_addr.to_string());
        assert!(checker.is_address_sanctioned(sanctioned_addr));

        // 鍒惰鍦板潃鐨勪氦鏄撲笉鍚堣
        let result = checker
            .check_transaction("user123", &TransactionType::Transfer, 100.0, sanctioned_addr, "US")
            .unwrap();

        match result {
            ComplianceResult::NonCompliant(_) => {}
            _ => panic!("Expected non-compliant for sanctioned address"),
        }

        checker.remove_sanctioned_address(sanctioned_addr);
        assert!(!checker.is_address_sanctioned(sanctioned_addr));
    }
}
