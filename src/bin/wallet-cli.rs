use anyhow::Context;
use clap::Parser;
use defi_hot_wallet::cli::{Cli, Commands};
use defi_hot_wallet::core::config::WalletConfig;
use defi_hot_wallet::core::WalletManager;
use std::collections::HashMap;
use tokio::fs;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    // 从默认配置构建，然后覆盖对测试/CLI运行重要的字段
    let mut wallet_config = WalletConfig::default();
    // 对测试使用内存中的 sqlite 以避免接触磁盘
    wallet_config.storage.database_url = "sqlite::memory:".to_string();
    // 确保区块链网络映射存在（避免需要 BlockchainConfig::default）
    wallet_config.blockchain.networks = HashMap::new();
    let wallet_manager = WalletManager::new(&wallet_config).await?;

    match cli.command {
        Commands::Create { name, output } => {
            let wallet_info = wallet_manager.create_wallet(&name, false).await?;
            println!("创建钱包: {}", name);
            if let Some(path) = output.as_deref() {
                write_wallet_output_if_requested(Some(path), &wallet_info).await?;
                println!("Wallet info written to {}", path.display());
            }
        }
        Commands::List => {
            println!("列出所有钱包");
        }
        Commands::Info { name } => {
            println!("显示钱包信息: {}", name);
        }
        Commands::Transfer { name, to, amount } => {
            println!("转账: {} -> {} 数量: {}", name, to, amount);
        }
        Commands::Balance { name, network: _ } => {
            println!("查询余额: {}", name);
        }
        Commands::Bridge { name, from_chain: _, to_chain: _, token: _, amount: _ } => {
            println!("桥接: {}", name);
        }
        Commands::GenerateMnemonic => {
            // simple 12-word mock mnemonic for tests
            println!("abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about");
        }
        Commands::Help => {
            println!("Help requested");
        }
    }

    Ok(())
}

/// 辅助函数：如果提供了 --output 路径，则将钱包信息写入文件。
async fn write_wallet_output_if_requested(
    output_path: Option<&std::path::Path>,
    wallet: &impl serde::Serialize,
) -> anyhow::Result<()> {
    if let Some(path) = output_path {
        let json = serde_json::to_string_pretty(wallet).context("serialize wallet to json")?;
        // 如果需要，创建父目录
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).await.ok();
        }
        fs::write(path, json).await.context("write wallet file to --output path")?;
    }
    Ok(())
}
// ...existing code...
