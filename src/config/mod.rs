//! 配置模块：负责加载和管理环境变量（如 ENCRYPTION_KEY、NETWORK）。
use std::env;

/// 钱包配置结构体
#[derive(Debug, Clone)]
pub struct WalletConfig {
    /// 加密密钥（建议32字节）
    pub encryption_key: String,
    /// 网络类型（如 mainnet/testnet）
    pub network: String,
}

impl WalletConfig {
    /// 从环境变量加载配置，优雅返回 Result，避免 panic
    pub fn from_env() -> Result<Self, String> {
        let encryption_key = env::var("ENCRYPTION_KEY")
            .map_err(|_| "必须在环境变量中设置 ENCRYPTION_KEY".to_string())?;
        if encryption_key.len() != 64 {
            return Err(format!(
                "ENCRYPTION_KEY 长度必须为64个字符，当前长度为 {}",
                encryption_key.len()
            ));
        }
        if !encryption_key.chars().all(|c| c.is_ascii_hexdigit()) {
            return Err("ENCRYPTION_KEY 必须只包含十六进制字符 (0-9, a-f, A-F)".to_string());
        }
        let network = env::var("NETWORK").unwrap_or_else(|_| "testnet".to_string());
        Ok(WalletConfig { encryption_key, network })
    }
}
