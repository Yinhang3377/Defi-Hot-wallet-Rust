// src/main.rs
//! DeFi 热钱包入口
//! 提供钱包生命周期和基本 CLI/Server 启动
use clap::{Parser, Subcommand};
use defi_hot_wallet::core::config::{BlockchainConfig, StorageConfig, WalletConfig};
use defi_hot_wallet::core::WalletManager;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use tracing::info;
use tracing_subscriber::{EnvFilter, FmtSubscriber};
use zeroize::Zeroize;

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
    command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Create a new wallet
    Create {
        /// Wallet name
        #[arg(short, long)]
        name: String,
        /// Output file path (writes wallet data to file if present)
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
    /// Show wallet info
    Info {
        /// Wallet name
        #[arg(short, long)]
        name: String,
    },
    /// Transfer assets
    Transfer {
        /// Wallet name
        #[arg(short, long)]
        name: String,
        /// Recipient address
        #[arg(short, long)]
        to: String,
        /// Amount
        #[arg(short, long)]
        amount: String,
    },
    /// Query balance
    Balance {
        /// Wallet name
        #[arg(short, long)]
        name: String,
    },
    /// Bridge assets
    Bridge {
        /// Wallet name
        #[arg(short, long)]
        name: String,
        /// From chain
        #[arg(long)]
        from_chain: String,
        /// To chain
        #[arg(long)]
        to_chain: String,
        /// Token
        #[arg(short, long)]
        token: String,
        /// Amount
        #[arg(short, long)]
        amount: String,
    },
    /// List wallets
    List,
    /// Generate mnemonic (will not log full mnemonic)
    GenerateMnemonic,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    // 初始化日志
    init_logging(&cli.log_level)?;

    info!("Starting DeFi Hot Wallet v{}", env!("CARGO_PKG_VERSION"));

    // 数据库 URL（环境变量或默认 sqlite 文件）
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

    // 执行子命令
    match cli.command {
        Some(Commands::Create { name, output }) => {
            let info = wallet_manager.create_wallet(&name, true).await?;
            if let Some(output_path) = output {
                // 将钱包信息写入文件（谨慎：文件包含敏感信息，请确保文件权限）
                let wallet_data = serde_json::to_string_pretty(&info)?;
                fs::write(&output_path, wallet_data)?;
                println!("Wallet '{}' created and written to {}", info.name, output_path.display());
            } else {
                // 不在控制台打印完整钱包数据以避免泄露敏感信息
                println!("Wallet '{}' created. To persist full wallet data use --output <path>.", info.name);
            }
        }
        Some(Commands::Info { name }) => {
            // Info may include non-sensitive metadata; prefer WalletManager API that omits secrets.
            println!("Wallet info for '{}': (use `list` for details)", name);
        }
        Some(Commands::Transfer { name, to, amount }) => {
            let tx_hash = wallet_manager.send_transaction(&name, &to, &amount, "eth").await?;
            println!("Transaction sent! Hash: {}", tx_hash);
        }
        Some(Commands::Balance { name }) => {
            let balance = wallet_manager.get_balance(&name, "eth").await?;
            println!("Balance for '{}': {} ETH", name, balance);
        }
        Some(Commands::Bridge { name, from_chain, to_chain, token, amount }) => {
            let bridge_id = wallet_manager
                .bridge_assets(&name, &from_chain, &to_chain, &token, &amount)
                .await?;
            println!("Bridge transaction initiated with ID: {}", bridge_id);
        }
        Some(Commands::List) => {
            let wallets = wallet_manager.list_wallets().await?;
            println!("Wallets:");
            for wallet in wallets {
                println!("  - {}", wallet.name);
            }
        }
        Some(Commands::GenerateMnemonic) => {
            // 生成助记词：出于安全考虑，不在日志中记录完整助记词。
            // 将助记词保存在内存后立即使用并清零。
            let mut mnemonic = wallet_manager.generate_mnemonic()?;
            // 如果确实需要在控制台显示，用户应明确设置环境变量 ALLOW_PLAINTEXT_MNEMONIC=1
            // 以避免意外泄露。默认只显示提示与 fingerprint-like info.
            match std::env::var("ALLOW_PLAINTEXT_MNEMONIC") {
                Ok(val) if val == "1" => {
                    println!("{}", mnemonic);
                }
                _ => {
                    println!("Mnemonic generated. To display it in plaintext set ALLOW_PLAINTEXT_MNEMONIC=1 (not recommended).");
                }
            }
            // 清除助记词在内存中的副本
            mnemonic.zeroize();
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
        .with_max_level(tracing::Level::TRACE) // allow env_filter to narrow it down
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

        match cli.command {
            Some(Commands::Create { name, .. }) => {
                println!("Simulated create: {}", name);
            }
            Some(Commands::Transfer { name, to, amount }) => {
                println!("Simulated transfer from {} to {} amount {}", name, to, amount);
            }
            Some(Commands::Balance { name }) => {
                println!("Simulated balance check for {}", name);
            }
            Some(Commands::Info { name }) => {
                println!("Simulated info for {}", name);
            }
            Some(Commands::Bridge { name: _, from_chain, to_chain, token, amount }) => {
                println!(
                    "Simulated bridge from {} to {} token {} amount {}",
                    from_chain, to_chain, token, amount
                );
            }
            Some(Commands::List) => {
                println!("Simulated list wallets");
            }
            Some(Commands::GenerateMnemonic) => {
                println!("abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about abandon abandon abandon abandon");
            }
            None => {
                return Err(WalletError::ValidationError(
                    "No subcommand provided. Use --help for usage.".into(),
                ));
            }
        }
        Ok(())
    }

    #[tokio::test(flavor = "current_thread")]
    async fn test_main_execution_help() {
        let args = vec!["hot_wallet", "--help"];
        let result = run(args).await;
        assert!(result.is_err());
        if let Err(WalletError::ValidationError(e)) = result {
            assert!(e.contains("Usage") || e.contains("usage"));
        } else {
            panic!("Expected ValidationError error for --help");
        }
    }

    #[tokio::test(flavor = "current_thread")]
    async fn test_main_invalid_args() {
        let args = vec!["hot_wallet", "--invalid-arg"];
        let result = run(args).await;
        assert!(result.is_err());
        assert!(matches!(result, Err(WalletError::ValidationError(_))));
    }

    #[tokio::test(flavor = "current_thread")]
    async fn test_main_no_subcommand() {
        let args = vec!["hot_wallet"];
        let result = run(args).await;
        assert!(result.is_err());
        assert!(matches!(
            result,
            Err(WalletError::ValidationError(ref msg)) if msg.contains("No subcommand")
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
        let args = vec![
            "hot_wallet",
            "transfer",
            "--name",
            "test_wallet",
            "--to",
            "0x123",
            "--amount",
            "1.0",
        ];
        let result = run(args).await;
        assert!(result.is_ok());
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
```// filepath: c:\Users\plant\Desktop\Rust区块链\Defi-Hot-wallet-Rust\src\main.rs
// src/main.rs
//! DeFi 热钱包入口
//! 提供钱包生命周期和基本 CLI/Server 启动
use clap::{Parser, Subcommand};
use defi_hot_wallet::core::config::{BlockchainConfig, StorageConfig, WalletConfig};
use defi_hot_wallet::core::WalletManager;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use tracing::info;
use tracing_subscriber::{EnvFilter, FmtSubscriber};
use zeroize::Zeroize;

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
    command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Create a new wallet
    Create {
        /// Wallet name
        #[arg(short, long)]
        name: String,
        /// Output file path (writes wallet data to file if present)
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
    /// Show wallet info
    Info {
        /// Wallet name
        #[arg(short, long)]
        name: String,
    },
    /// Transfer assets
    Transfer {
        /// Wallet name
        #[arg(short, long)]
        name: String,
        /// Recipient address
        #[arg(short, long)]
        to: String,
        /// Amount
        #[arg(short, long)]
        amount: String,
    },
    /// Query balance
    Balance {
        /// Wallet name
        #[arg(short, long)]
        name: String,
    },
    /// Bridge assets
    Bridge {
        /// Wallet name
        #[arg(short, long)]
        name: String,
        /// From chain
        #[arg(long)]
        from_chain: String,
        /// To chain
        #[arg(long)]
        to_chain: String,
        /// Token
        #[arg(short, long)]
        token: String,
        /// Amount
        #[arg(short, long)]
        amount: String,
    },
    /// List wallets
    List,
    /// Generate mnemonic (will not log full mnemonic)
    GenerateMnemonic,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    // 初始化日志
    init_logging(&cli.log_level)?;

    info!("Starting DeFi Hot Wallet v{}", env!("CARGO_PKG_VERSION"));

    // 数据库 URL（环境变量或默认 sqlite 文件）
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

    // 执行子命令
    match cli.command {
        Some(Commands::Create { name, output }) => {
            let info = wallet_manager.create_wallet(&name, true).await?;
            if let Some(output_path) = output {
                // 将钱包信息写入文件（谨慎：文件包含敏感信息，请确保文件权限）
                let wallet_data = serde_json::to_string_pretty(&info)?;
                fs::write(&output_path, wallet_data)?;
                println!("Wallet '{}' created and written to {}", info.name, output_path.display());
            } else {
                // 不在控制台打印完整钱包数据以避免泄露敏感信息
                println!("Wallet '{}' created. To persist full wallet data use --output <path>.", info.name);
            }
        }
        Some(Commands::Info { name }) => {
            // Info may include non-sensitive metadata; prefer WalletManager API that omits secrets.
            println!("Wallet info for '{}': (use `list` for details)", name);
        }
        Some(Commands::Transfer { name, to, amount }) => {
            let tx_hash = wallet_manager.send_transaction(&name, &to, &amount, "eth").await?;
            println!("Transaction sent! Hash: {}", tx_hash);
        }
        Some(Commands::Balance { name }) => {
            let balance = wallet_manager.get_balance(&name, "eth").await?;
            println!("Balance for '{}': {} ETH", name, balance);
        }
        Some(Commands::Bridge { name, from_chain, to_chain, token, amount }) => {
            let bridge_id = wallet_manager
                .bridge_assets(&name, &from_chain, &to_chain, &token, &amount)
                .await?;
            println!("Bridge transaction initiated with ID: {}", bridge_id);
        }
        Some(Commands::List) => {
            let wallets = wallet_manager.list_wallets().await?;
            println!("Wallets:");
            for wallet in wallets {
                println!("  - {}", wallet.name);
            }
        }
        Some(Commands::GenerateMnemonic) => {
            // 生成助记词：出于安全考虑，不在日志中记录完整助记词。
            // 将助记词保存在内存后立即使用并清零。
            let mut mnemonic = wallet_manager.generate_mnemonic()?;
            // 如果确实需要在控制台显示，用户应明确设置环境变量 ALLOW_PLAINTEXT_MNEMONIC=1
            // 以避免意外泄露。默认只显示提示与 fingerprint-like info.
            match std::env::var("ALLOW_PLAINTEXT_MNEMONIC") {
                Ok(val) if val == "1" => {
                    println!("{}", mnemonic);
                }
                _ => {
                    println!("Mnemonic generated. To display it in plaintext set ALLOW_PLAINTEXT_MNEMONIC=1 (not recommended).");
                }
            }
            // 清除助记词在内存中的副本
            mnemonic.zeroize();
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
        .with_max_level(tracing::Level::TRACE) // allow env_filter to narrow it down
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

        match cli.command {
            Some(Commands::Create { name, .. }) => {
                println!("Simulated create: {}", name);
            }
            Some(Commands::Transfer { name, to, amount }) => {
                println!("Simulated transfer from {} to {} amount {}", name, to, amount);
            }
            Some(Commands::Balance { name }) => {
                println!("Simulated balance check for {}", name);
            }
            Some(Commands::Info { name }) => {
                println!("Simulated info for {}", name);
            }
            Some(Commands::Bridge { name: _, from_chain, to_chain, token, amount }) => {
                println!(
                    "Simulated bridge from {} to {} token {} amount {}",
                    from_chain, to_chain, token, amount
                );
            }
            Some(Commands::List) => {
                println!("Simulated list wallets");
            }
            Some(Commands::GenerateMnemonic) => {
                println!("abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about abandon abandon abandon abandon");
            }
            None => {
                return Err(WalletError::ValidationError(
                    "No subcommand provided. Use --help for usage.".into(),
                ));
            }
        }
        Ok(())
    }

    #[tokio::test(flavor = "current_thread")]
    async fn test_main_execution_help() {
        let args = vec!["hot_wallet", "--help"];
        let result = run(args).await;
        assert!(result.is_err());
        if let Err(WalletError::ValidationError(e)) = result {
            assert!(e.contains("Usage") || e.contains("usage"));
        } else {
            panic!("Expected ValidationError error for --help");
        }
    }

    #[tokio::test(flavor = "current_thread")]
    async fn test_main_invalid_args() {
        let args = vec!["hot_wallet", "--invalid-arg"];
        let result = run(args).await;
        assert!(result.is_err());
        assert!(matches!(result, Err(WalletError::ValidationError(_))));
    }

    #[tokio::test(flavor = "current_thread")]
    async fn test_main_no_subcommand() {
        let args = vec!["hot_wallet"];
        let result = run(args).await;
        assert!(result.is_err());
        assert!(matches!(
            result,
            Err(WalletError::ValidationError(ref msg)) if msg.contains("No subcommand")
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
        let args = vec![
            "hot_wallet",
            "transfer",
            "--name",
            "test_wallet",
            "--to",
            "0x123",
            "--amount",
            "1.0",
        ];
        let result = run(args).await;
        assert!(result.is_ok());
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