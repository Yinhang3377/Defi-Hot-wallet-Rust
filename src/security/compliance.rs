// src/security/compliance.rs
//! 合规性检查模块
//! 用于确保钱包操作符合法规要求

use crate::tools::error::WalletError;
use std::collections::HashMap;

/// 合规检查结果
#[derive(Debug, Clone, PartialEq)]
pub enum ComplianceResult {
    Compliant,
    NonCompliant(String),
    RequiresApproval(String),
}

/// 交易类型
#[derive(Debug, Clone, PartialEq)]
pub enum TransactionType {
    Transfer,
    Receive,
    Swap,
    Stake,
    Unstake,
    Bridge,
}

/// 风险等级
#[derive(Debug, Clone, PartialEq)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
    Critical,
}

/// 合规检查器
pub struct ComplianceChecker {
    max_daily_limit: f64,
    max_transaction_limit: f64,
    restricted_countries: Vec<String>,
    sanctioned_addresses: Vec<String>,
    user_daily_totals: HashMap<String, f64>,
}

impl ComplianceChecker {
    /// 创建新的合规检查器
    pub fn new() -> Self {
        Self {
            max_daily_limit: 10000.0,      // 每日最大交易限额
            max_transaction_limit: 1000.0, // 单笔最大交易限额
            restricted_countries: vec![
                "IR".to_string(), // 伊朗
                "KP".to_string(), // 朝鲜
                "CU".to_string(), // 古巴
                "SY".to_string(), // 叙利亚
            ],
            sanctioned_addresses: vec![
                // 这里应该包含制裁地址列表
                // 在实际应用中，这应该从外部数据源加载
            ],
            user_daily_totals: HashMap::new(),
        }
    }

    /// 检查交易合规性
    pub fn check_transaction(
        &mut self,
        user_id: &str,
        transaction_type: &TransactionType,
        amount: f64,
        recipient_address: &str,
        user_country: &str,
    ) -> Result<ComplianceResult, WalletError> {
        // 检查国家限制
        if self.restricted_countries.contains(&user_country.to_string()) {
            return Ok(ComplianceResult::NonCompliant(format!(
                "Transactions from {} are restricted",
                user_country
            )));
        }

        // 检查制裁地址
        if self.sanctioned_addresses.contains(&recipient_address.to_string()) {
            return Ok(ComplianceResult::NonCompliant(
                "Recipient address is sanctioned".to_string(),
            ));
        }

        // 检查交易金额限制
        if amount > self.max_transaction_limit {
            return Ok(ComplianceResult::RequiresApproval(format!(
                "Transaction amount {} exceeds limit {}",
                amount, self.max_transaction_limit
            )));
        }

        // 检查每日限额
        let current_daily = self.user_daily_totals.get(user_id).unwrap_or(&0.0);
        if current_daily + amount > self.max_daily_limit {
            return Ok(ComplianceResult::RequiresApproval(format!(
                "Daily limit would be exceeded: current {}, adding {}, limit {}",
                current_daily, amount, self.max_daily_limit
            )));
        }

        // 检查交易类型特定规则
        match transaction_type {
            TransactionType::Bridge => {
                // 跨链桥接可能需要额外检查
                if amount > self.max_transaction_limit * 0.5 {
                    return Ok(ComplianceResult::RequiresApproval(
                        "Large bridge transactions require approval".to_string(),
                    ));
                }
            }
            TransactionType::Swap => {
                // 交换交易的检查
                // 这里可以添加去中心化交易所的特定规则
            }
            _ => {}
        }

        // 更新每日总额
        let new_total = current_daily + amount;
        self.user_daily_totals.insert(user_id.to_string(), new_total);

        Ok(ComplianceResult::Compliant)
    }

