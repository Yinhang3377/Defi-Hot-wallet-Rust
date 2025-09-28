// filepath: src\bin\bridge_test.rs
use clap::{Parser, Subcommand};
use defi_hot_wallet::blockchain::bridge::{
    Bridge, BridgeTransactionStatus, EthereumToBSCBridge, EthereumToSolanaBridge,
    SolanaToEthereumBridge,
};
use defi_hot_wallet::core::{SecureWalletData, WalletInfo};
use std::str::FromStr;
use uuid::Uuid;

#[derive(Parser)]
#[clap(name = "bridge-test", about = "Test cross-chain bridge functionality")]
struct Cli {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Test ETH to SOL bridge
    EthToSol {
        /// Amount to bridge
        #[clap(long, default_value = "10.0")]
        amount: String,

        /// Token symbol
        #[clap(long, default_value = "USDC")]
        token: String,
    },

    /// Test SOL to ETH bridge
    SolToEth {
        /// Amount to bridge
        #[clap(long, default_value = "10.0")]
        amount: String,

        /// Token symbol
        #[clap(long, default_value = "USDC")]
        token: String,
    },

    /// Test ETH to BSC bridge
    EthToBsc {
        /// Amount to bridge
        #[clap(long, default_value = "10.0")]
        amount: String,

        /// Token symbol
        #[clap(long, default_value = "USDT")]
        token: String,
    },
}

// 模拟一个 SecureWalletData 结构体用于测试
fn create_mock_wallet_data() -> SecureWalletData {
    SecureWalletData {
        info: WalletInfo {
            id: Uuid::from_str("12345678-1234-1234-1234-123456789012").unwrap(),
            name: "test-wallet".to_string(),
            created_at: chrono::Utc::now(),
            quantum_safe: true,
            multi_sig_threshold: 1,
            networks: vec!["eth".to_string(), "solana".to_string(), "bsc".to_string()],
        },
        encrypted_master_key: vec![1, 2, 3, 4],
        salt: vec![5, 6, 7, 8],
        nonce: vec![9, 10, 11, 12],
    }
}

// Helper function to monitor bridge transaction status
async fn monitor_bridge_status(bridge: &impl Bridge, tx_hash: &str) {
    println!("🔍 Monitoring bridge transaction: {}", tx_hash);

    // 设置最大检查次数和超时
    let max_checks = 10;
    let timeout = tokio::time::Duration::from_secs(20);
    let start_time = tokio::time::Instant::now();

    for i in 1..=max_checks {
        // 检查总时间是否已超时
        if start_time.elapsed() > timeout {
            println!("⏰ Monitoring timed out after {} seconds", timeout.as_secs());
            break;
        }

        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        match bridge.check_transfer_status(tx_hash).await {
            Ok(status) => {
                println!("⏱️  Status check {}: {:?}", i, status);
                if matches!(status, BridgeTransactionStatus::Completed) {
                    println!("✅ Bridge transfer completed!");
                }
                if let BridgeTransactionStatus::Failed(ref reason) = status {
                    println!("❌ Bridge transfer failed: {}", reason);
                }
            }
            Err(e) => {
                println!("❌ Error checking status: {}", e);
            }
        }
    }
}

