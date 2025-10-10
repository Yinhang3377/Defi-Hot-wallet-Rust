﻿// src/core/bridge_test.rs
use defi_hot_wallet::blockchain::bridge::{ // 浣跨敤姝ｇ‘鐨勬ā鍧楄矾寰?
    mock::{EthereumToBSCBridge, EthereumToSolanaBridge, SolanaToEthereumBridge},
    BridgeTransactionStatus,
};
use defi_hot_wallet::blockchain::traits::Bridge;
use defi_hot_wallet::core::wallet::{SecureWalletData, WalletInfo};
use std::str::FromStr;
use uuid::Uuid;
use clap::{Parser, Subcommand};

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

// 妯℃嫙涓€涓?SecureWalletData 缁撴瀯浣撶敤浜庢祴璇?
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

async fn monitor_bridge_status(bridge: &impl Bridge, tx_hash: &str) {
    println!("馃攳 Monitoring bridge transaction: {}", tx_hash);
    for i in 1..=5 {
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        match bridge.check_transfer_status(tx_hash).await {
            Ok(status) => {
                println!("鈴憋笍  Status check {}: {:?}", i, status);
                if matches!(status, BridgeTransactionStatus::Completed) {
                    println!("鉁?Bridge transfer completed!");
                    break;
                }
                if let BridgeTransactionStatus::Failed(ref reason) = status {
                    println!("鉂?Bridge transfer failed: {}", reason);
                    break;
                }
            },
            Err(e) => {
                println!("鉂?Error checking status: {}", e);
                break;
            }
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();
    
    let cli = Cli::parse();
    let wallet_data = create_mock_wallet_data();
    
    match cli.command {
        Commands::EthToSol { amount, token } => {
            println!("馃寜 Testing ETH to Solana bridge with {} {}", amount, token);
            
            let bridge = EthereumToSolanaBridge::new("0xMockBridgeContract");
            let result = bridge.transfer_across_chains(
                "eth", "solana", &token, &amount, &wallet_data
            ).await?;
            
            println!("馃攧 Bridge transaction initiated: {}", result);
            monitor_bridge_status(&bridge, &result).await;
        },
        
        Commands::SolToEth { amount, token } => {
            println!("馃寜 Testing Solana to ETH bridge with {} {}", amount, token);
            
            let bridge = SolanaToEthereumBridge::new("0xMockReverseBridgeContract");
            let result = bridge.transfer_across_chains(
                "solana", "eth", &token, &amount, &wallet_data
            ).await?;
            
            println!("馃攧 Bridge transaction initiated: {}", result);
            monitor_bridge_status(&bridge, &result).await;
        },
        
        Commands::EthToBsc { amount, token } => {
            println!("馃寜 Testing ETH to BSC bridge with {} {}", amount, token);
            
            let bridge = EthereumToBSCBridge::new("0xMockEthBscBridge");
            let result = bridge.transfer_across_chains(
                "eth", "bsc", &token, &amount, &wallet_data
            ).await?;
            
            println!("馃攧 Bridge transaction initiated: {}", result);
            monitor_bridge_status(&bridge, &result).await;
        },
    }
    
    Ok(())
}
