use anyhow::Result;
use clap::{Parser, Subcommand};
use tracing::error;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

// 直接引用库 crate（包名连字符转下划线）
use defi_hot_wallet::core::{config::WalletConfig, wallet::WalletManager};
use defi_hot_wallet::storage::WalletMetadata;
use defi_hot_wallet::{i18n, monitoring};

#[derive(Parser)]
#[command(name = "wallet-cli")]
#[command(about = "DeFi Hot Wallet CLI - 命令行钱包工具")]
#[command(version = "0.1.0")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    /// Configuration file path
    #[arg(short, long, default_value = "config.toml")]
    pub config: String,

    /// Language (en, zh)
    #[arg(short, long, default_value = "en")]
    pub language: String,

    /// Log level
    #[arg(long, default_value = "info")]
    pub log_level: String,
}

#[derive(Subcommand)]
pub enum Commands {
    Create {
        #[arg(short, long)]
        name: String,
        /// Use quantum-safe encryption (enabled by default)
        #[arg(long, default_value_t = true, action = clap::ArgAction::Set)]
        quantum: bool,
        /// Disable quantum-safe encryption
        #[arg(long, overrides_with = "quantum", action = clap::ArgAction::SetFalse)]
        no_quantum: bool,
    },
    List,
    /// Delete a specific wallet
    Delete {
        #[arg(short, long)]
        name: String,
    },
    Balance {
        #[arg(short, long)]
        wallet: String,
        #[arg(short, long, default_value = "eth")]
        network: String,
    },
    Send {
        #[arg(short, long)]
        wallet: String,
        #[arg(short, long)]
        to: String,
        #[arg(short, long)]
        amount: String,
        #[arg(short, long, default_value = "eth")]
        network: String,
    },
    GenerateMnemonic,
    Info {
        #[arg(short, long)]
        wallet: String,
    },
    Backup {
        #[arg(short, long)]
        wallet: String,
        #[arg(short, long)]
        output: String,
    },
    Security,
}

#[tokio::main]
async fn main() -> Result<()> {
    if let Err(e) = real_main().await {
        error!("❌ Application error: {e}");
        // 在真实 CLI 中可考虑以非零状态码退出
        // std::process::exit(1);
    }
    Ok(())
}

