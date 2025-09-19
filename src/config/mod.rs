//! 配置模块：负责加载和管理环境变量（如 ENCRYPTION_KEY、NETWORK、SALT）。
//! 该目录下的 `config.rs` 定义了包含 `salt` 字段的完整 `WalletConfig`。

pub mod config;

// 方便外部直接使用 WalletConfig
pub use config::WalletConfig;
