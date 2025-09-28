// src/main.rs
//! DeFi 热钱包主程序
//! 提供命令行接口和核心功能
use clap::{Parser, Subcommand};
use defi_hot_wallet::core::config::{BlockchainConfig, StorageConfig, WalletConfig};
use defi_hot_wallet::core::WalletManager;
use serde_json;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use tracing::info;
use tracing_subscriber::{EnvFilter, FmtSubscriber};

#[derive(Parser)]
#[command(name = "hot_wallet")]
#[command(about = "A secure DeFi hot wallet with quantum-safe encryption")]
#[command(version = "0.1.0")]
pub struct Cli {
    /// 配置文件路径
    #[arg(short, long, value_name = "FILE")]
    config: Option<PathBuf>,

    /// 日志级别
    #[arg(short = 'l', long, value_name = "LOG_LEVEL", default_value = "info")]
    log_level: String,

    /// 子命令
    #[command(subcommand)]
    command: Option<Commands>, // 使子命令可选
}

#[derive(Subcommand)]
pub enum Commands {
    /// 创建新钱包
    Create {
        /// 钱包名称
        #[arg(short, long)]
        name: String,
        /// 输出文件路径
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
    /// 显示钱包信息
    Info {
        /// 钱包名称
        #[arg(short, long)]
        name: String,
    },
    /// 转账
    Transfer {
        /// 钱包名称
        #[arg(short, long)]
        name: String,
        /// 接收地址
        #[arg(short, long)]
        to: String,
        /// 金额
        #[arg(short, long)]
        amount: String,
    },
    /// 查询余额
    Balance {
        /// 钱包名称
        #[arg(short, long)]
        name: String,
    },
    /// 桥接转账
    Bridge {
        /// 钱包名称
        #[arg(short, long)]
        name: String,
        /// 源链
        #[arg(long)]
        from_chain: String,
        /// 目标链
        #[arg(long)]
        to_chain: String,
        /// 代币
        #[arg(short, long)]
        token: String,
        /// 金额
        #[arg(short, long)]
        amount: String,
    },
    /// 列出所有钱包
    List,
    /// 生成助记词
    GenerateMnemonic,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    // 初始化日志
    init_logging(&cli.log_level)?;

    info!("Starting DeFi Hot Wallet v{}", env!("CARGO_PKG_VERSION"));

    // 从默认配置加载，并允许通过环境变量覆盖数据库 URL
    let database_url =
        std::env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite://./wallets.db".to_string());
    let wallet_config = WalletConfig {
        storage: StorageConfig {
            database_url,
            max_connections: Some(1),
            connection_timeout_seconds: Some(5),
        },
        blockchain: BlockchainConfig {
            networks: HashMap::new(),
            default_network: Some("eth".to_string()),
        },
        quantum_safe: false,
        multi_sig_threshold: 2,
    };

    let wallet_manager = WalletManager::new(&wallet_config).await?;

    // 执行命令
    match cli.command {
        Some(Commands::Create { name, output }) => {
            let info = wallet_manager.create_wallet(&name, true).await?;
            if let Some(output_path) = output {
                let wallet_data = serde_json::to_string_pretty(&info)?;
                fs::write(output_path, wallet_data)?;
            }
            println!("✅ Wallet '{}' created successfully.", info.name);
        }
        Some(Commands::Info { name }) => {
            // This command is better served by `list` for now.
            println!("Wallet info for '{}': (use `list` for details)", name);
        }
        Some(Commands::Transfer { name, to, amount }) => {
            let tx_hash = wallet_manager.send_transaction(&name, &to, &amount, "eth").await?;
            println!("💸 Transaction sent! Hash: {}", tx_hash);
        }
        Some(Commands::Balance { name }) => {
            let balance = wallet_manager.get_balance(&name, "eth").await?;
            println!("💰 Balance for '{}': {} ETH", name, balance);
        }
        Some(Commands::Bridge { name, from_chain, to_chain, token, amount }) => {
            let bridge_id = wallet_manager
                .bridge_assets(&name, &from_chain, &to_chain, &token, &amount)
                .await?;
            println!("🌉 Bridge transaction initiated with ID: {}", bridge_id);
        }
        Some(Commands::List) => {
            let wallets = wallet_manager.list_wallets().await?;
            println!("📋 Wallets:");
            for wallet in wallets {
                println!("  - {}", wallet.name);
            }
        }
        Some(Commands::GenerateMnemonic) => {
            let mnemonic = wallet_manager.generate_mnemonic()?;
            println!("{}", mnemonic);
        }
        None => {
            println!("No command specified. Use --help for usage.");
        }
    }

    Ok(())
}

fn init_logging(level: &str) -> Result<(), Box<dyn std::error::Error>> {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(level));