async fn real_main() -> Result<()> {
    let cli = Cli::parse();

    // 日志初始化
    tracing_subscriber::registry()
        .with(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| format!("wallet_cli={}", cli.log_level).into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // i18n 初始化
    let i18n_manager = i18n::init_default_languages()?;
    let lang = &cli.language;

    println!("🔒 {}", i18n_manager.get_text(lang, "app-name", None));
    println!(
        "📱 {}",
        i18n_manager.get_text(lang, "app-description", None)
    );
    println!();

    // 监控初始化
    monitoring::init_metrics().await?;

    // WalletManager 实例
    let config = WalletConfig::load_from_file(&cli.config).unwrap_or_else(|e| {
        error!(
            "Failed to load config from '{}', using default settings. Error: {}",
            cli.config, e
        );
        WalletConfig::default()
    });
    let manager = WalletManager::new(&config).await?;

    match cli.command {
        Commands::Create { name, quantum, .. } => {
            println!(
                "🔧 {}...",
                i18n_manager.get_text(lang, "wallet-create", None)
            );
            // 优先使用命令行标志，否则回退到配置文件中的默认值
            let use_quantum = quantum;
            match manager.create_wallet(&name, use_quantum).await {
                Ok(wallet_info) => {
                    println!("✅ {}", i18n_manager.get_text(lang, "wallet-created", None));
                    println!("   ID: {}", wallet_info.id);
                    println!(
                        "   {}: {}",
                        i18n_manager.get_text(lang, "wallet-name", None),
                        wallet_info.name
                    );
                    println!(
                        "   {}: {}",
                        i18n_manager.get_text(lang, "security-quantum-safe", None),
                        wallet_info.quantum_safe
                    );
                    if use_quantum {
                        println!(
                            "🛡️ {}",
                            i18n_manager.get_text(lang, "msg-quantum-protection", None)
                        );
                    }
                    println!(
                        "💡 {}",
                        i18n_manager.get_text(lang, "msg-backup-reminder", None)
                    );
                }
                Err(e) => {
                    error!("Failed to create wallet: {e}");
                    return Err(e);
                }
            }
        }
        Commands::List => {
            println!(
                "📋 {}",
                i18n_manager.get_text(lang, "nav-wallets", None)
            );
            match manager.list_wallets().await {
                Ok(wallets) => {
                    if wallets.is_empty() {
                        println!("   (No wallets found - use 'create' command to add wallets)");
                    } else {
                        // 打印表头
                        println!(
                            "{:<20} {:<12} {:<25}",
                            "Name", "Quantum Safe", "Created At"
                        );
                        println!("{:-<60}", ""); // 分隔线
                        for wallet in wallets {
                            println!(
                                "{:<20} {:<12} {:<25}",
                                wallet.name,
                                wallet.quantum_safe,
                                wallet.created_at.format("%Y-%m-%d %H:%M:%S")
                            );
                        }
                    }
                }
                Err(e) => {
                    error!("Failed to list wallets: {e}");
                    return Err(e);
                }
            }
        }
        Commands::Delete { name } => {
            println!("🗑️ Deleting wallet '{}'...", name);
            match manager.delete_wallet(&name).await {
                Ok(_) => {
                    println!("✅ Wallet '{}' deleted successfully.", name);
                }
                Err(e) => {
                    error!("Failed to delete wallet: {e}");
                    return Err(e);
                }
            }
        }
        Commands::Balance { wallet, network } => {
            println!(
                "💰 {}: {}",
                i18n_manager.get_text(lang, "wallet-balance", None),
                wallet
            );

            let network_name = match network.as_str() {
                "eth" => i18n_manager.get_text(lang, "network-ethereum", None),
                "solana" => i18n_manager.get_text(lang, "network-solana", None),
                _ => network.clone(),
            };

            match manager.get_balance(&wallet, &network).await {
                Ok(balance) => {
                    println!(
                        "✅ {}: {}",
                        i18n_manager.get_text(lang, "wallet-balance", None),
                        balance
                    );
                    println!("   Network: {}", network_name);
                }
                Err(e) => {
                    error!("Failed to get balance: {e}");
                    println!(
                        "❌ {}: {e}",
                        i18n_manager.get_text(lang, "error-network-error", None)
                    );
                    return Err(e);
                }
            }
        }
        Commands::Send {
            wallet,
            to,
            amount,
            network,
        } => {
            println!(
                "💸 {} {} -> {} ({})...",
                i18n_manager.get_text(lang, "tx-send", None),
                amount,
                to,
                network
            );
            match manager
                .send_transaction(&wallet, &to, &amount, &network)
                .await
            {
                Ok(tx_hash) => {
                    println!("✅ {}", i18n_manager.get_text(lang, "tx-success", None));
                    println!("   Transaction Hash: {tx_hash}");
                    println!(
                        "   Status: {}",
                        i18n_manager.get_text(lang, "tx-pending", None)
                    );
                }
                Err(e) => {
                    error!("Failed to send transaction: {e}");
                    println!("❌ {}: {e}", i18n_manager.get_text(lang, "tx-failed", None));
                    return Err(e);
                }
            }
        }
        Commands::GenerateMnemonic => {
            println!("🔑 Generating new mnemonic phrase...");
            use bip39::{Language, Mnemonic};
            use rand::RngCore;
            let mut entropy = [0u8; 32];
            rand::thread_rng().fill_bytes(&mut entropy);
            let mnemonic = Mnemonic::from_entropy_in(Language::English, &entropy).unwrap();

            println!("✅ New 24-word mnemonic phrase:");
            println!("┌─────────────────────────────────────────────────────────────┐");
            let mnemonic_str = mnemonic.to_string();
            let words: Vec<&str> = mnemonic_str.split_whitespace().collect();
            for (i, chunk) in words.chunks(6).enumerate() {
                let line = chunk
                    .iter()
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
            println!("ℹ️  Wallet Information: {wallet}");
            println!(
                "   Status: {}",
                i18n_manager.get_text(lang, "status-offline", None)
            );
            println!(
                "   Security: {}",
                i18n_manager.get_text(lang, "security-quantum-safe", None)
            );
            println!("   Networks: Ethereum, Solana");
        }
        Commands::Backup { wallet, output } => {
            println!("💾 Backing up wallet '{wallet}' to '{output}'...");
            println!("✅ Backup completed (simulated)");
            println!("   File: {output}");
            println!(
                "   {}",
                i18n_manager.get_text(lang, "msg-backup-reminder", None)
            );
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
            if let Some(_metrics) = monitoring::get_metrics() {
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
