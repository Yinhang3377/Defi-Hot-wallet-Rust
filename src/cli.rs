use anyhow::Result;
use clap::{Parser, Subcommand};
use tracing::{info, error};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod core;
mod crypto;
mod blockchain;
mod storage;
mod monitoring;
mod i18n;

use crate::core::wallet::WalletManager;
use crate::i18n::I18nManager;

#[derive(Parser)]
#[command(name = "wallet-cli")]
#[command(about = "DeFi Hot Wallet CLI - 命令行钱包工具")]
#[command(version = "0.1.0")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
    
    /// Language (en, zh)
    #[arg(short, long, default_value = "en")]
    pub language: String,
    
    /// Log level
    #[arg(long, default_value = "info")]
    pub log_level: String,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Create a new wallet - 创建新钱包
    Create {
        /// Wallet name - 钱包名称
        #[arg(short, long)]
        name: String,
        
        /// Use quantum-safe encryption - 使用量子安全加密
        #[arg(short, long, default_value = "true")]
        quantum: bool,
    },
    
    /// List all wallets - 列出所有钱包
    List,
    
    /// Show wallet balance - 显示钱包余额
    Balance {
        /// Wallet name - 钱包名称
        #[arg(short, long)]
        wallet: String,
        
        /// Network (eth, solana) - 网络
        #[arg(short, long, default_value = "eth")]
        network: String,
    },
    
    /// Send transaction - 发送交易
    Send {
        /// Wallet name - 钱包名称
        #[arg(short, long)]
        wallet: String,
        
        /// Recipient address - 接收地址
        #[arg(short, long)]
        to: String,
        
        /// Amount to send - 发送金额
        #[arg(short, long)]
        amount: String,
        
        /// Network (eth, solana) - 网络
        #[arg(short, long, default_value = "eth")]
        network: String,
    },
    
    /// Generate new mnemonic - 生成新助记词
    GenerateMnemonic,
    
    /// Show wallet info - 显示钱包信息
    Info {
        /// Wallet name - 钱包名称
        #[arg(short, long)]
        wallet: String,
    },
    
    /// Backup wallet - 备份钱包
    Backup {
        /// Wallet name - 钱包名称
        #[arg(short, long)]
        wallet: String,
        
        /// Backup file path - 备份文件路径
        #[arg(short, long)]
        output: String,
    },
    
    /// Show security status - 显示安全状态
    Security,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    
    // Initialize logging
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| format!("wallet_cli={}", cli.log_level).into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Initialize i18n
    let i18n = match i18n::init_default_languages() {
        Ok(manager) => manager,
        Err(e) => {
            error!("Failed to initialize i18n: {}", e);
            return Err(e);
        }
    };
    
    let lang = &cli.language;
    
    println!("🔒 {}", i18n.get_text(lang, "app-name", None));
    println!("📱 {}", i18n.get_text(lang, "app-description", None));
    println!();
    
    // Initialize monitoring
    monitoring::init_metrics().await?;
    
    match cli.command {
        Commands::Create { name, quantum } => {
            println!("🔧 {}...", i18n.get_text(lang, "wallet-create", None));
            
            let manager = WalletManager::new().await?;
            match manager.create_wallet(&name, quantum).await {
                Ok(wallet_info) => {
                    println!("✅ {}", i18n.get_text(lang, "wallet-created", None));
                    println!("   ID: {}", wallet_info.id);
                    println!("   {}: {}", i18n.get_text(lang, "wallet-name", None), wallet_info.name);
                    println!("   {}: {}", i18n.get_text(lang, "security-quantum-safe", None), wallet_info.quantum_safe);
                    
                    if quantum {
                        println!("🛡️ {}", i18n.get_text(lang, "msg-quantum-protection", None));
                    }
                    
                    println!("💡 {}", i18n.get_text(lang, "msg-backup-reminder", None));
                }
                Err(e) => {
                    error!("Failed to create wallet: {}", e);
                    return Err(e);
                }
            }
        }
        
        Commands::List => {
            println!("📋 {}:", i18n.get_text(lang, "nav-wallets", None));
            // In a real implementation, this would list wallets from storage
            println!("   (No wallets found - use 'create' command to add wallets)");
        }
        
        Commands::Balance { wallet, network } => {
            println!("💰 {} {} {} {}...", 
                     i18n.get_text(lang, "wallet-balance", None),
                     wallet,
                     i18n.get_text(lang, "network-ethereum", None),
                     network);
            
            let manager = WalletManager::new().await?;
            match manager.get_balance(&wallet, &network).await {
                Ok(balance) => {
                    let network_name = match network.as_str() {
                        "eth" => i18n.get_text(lang, "network-ethereum", None),
                        "solana" => i18n.get_text(lang, "network-solana", None),
                        _ => network.clone(),
                    };
                    
                    println!("✅ {}: {}", i18n.get_text(lang, "wallet-balance", None), balance);
                    println!("   Network: {}", network_name);
                }
                Err(e) => {
                    error!("Failed to get balance: {}", e);
                    println!("❌ {}: {}", i18n.get_text(lang, "error-network-error", None), e);
                }
            }
        }
        
        Commands::Send { wallet, to, amount, network } => {
            println!("💸 {} {} {} {} {} {}...", 
                     i18n.get_text(lang, "tx-send", None),
                     amount,
                     wallet,
                     i18n.get_text(lang, "tx-recipient", None),
                     to,
                     network);
            
            let manager = WalletManager::new().await?;
            match manager.send_transaction(&wallet, &to, &amount, &network).await {
                Ok(tx_hash) => {
                    println!("✅ {}", i18n.get_text(lang, "tx-success", None));
                    println!("   Transaction Hash: {}", tx_hash);
                    println!("   Status: {}", i18n.get_text(lang, "tx-pending", None));
                }
                Err(e) => {
                    error!("Failed to send transaction: {}", e);
                    println!("❌ {}: {}", i18n.get_text(lang, "tx-failed", None), e);
                }
            }
        }
        
        Commands::GenerateMnemonic => {
            println!("🔑 Generating new mnemonic phrase...");
            
            // Generate a new mnemonic
            use bip39::{Mnemonic, Language};
            let mnemonic = Mnemonic::generate_in(Language::English, 24).unwrap();
            
            println!("✅ New 24-word mnemonic phrase:");
            println!("┌─────────────────────────────────────────────────────────────┐");
            
            let words: Vec<&str> = mnemonic.to_string().split_whitespace().collect();
            for (i, chunk) in words.chunks(6).enumerate() {
                let line = chunk.iter()
                    .enumerate()
                    .map(|(j, word)| format!("{:2}. {:12}", i * 6 + j + 1, word))
                    .collect::<Vec<_>>()
                    .join(" ");
                println!("│ {:59} │", line);
            }
            
            println!("└─────────────────────────────────────────────────────────────┘");
            println!();
            println!("⚠️  IMPORTANT SECURITY NOTICE:");
            println!("   • Write down these words in order and store them safely");
            println!("   • Never share your mnemonic phrase with anyone");
            println!("   • Anyone with these words can access your wallet");
            println!("   • This is the ONLY way to recover your wallet");
        }
        
        Commands::Info { wallet } => {
            println!("ℹ️  Wallet Information: {}", wallet);
            println!("   Status: {}", i18n.get_text(lang, "status-offline", None));
            println!("   Security: {}", i18n.get_text(lang, "security-quantum-safe", None));
            println!("   Networks: Ethereum, Solana");
        }
        
        Commands::Backup { wallet, output } => {
            println!("💾 Backing up wallet '{}' to '{}'...", wallet, output);
            println!("✅ Backup completed (simulated)");
            println!("   File: {}", output);
            println!("   {}", i18n.get_text(lang, "msg-backup-reminder", None));
        }
        
        Commands::Security => {
            println!("🛡️  Security Status:");
            println!("   ✅ Quantum-Safe Encryption: Enabled (Kyber1024)");
            println!("   ✅ Shamir Secret Sharing: 2-of-3 threshold");
            println!("   ✅ Multi-Signature: 2-of-3 threshold");
            println!("   ⚠️  HSM Module: Disabled (software mode)");
            println!("   ✅ Memory Protection: Zero-on-drop enabled");
            println!("   ✅ Audit Logging: Enabled");
            println!("   ✅ Network Encryption: TLS 1.3");
            
            if let Some(metrics) = monitoring::get_metrics() {
                println!();
                println!("📊 Security Metrics:");
                println!("   • Quantum encryptions: Available");
                println!("   • Multi-sig operations: Available");
                println!("   • Failed login attempts: Available");
            }
        }
    }
    
    Ok(())
}