    /// 评估交易风险等级
    pub fn assess_risk(
        &self,
        transaction_type: &TransactionType,
        amount: f64,
        recipient_address: &str,
        user_history: usize, // 用户历史交易次数
    ) -> RiskLevel {
        let mut risk_score: u32 = 0;

        // 基于金额的风险
        if amount > self.max_transaction_limit * 5.0 {
            // 极高金额
            risk_score += 5; // 调整权重以更好地区分风险
        } else if amount > self.max_transaction_limit {
            risk_score += 3; // 高风险
        }

        // 基于交易类型的风险
        match transaction_type {
            TransactionType::Bridge => risk_score += 2,
            TransactionType::Swap => risk_score += 1,
            TransactionType::Stake | TransactionType::Unstake => {} // Staking is often lower risk
            _ => {}
        }

        // 基于用户历史的风险
        if user_history < 5 {
            risk_score += 2; // 新用户风险更高
        }

        // 基于接收地址的风险（简化检查）
        // 在真实世界中，这里会检查地址是否在黑名单、是否与混币器交互等
        if recipient_address.len() < 20 {
            risk_score += 3; // 可疑地址，提高权重
        }

        match risk_score {
            0..=2 => RiskLevel::Low,
            3..=5 => RiskLevel::Medium,
            6..=8 => RiskLevel::High,
            _ => RiskLevel::Critical, // 9+ is critical
        }
    }

    /// 重置每日限额（通常在每日重置任务中调用）
    pub fn reset_daily_limits(&mut self) {
        self.user_daily_totals.clear();
    }

    /// 添加制裁地址
    pub fn add_sanctioned_address(&mut self, address: String) {
        if !self.sanctioned_addresses.contains(&address) {
            self.sanctioned_addresses.push(address);
        }
    }

    /// 移除制裁地址
    pub fn remove_sanctioned_address(&mut self, address: &str) {
        self.sanctioned_addresses.retain(|a| a != address);
    }

    /// 获取用户每日使用额度
    pub fn get_user_daily_usage(&self, user_id: &str) -> f64 {
        self.user_daily_totals.get(user_id).unwrap_or(&0.0).clone()
    }

    /// 检查地址是否被制裁
    pub fn is_address_sanctioned(&self, address: &str) -> bool {
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

        // 正常交易应该合规
        let result = checker
            .check_transaction(user_id, &TransactionType::Transfer, 100.0, recipient, country)
            .unwrap();
        assert_eq!(result, ComplianceResult::Compliant);

        // 检查每日总额已更新
        assert_eq!(checker.get_user_daily_usage(user_id), 100.0);
    }

    #[test]
    fn test_transaction_limit() {
        let mut checker = ComplianceChecker::new();
        let user_id = "user123";

        // 超过单笔限额的交易需要批准
        let result = checker
            .check_transaction(
                user_id,
                &TransactionType::Transfer,
                2000.0, // 超过1000的限额
                "0x1234567890abcdef",
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

        // 受限国家的交易不合规
        let result = checker
            .check_transaction(
                user_id,
                &TransactionType::Transfer,
                100.0,
                "0x1234567890abcdef",
                "IR", // 伊朗
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

        // 低风险交易
        let risk = checker.assess_risk(
            &TransactionType::Transfer,
            100.0,
            "0x1234567890abcdef1234567890abcdef",
            10, // 有经验的用户
        );
        assert_eq!(risk, RiskLevel::Low);

        // 高风险交易
        // 调整测试数据以达到 Critical
        let risk = checker.assess_risk(
            &TransactionType::Bridge,
            6000.0,  // 超过 5000, +5
            "short", // len < 20, +3
            1,       // 新用户, +2
        );
        assert_eq!(risk, RiskLevel::Critical); // 5 + 2 + 3 + 2 = 12, Critical
    }

    #[test]
    fn test_sanctioned_addresses() {
        let mut checker = ComplianceChecker::new();
        let sanctioned_addr = "0x1111111111111111111111111111111111111111";

        checker.add_sanctioned_address(sanctioned_addr.to_string());
        assert!(checker.is_address_sanctioned(sanctioned_addr));

        // 制裁地址的交易不合规
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
