// src/main.rs
//! DeFi Hot Wallet Server Entry Point
//! This binary is responsible for starting the API server.
use anyhow::Result;
use clap::{Parser, Subcommand};
use defi_hot_wallet::api::server::WalletServer;
use defi_hot_wallet::core::config::{BlockchainConfig, StorageConfig, WalletConfig};
use std::collections::HashMap;
use tracing::info;
use tracing_subscriber::{EnvFilter, FmtSubscriber};

#[derive(Parser)]
#[command(name = "hot_wallet")]
#[command(about = "DeFi Hot Wallet Server")]
#[command(version = env!("CARGO_PKG_VERSION"))]
struct Args {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Start the wallet server
    Server {
        /// Host to bind the server to
        /// Port to bind the server to
        #[arg(long, default_value = "8080")]
        port: u16,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    // Initialize logging
    init_logging()?;

    info!("Starting DeFi Hot Wallet v{}", env!("CARGO_PKG_VERSION"));

    // Read DATABASE_URL from env or fallback to relative path in current dir
    let database_url =
        std::env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite://./wallets.db".to_string());

    // A default configuration.
    let wallet_config = WalletConfig {
        storage: StorageConfig {
            database_url: database_url.clone(),
            max_connections: Some(10),
            connection_timeout_seconds: Some(30),
        },
        blockchain: BlockchainConfig {
            networks: HashMap::new(), // WalletManager will populate this
            default_network: Some("eth".to_string()),
        },
        quantum_safe: false,
        multi_sig_threshold: 2,
    };

    // Read API_KEY from environment
    let api_key = std::env::var("API_KEY").ok();

    let server = WalletServer::new("127.0.0.1".to_string(), 8080, wallet_config, api_key).await?;

    match args.command {
        Some(Commands::Server { port }) => {
            info!("Starting server on port {}", port);
            let server_with_port = WalletServer { port, ..server };
            server_with_port.start().await?;
        }
        None => {
            // Default behavior: start the server on 127.0.0.1:8080
            info!("No command specified, starting server on default port 8080");
            server.start().await?;
        }
    }

    Ok(())
}

fn init_logging() -> Result<()> {
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info,hyper=info,h2=info"));

    let subscriber = FmtSubscriber::builder()
        .with_env_filter(filter)
        .with_max_level(tracing::Level::TRACE)
        .finish();

    tracing::subscriber::set_global_default(subscriber)?;
    Ok(())
}
