<<<<<<< HEAD
#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(unused_mut)]
#![allow(unused_assignments)]
#![allow(clippy::upper_case_acronyms)]
use anyhow::{Ok, Result};
use clap::{Parser, Subcommand};
use tracing::{info, warn};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod api;
mod blockchain;
mod core;
mod crypto;
mod i18n;
mod monitoring;
mod storage;

use crate::api::server::WalletServer;
use crate::core::config::WalletConfig;
use crate::core::wallet::WalletInfo;
use crate::core::wallet::WalletManager;

#[derive(Parser)]
#[command(name = "defi-wallet")]
#[command(about = "DeFi级热钱包，Rust打造，安全如堡垒！")]
#[command(version = "0.1.0")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    /// Configuration file path
    #[arg(short, long, default_value = "config.toml")]
    pub config: String,

    /// Log level
    #[arg(short, long, default_value = "info")]
    pub log_level: String,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Start the wallet server
    Server {
        /// Server port
        #[arg(short, long)]
        port: Option<u16>,

        /// Server host
        #[arg(short = 'H', long, default_value = "127.0.0.1")]
        host: Option<String>,
    },
    /// Initialize a new wallet
    Init {
        /// Wallet name
        #[arg(short, long)]
        name: String,

        /// Use quantum-safe encryption (enabled by default)
        #[arg(long, default_value_t = true, action = clap::ArgAction::Set)]
        quantum_safe: bool,
        /// Disable quantum-safe encryption
        #[arg(long, overrides_with = "quantum_safe", action = clap::ArgAction::SetFalse)]
        no_quantum_safe: bool,
    },
    /// Show wallet balance
    Balance {
        /// Wallet name
        #[arg(short, long)]
        wallet: String,

        /// Blockchain network (eth, solana)
        #[arg(short, long)]
        network: String,
    },
    /// Send transaction
    Send {
        /// Wallet name
        #[arg(short, long)]
        wallet: String,

        /// Recipient address
        #[arg(short, long)]
        to: String,

        /// Amount to send
        #[arg(short, long)]
        amount: String,

        /// Blockchain network (eth, solana)
        #[arg(short, long)]
        network: String,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Initialize logging
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| format!("defi_wallet={}", cli.log_level).into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    info!("🔒 Starting DeFi Hot Wallet - Rust Edition");
    info!("Version: {}", env!("CARGO_PKG_VERSION"));

    // Initialize monitoring
    monitoring::init_metrics().await?;

    // Load configuration
    let config = WalletConfig::load_from_file(&cli.config).unwrap_or_else(|e| {
        warn!(
            "Failed to load config from '{}', using default settings. Error: {}",
            cli.config, e
        );
        WalletConfig::default()
    });

    match cli.command {
        Commands::Server { port, host } => {
            // 优先使用命令行参数，否则从配置文件中读取
            let server_host = host.unwrap_or(config.server.host.clone());
            let server_port = port.unwrap_or(config.server.port);
            info!(
                "🚀 Starting wallet server on {}:{}",
                server_host, server_port
            );
            let server = WalletServer::new(server_host, server_port, config).await?;
            server.start().await?;
        }
        Commands::Init { name, quantum_safe, .. } => {
            info!("🔧 Initializing new wallet: {}", name);
            let manager = WalletManager::new(&config).await?;
            // 优先使用命令行标志，否则回退到配置文件中的默认值
            let use_quantum = quantum_safe;
            manager.create_wallet(&name, use_quantum).await?;
            info!("✅ Wallet '{}' created successfully", name);
        }
        Commands::Balance { wallet, network } => {
            info!("💰 Checking balance for wallet: {} on {}", wallet, network);
            let manager = WalletManager::new(&config).await?;
            let balance = manager.get_balance(&wallet, &network).await?;
            println!("Balance: {}", balance);
        }
        Commands::Send {
            wallet,
            to,
            amount,
            network,
        } => {
            info!(
                "💸 Sending {} from {} to {} on {}",
                amount, wallet, to, network
            );
            let manager = WalletManager::new(&config).await?;
            let tx_hash = manager
                .send_transaction(&wallet, &to, &amount, &network)
                .await?;
            info!("✅ Transaction sent: {}", tx_hash);
            println!("Transaction hash: {}", tx_hash);
=======
/// 主入口：集成配置、安全、错误等模块，实现 wallet create 命令生成加密账户
use clap::{Parser, Subcommand};
use env_logger::init;
use hex::encode;
use hot_wallet::config::WalletConfig; // 钱包配置加载
use hot_wallet::security::encryption::WalletSecurity; // 加密/解密操作
use hot_wallet::security::memory_protection::SensitiveData;
use rand::Rng;
use secp256k1::{PublicKey, Secp256k1, SecretKey};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs::OpenOptions;
use std::io::{self, Write};
#[cfg(unix)]
use std::os::unix::fs::OpenOptionsExt;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(author, version, about = "A secure, multi-chain hot wallet framework", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Create a new encrypted wallet keypair
    Create {
        /// Optional associated data to bind to the encryption
        #[arg(long)]
        aad: Option<String>,

        /// Path to save the wallet file
        #[arg(short, long)]
        output: PathBuf,
    },
}

#[derive(Serialize, Deserialize)]
struct WalletFile {
    public_key: String,
    encrypted_private_key: String,
    network: String,
    aad: String,
}

impl Cli {
    pub fn prompt_password() -> Result<String, io::Error> {
        print!("请输入加密密钥: ");
        io::stdout().flush()?;
        let mut password = String::new();
        io::stdin().read_line(&mut password)?;
        Ok(password.trim().to_string())
    }
}

/// 程序主入口
fn main() -> Result<(), Box<dyn Error>> {
    // 初始化日志记录器，以便在加密等模块中打印错误日志
    init();

    // 1. 使用 clap 解析命令行参数
    let cli = Cli::parse();

    // 2. 加载环境变量配置
    let config = match WalletConfig::from_env() {
        Ok(cfg) => cfg,
        Err(e) => {
            eprintln!("[配置错误] {}", e);
            return Err(Box::from(e));
        }
    };
    println!("[配置] network: {}", config.network);

    // 3. 根据解析的命令执行相应操作
    match &cli.command {
        Commands::Create { aad, output } => {
            println!("正在创建新的加密钱包...");

            // 提示用户输入加密密钥
            let encryption_key = Cli::prompt_password()?;

            // 生成 secp256k1 密钥对（手动随机 32 字节，避免依赖 crate 的 rand feature）
            let secp = Secp256k1::new();
            // rand 0.9: thread_rng() 已弃用，使用 rng()
            let mut rng = rand::rng();
            let mut sk_bytes = [0u8; 32];
            rng.fill(&mut sk_bytes);
            // secp256k1 0.31: from_slice 已弃用，改用 from_byte_array
            let secret_key = SecretKey::from_byte_array(sk_bytes).expect("32-byte secret key");
            let public_key = PublicKey::from_secret_key(&secp, &secret_key);
            println!("[生成] 公钥: {}", public_key);

            // 用 SensitiveData 包裹私钥并锁定内存
            let sensitive_sk = SensitiveData::new(secret_key.secret_bytes());

            // 准备关联数据 (AAD)
            let aad_bytes = aad.as_deref().unwrap_or("").as_bytes();
            println!("[加密] 使用关联数据 (AAD): '{}'", aad.as_deref().unwrap_or("<无>"));

            // 用用户输入的加密密钥加密私钥
            let encrypted = WalletSecurity::encrypt_private_key(
                &sensitive_sk.data,
                &encryption_key,
                aad_bytes,
            )?;

            println!("[加密] 加密私钥(hex): {}", encode(&encrypted));

            // 创建并保存钱包文件
            let wallet_file = WalletFile {
                public_key: public_key.to_string(),
                encrypted_private_key: encode(&encrypted),
                network: config.network.clone(),
                aad: aad.as_deref().unwrap_or("").to_string(),
            };

            let wallet_json = serde_json::to_string_pretty(&wallet_file)?;

            let mut open_options = OpenOptions::new();
            open_options.write(true).create_new(true); // 防止覆盖已存在的文件

            // 在 Unix 系统上，设置文件权限为 600 (仅所有者可读写)
            #[cfg(unix)]
            open_options.mode(0o600);

            open_options.open(output)?.write_all(wallet_json.as_bytes())?;

            println!("✅ 钱包已成功创建并保存至: {}", output.display());
>>>>>>> be35db3d094cb6edd3c63585f33fdcb299a57158
        }
    }

    Ok(())
}
