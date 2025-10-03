// src/main.rs
//! DeFi 鐑挶鍖呬富绋嬪簭
//! 鎻愪緵鍛戒护琛屾帴鍙ｅ拰鏍稿績鍔熻兘
use clap::{Parser, Subcommand};
use defi_hot_wallet::core::config::{BlockchainConfig, StorageConfig, WalletConfig};
use defi_hot_wallet::core::WalletManager;
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
    /// 閰嶇疆鏂囦欢璺緞
    #[arg(short, long, value_name = "FILE")]
    config: Option<PathBuf>,

    /// 鏃ュ織绾у埆
    #[arg(short = 'l', long, value_name = "LOG_LEVEL", default_value = "info")]
    log_level: String,

    /// 瀛愬懡浠?    #[command(subcommand)]
    command: Option<Commands>, // 浣垮瓙鍛戒护鍙€?}

#[derive(Subcommand)]
pub enum Commands {
    /// 鍒涘缓鏂伴挶鍖?    Create {
        /// 閽卞寘鍚嶇О
        #[arg(short, long)]
        name: String,
        /// 杈撳嚭鏂囦欢璺緞
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
    /// 鏄剧ず閽卞寘淇℃伅
    Info {
        /// 閽卞寘鍚嶇О
        #[arg(short, long)]
        name: String,
    },
    /// 杞处
    Transfer {
        /// 閽卞寘鍚嶇О
        #[arg(short, long)]
        name: String,
        /// 鎺ユ敹鍦板潃
        #[arg(short, long)]
        to: String,
        /// 閲戦
        #[arg(short, long)]
        amount: String,
    },
    /// 鏌ヨ浣欓
    Balance {
        /// 閽卞寘鍚嶇О
        #[arg(short, long)]
        name: String,
    },
    /// 妗ユ帴杞处
    Bridge {
        /// 閽卞寘鍚嶇О
        #[arg(short, long)]
        name: String,
        /// 婧愰摼
        #[arg(long)]
        from_chain: String,
        /// 鐩爣閾?        #[arg(long)]
        to_chain: String,
        /// 浠ｅ竵
        #[arg(short, long)]
        token: String,
        /// 閲戦
        #[arg(short, long)]
        amount: String,
    },
    /// 鍒楀嚭鎵€鏈夐挶鍖?    List,
    /// 鐢熸垚鍔╄璇?    GenerateMnemonic,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    // 鍒濆鍖栨棩蹇?    init_logging(&cli.log_level)?;

    info!("Starting DeFi Hot Wallet v{}", env!("CARGO_PKG_VERSION"));

    // 浠庨粯璁ら厤缃姞杞斤紝骞跺厑璁搁€氳繃鐜鍙橀噺瑕嗙洊鏁版嵁搴?URL
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

    // 鎵ц鍛戒护
    match cli.command {
        Some(Commands::Create { name, output }) => {
            let info = wallet_manager.create_wallet(&name, true).await?;
            if let Some(output_path) = output {
                let wallet_data = serde_json::to_string_pretty(&info)?;
                fs::write(output_path, wallet_data)?;
            }
            println!("鉁?Wallet '{}' created successfully.", info.name);
        }
        Some(Commands::Info { name }) => {
            // This command is better served by `list` for now.
            println!("Wallet info for '{}': (use `list` for details)", name);
        }
        Some(Commands::Transfer { name, to, amount }) => {
            let tx_hash = wallet_manager.send_transaction(&name, &to, &amount, "eth").await?;
            println!("馃捀 Transaction sent! Hash: {}", tx_hash);
        }
        Some(Commands::Balance { name }) => {
            let balance = wallet_manager.get_balance(&name, "eth").await?;
            println!("馃挵 Balance for '{}': {} ETH", name, balance);
        }
        Some(Commands::Bridge { name, from_chain, to_chain, token, amount }) => {
            let bridge_id = wallet_manager
                .bridge_assets(&name, &from_chain, &to_chain, &token, &amount)
                .await?;
            println!("馃寜 Bridge transaction initiated with ID: {}", bridge_id);
        }
        Some(Commands::List) => {
            let wallets = wallet_manager.list_wallets().await?;
            println!("馃搵 Wallets:");
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
        .with_max_level(tracing::Level::TRACE) // 纭繚鎵€鏈夌骇鍒兘鑳借 env_filter 澶勭悊
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
        // 妯℃嫙 main 閫昏緫锛屼絾绠€鍖?        match cli.command {
            Some(Commands::Create { name, output: _ }) => {
                // 妯℃嫙鍒涘缓
                println!("Simulated create: {}", name);
            }
            Some(Commands::Transfer { name, to, amount }) => {
                // 妯℃嫙杞处
                println!("Simulated transfer from {} to {} amount {}", name, to, amount);
            }
            Some(Commands::Balance { name }) => {
                // 妯℃嫙鏌ヨ浣欓
                println!("Simulated balance check for {}", name);
            }
            Some(Commands::Info { name }) => {
                // 妯℃嫙鏌ヨ淇℃伅
                println!("Simulated info for {}", name);
            }
            Some(Commands::List) => {
                // 妯℃嫙鍒楀嚭
                println!("Simulated list wallets");
            }
            Some(Commands::GenerateMnemonic) => {
                // 妯℃嫙鐢熸垚鍔╄璇?                println!("abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon");
                // 24 瀛楃ず渚?            }
            None => {
                // 鏃犲瓙鍛戒护鏃惰繑鍥為敊璇?                return Err(WalletError::ValidationError(
                    "No subcommand provided. Use --help for usage.".into(),
                ));
            }
            _ => {
                // 瀵逛簬鍏朵粬鍛戒护锛屾殏鏃惰繑鍥為敊璇垨鎵撳嵃娑堟伅
                println!("Unsupported command in test");
            }
        }
        Ok(())
    }

    #[tokio::test(flavor = "current_thread")]
    async fn test_main_execution_help() {
        // 姝ｅ父璺緞锛氭ā鎷熶富鍑芥暟璋冪敤 --help
        // clap 鍦?--help 鏃朵細姝ｅ父閫€鍑猴紝杩欎細瀵艰嚧 try_parse_from 杩斿洖閿欒锛屼絾杩欐槸棰勬湡琛屼负銆?        let args = vec!["hot_wallet", "--help"];
        let result = run(args).await;
        // --help 鎵撳嵃淇℃伅骞朵互鎴愬姛鐘舵€侀€€鍑猴紝clap 鐨?try_parse_from 浼氬皢鍏惰涓洪敊璇?        assert!(result.is_err());
        if let Err(WalletError::ValidationError(e)) = result {
            assert!(e.contains("Usage: hot_wallet"));
        } else {
            panic!("Expected ValidationError error for --help");
        }
    }

    #[tokio::test(flavor = "current_thread")]
    async fn test_main_invalid_args() {
        // 閿欒璺緞锛氭棤鏁堝弬鏁?        let args = vec!["hot_wallet", "--invalid-arg"];
        let result = run(args).await;
        assert!(result.is_err());
        assert!(matches!(result, Err(WalletError::ValidationError(_))));
    }

    #[tokio::test(flavor = "current_thread")]
    async fn test_main_no_subcommand() {
        // 杈圭紭鎯呭喌锛氭棤瀛愬懡浠?        let args = vec!["hot_wallet"];
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
        assert!(result.is_ok()); // 鍋囪妯℃嫙鎴愬姛
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
