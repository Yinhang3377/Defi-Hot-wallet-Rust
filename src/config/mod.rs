//! 配置模块：负责加载和管理环境变量（如 SALT、NETWORK）。
use std::env;

/// 钱包配置结构体
#[derive(Debug, Clone)]
pub struct WalletConfig {
    #[allow(dead_code)]
    /// 盐值（建议32字节）
    pub salt: String,
    /// 网络类型（如 mainnet/testnet）
    pub network: String,
}

impl WalletConfig {
    /// 从环境变量加载配置，优雅返回 Result，避免 panic
    pub fn from_env() -> Result<Self, String> {
        let salt = env::var("SALT").map_err(|_| "必须在环境变量中设置 SALT".to_string())?;
        if salt.len() != 64 {
            return Err(format!("SALT 长度必须为64个字符，当前长度为 {}", salt.len()));
        }
        let network =
            env::var("NETWORK").map_err(|_| "必须在环境变量中设置 NETWORK".to_string())?;
        Ok(Self { salt, network })
    }
}
