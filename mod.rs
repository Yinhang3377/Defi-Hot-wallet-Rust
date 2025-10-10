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
    Create {
        /// Wallet name
        #[arg(long)]
        pub name: String,
        /// Optional output path
        #[arg(long)]
        pub output: Option<PathBuf>,
    },
    Info {
        #[arg(long)]
        pub name: String,
    },
    Transfer {
        #[arg(long)]
        pub name: String,
        #[arg(long)]
        pub to: String,
        #[arg(long)]
        pub amount: String,
    },
    Balance {
        #[arg(long)]
        pub name: String,
        #[arg(long)]
        pub network: Option<String>,
    },
    Bridge {
        #[arg(long = "name")]
        pub name: String,
        #[arg(long = "from-chain")]
        pub from_chain: String,
        #[arg(long = "to-chain")]
        pub to_chain: String,
        #[arg(long)]
        pub token: String,
        #[arg(long)]
        pub amount: String,
    },
    List,
    GenerateMnemonic,
    Help,
}