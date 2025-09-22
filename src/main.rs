#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(unused_mut)]
#![allow(unused_assignments)]
#![allow(clippy::upper_case_acronyms)]
use anyhow::Result;
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

        /// Use quantum-safe encryption
        #[arg(short, long, default_value = "true")]
        quantum_safe: bool,
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
            info!("🚀 Starting wallet server on {}:{}", server_host, server_port);
            let server = WalletServer::new(server_host, server_port, config).await?;
            server.start().await?;
        }
        Commands::Init { name, quantum_safe } => {
            info!("🔧 Initializing new wallet: {}", name);
            let manager = WalletManager::new(&config).await?;
            manager.create_wallet(&name, quantum_safe).await?;
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
        }
    }

    Ok(())
}