// Helper function to execute a parsed bridge command
async fn execute_bridge_command(
    command: Commands,
    wallet_data: SecureWalletData,
) -> Result<(), Box<dyn std::error::Error>> {
    match command {
        Commands::EthToSol { amount, token } => {
            println!("🌉 Testing ETH to Solana bridge with {} {}", amount, token);

            let bridge = EthereumToSolanaBridge::new("0xMockBridgeContract");
            let result = bridge
                .transfer_across_chains("eth", "solana", &token, &amount, &wallet_data)
                .await?;

            println!("🔄 Bridge transaction initiated: {}", result);
            monitor_bridge_status(&bridge, &result).await;
        }

        Commands::SolToEth { amount, token } => {
            println!("🌉 Testing Solana to ETH bridge with {} {}", amount, token);

            let bridge = SolanaToEthereumBridge::new("0xMockReverseBridgeContract");
            let result = bridge
                .transfer_across_chains("solana", "eth", &token, &amount, &wallet_data)
                .await?;

            println!("🔄 Bridge transaction initiated: {}", result);
            monitor_bridge_status(&bridge, &result).await;
        }

        Commands::EthToBsc { amount, token } => {
            println!("🌉 Testing ETH to BSC bridge with {} {}", amount, token);

            let bridge = EthereumToBSCBridge::new("0xMockEthBscBridge");
            let result =
                bridge.transfer_across_chains("eth", "bsc", &token, &amount, &wallet_data).await?;

            println!("🔄 Bridge transaction initiated: {}", result);
            monitor_bridge_status(&bridge, &result).await;
        }
    }
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 设置详细日志
    tracing_subscriber::fmt::init();

    tracing::info!("🚀 Starting bridge test application");

    let cli = Cli::parse();
    let wallet_data = create_mock_wallet_data();

    execute_bridge_command(cli.command, wallet_data).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::CommandFactory; // For Cli::command().debug_assert()

    // Helper function to run a bridge test command programmatically
    async fn run_bridge_test(
        from_chain: &str,
        to_chain: &str,
        amount: &str,
        token: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Initialize tracing for tests if not already done by tokio::test
        let _ = tracing_subscriber::fmt::try_init(); // Use try_init to avoid re-initializing

        let args = match (from_chain, to_chain) {
            ("eth", "solana") => {
                vec!["bridge-test", "eth-to-sol", "--amount", amount, "--token", token]
            }
            ("solana", "eth") => {
                vec!["bridge-test", "sol-to-eth", "--amount", amount, "--token", token]
            }
            ("eth", "bsc") => {
                vec!["bridge-test", "eth-to-bsc", "--amount", amount, "--token", token]
            }
            _ => {
                return Err(format!("Unsupported chain pair: {} to {}", from_chain, to_chain).into())
            }
        };

        let cli = Cli::try_parse_from(args)?;
        let wallet_data = create_mock_wallet_data(); // Mock wallet data for tests
        execute_bridge_command(cli.command, wallet_data).await
    }

    #[test]
    fn verify_cli() {
        Cli::command().debug_assert();
    }

    #[tokio::test]
    async fn test_bridge_execution() {
        // 正常路径：桥接测试
        let result = run_bridge_test("eth", "solana", "10.0", "USDC").await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_bridge_invalid_chains() {
        // 错误路径：无效链
        let result = run_bridge_test("invalid", "solana", "10.0", "USDC").await;
        assert!(result.is_err());
        if let Err(e) = result {
            assert!(e.to_string().contains("Unsupported chain pair"));
        }
    }

    #[tokio::test]
    async fn test_bridge_zero_value() {
        // 边缘情况：零值
        let result = run_bridge_test("eth", "solana", "0.0", "USDC").await;
        // The mock bridge doesn't explicitly fail on zero amount, so this should be Ok
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_cli_parse_eth_to_sol() {
        let args = ["bridge-test", "eth-to-sol", "--amount", "5.0", "--token", "ETH"];
        let cli = Cli::try_parse_from(args).unwrap();
        match cli.command {
            Commands::EthToSol { amount, token } => {
                assert_eq!(amount, "5.0");
                assert_eq!(token, "ETH");
            }
            _ => panic!("Expected EthToSol command"),
        }
    }

    #[tokio::test]
    async fn test_cli_parse_sol_to_eth_defaults() {
        let args = ["bridge-test", "sol-to-eth"];
        let cli = Cli::try_parse_from(args).unwrap();
        match cli.command {
            Commands::SolToEth { amount, token } => {
                assert_eq!(amount, "10.0"); // Default value
                assert_eq!(token, "USDC"); // Default value
            }
            _ => panic!("Expected SolToEth command"),
        }
    }

    #[tokio::test]
    async fn test_cli_invalid_subcommand() {
        let args = ["bridge-test", "unknown-command"];
        let result = Cli::try_parse_from(args);
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_cli_missing_amount_for_eth_to_sol() {
        // amount has a default, so this should not fail parsing
        let args = ["bridge-test", "eth-to-sol"];
        let result = Cli::try_parse_from(args);
        assert!(result.is_ok());
    }
}
