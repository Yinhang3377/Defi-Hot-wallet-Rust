// ...existing code...
use anyhow::Result;
use serde::Deserialize;
use std::env;

#[derive(Debug, Clone, Deserialize)]
pub struct AppEnvConfig {
    /// Database URL (uses DATABASE_URL env or falls back to sqlite file)
    pub database_url: String,
    /// Optional Ethereum RPC URL (WALLET_ETHEREUM_RPC_URL)
    pub ethereum_rpc_url: Option<String>,
    /// Optional additional config fields used by the app
    pub some_field: Option<String>,
    pub another_field: Option<String>,
}

impl AppEnvConfig {
    pub fn from_env() -> Result<Self> {
        let database_url =
            env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite://./wallets.db".to_string());
        let ethereum_rpc_url = env::var("WALLET_ETHEREUM_RPC_URL").ok();
        let some_field = env::var("APP_SOME_FIELD").ok();
        let another_field = env::var("APP_ANOTHER_FIELD").ok();

        Ok(AppEnvConfig { database_url, ethereum_rpc_url, some_field, another_field })
    }
}
// ...existing code...