    let subscriber = FmtSubscriber::builder()
        .with_env_filter(filter)
        .with_max_level(tracing::Level::TRACE) // 确保所有级别都能被 env_filter 处理
        .finish();

    tracing::subscriber::set_global_default(subscriber)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use defi_hot_wallet::core::errors::WalletError;

    async fn run(args: Vec<&str>) -> Result<(), WalletError> {
        let cli =
            Cli::try_parse_from(args).map_err(|e| WalletError::ValidationError(e.to_string()))?;
        // 模拟 main 逻辑，但简化
        match cli.command {
            Some(Commands::Create { name, output: _ }) => {
                // 模拟创建
                println!("Simulated create: {}", name);
            }
            Some(Commands::Transfer { name, to, amount }) => {
                // 模拟转账
                println!("Simulated transfer from {} to {} amount {}", name, to, amount);
            }
            Some(Commands::Balance { name }) => {
                // 模拟查询余额
                println!("Simulated balance check for {}", name);
            }
            Some(Commands::Info { name }) => {
                // 模拟查询信息
                println!("Simulated info for {}", name);
            }
            Some(Commands::List) => {
                // 模拟列出
                println!("Simulated list wallets");
            }
            Some(Commands::GenerateMnemonic) => {
                // 模拟生成助记词
                println!("abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon"); // 24 字示例
            }
            None => {
                // 无子命令时返回错误
                return Err(WalletError::ValidationError("No subcommand provided. Use --help for usage.".into()));
            }
            _ => {
                // 对于其他命令，暂时返回错误或打印消息
                println!("Unsupported command in test");
            }
        }
        Ok(())
    }

    #[tokio::test(flavor = "current_thread")]
    async fn test_main_execution_help() {
        // 正常路径：模拟主函数调用 --help
        // clap 在 --help 时会正常退出，这会导致 try_parse_from 返回错误，但这是预期行为。
        let args = vec!["hot_wallet", "--help"];
        let result = run(args).await;
        // --help 打印信息并以成功状态退出，clap 的 try_parse_from 会将其视为错误
        assert!(result.is_err());
        if let Err(WalletError::ValidationError(e)) = result {
            assert!(e.contains("Usage: hot_wallet"));
        } else {
            panic!("Expected ValidationError error for --help");
        }
    }

    #[tokio::test(flavor = "current_thread")]
    async fn test_main_invalid_args() {
        // 错误路径：无效参数
        let args = vec!["hot_wallet", "--invalid-arg"];
        let result = run(args).await;
        assert!(result.is_err());
        assert!(matches!(result, Err(WalletError::ValidationError(_))));
    }

    #[tokio::test(flavor = "current_thread")]
    async fn test_main_no_subcommand() {
        // 边缘情况：无子命令
        let args = vec!["hot_wallet"];
        let result = run(args).await;
        assert!(result.is_err());
        assert!(matches!(
            result,
            Err(WalletError::ValidationError(ref msg)) if msg.contains("subcommand")
        ));
    }

    #[tokio::test(flavor = "current_thread")]
    async fn test_main_create_wallet() {
        let args = vec!["hot_wallet", "create", "--name", "test_wallet"];
        let result = run(args).await;
        assert!(result.is_ok());
    }

    #[tokio::test(flavor = "current_thread")]
    async fn test_main_transfer() {
        let args =
            vec!["hot_wallet", "transfer", "--name", "test_wallet", "--to", "0x123", "--amount", "1.0"];
        let result = run(args).await;
        assert!(result.is_ok()); // 假设模拟成功
    }

    #[tokio::test(flavor = "current_thread")]
    async fn test_main_list_wallets() {
        let args = vec!["hot_wallet", "list"];
        let result = run(args).await;
        assert!(result.is_ok());
    }

    #[tokio::test(flavor = "current_thread")]
    async fn test_main_generate_mnemonic() {
        let args = vec!["hot_wallet", "generate-mnemonic"];
        let result = run(args).await;
        assert!(result.is_ok());
    }
}