// src/cli/mod.rs
use clap::{Parser, Subcommand};
use std::path::PathBuf;

/// DeFi Hot Wallet CLI (library-facing definitions)
#[derive(Debug, Parser)]
#[command(name = "wallet-cli", about = "DeFi Hot Wallet CLI")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// 创建钱包
    Create {
        /// 钱包名称
        #[arg(long)]
        name: String,
        /// 输出文件路径
        #[arg(long)]
        output: Option<PathBuf>,
    },
    /// 显示钱包信息
    Info {
        #[arg(long)]
        name: String,
    },
    /// 转账
    Transfer {
        #[arg(long)]
        name: String,
        #[arg(long)]
        to: String,
        #[arg(long)]
        amount: String,
    },
    /// 查询余额
    Balance {
        #[arg(long)]
        name: String,
        /// 网络名称 (e.g., eth, solana)
        #[arg(long)]
        network: String,
    },
    /// 跨链桥转账
    Bridge {
        #[arg(long = "name")]
        name: String,
        #[arg(long = "from-chain")]
        from_chain: String,
        #[arg(long = "to-chain")]
        to_chain: String,
        #[arg(long)]
        token: String,
        #[arg(long)]
        amount: String,
    },
    /// 列出所有钱包
    List,
    /// 生成助记词（示例）
    GenerateMnemonic,
}
