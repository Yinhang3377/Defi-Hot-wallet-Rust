use anyhow::Result;
use async_trait::async_trait;
use chrono::{DateTime /* NaiveDate */};
use sqlx::types::chrono::Utc;
use sqlx::{sqlite::SqlitePool, types::chrono::NaiveDateTime, Row};
use tracing::{debug, info, warn};

use crate::blockchain::bridge::{BridgeTransaction, BridgeTransactionStatus};

#[derive(Debug)]
pub struct WalletStorage {
    pool: SqlitePool,
}

impl WalletStorage {
    pub async fn new() -> Result<Self> {
        // 使用标准前缀为 data 目录，确保文件可创建
        Self::new_with_url("sqlite://./data/wallet.db?mode=rwc").await
    }

    pub async fn new_with_url(database_url: &str) -> Result<Self> {
        info!("🔧 Initializing wallet storage: {}", database_url);

        // 1) 规范化 sqlite URL: sqlite: -> sqlite://
        let mut db_url = database_url.to_string();
        if db_url.starts_with("sqlite:") && !db_url.starts_with("sqlite://") {
            db_url = db_url.replacen("sqlite:", "sqlite://", 1);
        }

        // 2) 为基于文件的 sqlite 创建父目录
        if let Some(path) = db_url.strip_prefix("sqlite://") {
            let mut path_only = path.split('?').next().unwrap_or(path).to_string();

            // On Windows, urls like sqlite:///C:/path will produce a leading '/' before drive letter.
            // Normalize by removing the leading slash when present (e.g. "/C:/..." -> "C:/...")
            #[cfg(windows)]
            {
                if path_only.starts_with('/') && path_only.len() > 2 {
                    let bytes = path_only.as_bytes();
                    if bytes[2] == b':' {
                        // strip leading '/'
                        path_only = path_only[1..].to_string();
                    }
                }
            }

            if path_only != ":memory:" && !path_only.is_empty() {
                if let Some(parent) = std::path::Path::new(&path_only).parent() {
                    if !parent.as_os_str().is_empty() {
                        // 忽略已存在等非致命错误
                        if let Err(e) = std::fs::create_dir_all(parent) {
                            warn!("Failed to create database dir {:?}: {}", parent, e);
                        }
                    }
                }

                // Rebuild db_url to the normalized form so SqlitePool can open it correctly
                // Preserve query params if any
                if let Some(query) = path.splitn(2, '?').nth(1) {
                    if cfg!(windows)
                        && path_only.len() > 1
                        && path_only.as_bytes().get(1) == Some(&b':')
                    {
                        // Windows absolute path: sqlite:///C:/path
                        db_url = format!("sqlite:///{}?{}", path_only, query);
                    } else {
                        db_url = format!("sqlite://{}?{}", path_only, query);
                    }
                } else {
                    if cfg!(windows)
                        && path_only.len() > 1
                        && path_only.as_bytes().get(1) == Some(&b':')
                    {
                        db_url = format!("sqlite:///{}", path_only);
                    } else {
                        db_url = format!("sqlite://{}", path_only);
                    }
                }
            }
        }

        // 3) 连接使用规范化后的 db_url
        eprintln!("[storage] connecting to db_url={}", db_url);
        let pool = SqlitePool::connect(&db_url)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to connect to database: {}", e))?;

        let storage = Self { pool };
        storage.initialize_schema().await?;

        info!("✅ Wallet storage initialized");
        Ok(storage)
    }

    async fn initialize_schema(&self) -> Result<()> {
        debug!("Initializing database schema");

        // Wallets table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS wallets (
                id TEXT PRIMARY KEY,
                name TEXT UNIQUE NOT NULL,
                encrypted_data BLOB NOT NULL,
                quantum_safe BOOLEAN NOT NULL,
                created_at DATETIME NOT NULL,
                updated_at DATETIME NOT NULL
            )
            "#,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to create wallets table: {}", e))?;

        // Transactions table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS transactions (
                id TEXT PRIMARY KEY,
                wallet_id TEXT NOT NULL,
                tx_hash TEXT NOT NULL,
                network TEXT NOT NULL,
                from_address TEXT NOT NULL,
                to_address TEXT NOT NULL,
                amount TEXT NOT NULL,
                fee TEXT NOT NULL,
                status TEXT NOT NULL,
                created_at DATETIME NOT NULL,
                confirmed_at DATETIME,
                FOREIGN KEY (wallet_id) REFERENCES wallets (id)
            )
            "#,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to create transactions table: {}", e))?;

        // Audit logs table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS audit_logs (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                wallet_id TEXT,
                action TEXT NOT NULL,
                details TEXT,
                ip_address TEXT,
                user_agent TEXT,
                created_at DATETIME NOT NULL
            )
            "#,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to create audit_logs table: {}", e))?;

        // Bridge Transactions table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS bridge_transactions (
                id TEXT PRIMARY KEY,
                from_wallet TEXT NOT NULL,
                from_chain TEXT NOT NULL,
                to_chain TEXT NOT NULL,
                token TEXT NOT NULL,
                amount TEXT NOT NULL,
                status TEXT NOT NULL,
                source_tx_hash TEXT,
                destination_tx_hash TEXT,
                created_at DATETIME NOT NULL,
                updated_at DATETIME NOT NULL,
                fee_amount TEXT,
                estimated_completion_time DATETIME
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Create indexes
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_wallets_name ON wallets (name)")
            .execute(&self.pool)
            .await?;

        sqlx::query(
            "CREATE INDEX IF NOT EXISTS idx_transactions_wallet_id ON transactions (wallet_id)",
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            "CREATE INDEX IF NOT EXISTS idx_transactions_tx_hash ON transactions (tx_hash)",
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            "CREATE INDEX IF NOT EXISTS idx_audit_logs_wallet_id ON audit_logs (wallet_id)",
        )
        .execute(&self.pool)
        .await?;

        debug!("鉁?Database schema initialized");
        Ok(())
    }

    pub async fn store_wallet(
        &self,
        name: &str,
        encrypted_data: &[u8],
        quantum_safe: bool,
    ) -> Result<()> {
        debug!("Storing wallet: {}", name);

        let wallet_id = uuid::Uuid::new_v4().to_string();
        let now = Utc::now().naive_utc();

        sqlx::query(
            r#"
            INSERT INTO wallets (id, name, encrypted_data, quantum_safe, created_at, updated_at)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6)
            "#,
        )
        .bind(&wallet_id)
        .bind(name)
        .bind(encrypted_data)
        .bind(quantum_safe)
        .bind(now)
        .bind(now)
        .execute(&self.pool)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to store wallet: {}", e))?;

        // Log the action
        self.log_action(
            &wallet_id,
            "wallet_created",
            &format!("Wallet '{}' created", name),
            None,
            None,
        )
        .await?;

        debug!("✅ Database schema initialized");
        Ok(())
    }

    pub async fn load_wallet(&self, name: &str) -> Result<(Vec<u8>, bool)> {
        debug!("Loading wallet: {}", name);

        let row =
            sqlx::query("SELECT id, encrypted_data, quantum_safe FROM wallets WHERE name = ?1")
                .bind(name)
                .fetch_optional(&self.pool)
                .await
                .map_err(|e| anyhow::anyhow!("Failed to load wallet: {}", e))?;

        match row {
            Some(row) => {
                let wallet_id: String = row.get("id");
                let encrypted_data: Vec<u8> = row.get("encrypted_data");
                let quantum_safe: bool = row.get("quantum_safe");

                // Log the action
                self.log_action(
                    &wallet_id,
                    "wallet_accessed",
                    &format!("Wallet '{}' accessed", name),
                    None,
                    None,
                )
                .await?;

                debug!("✅ Wallet loaded: {}", name);
                Ok((encrypted_data, quantum_safe))
            }
            None => Err(anyhow::anyhow!("Wallet not found: {}", name)),
        }
    }

    pub async fn list_wallets(&self) -> Result<Vec<WalletMetadata>> {
        debug!("Listing all wallets");

        let rows = sqlx
            ::query(
                "SELECT id, name, quantum_safe, created_at, updated_at FROM wallets ORDER BY created_at DESC"
            )
            .fetch_all(&self.pool).await
            .map_err(|e| anyhow::anyhow!("Failed to list wallets: {}", e))?;

        let wallets: Vec<WalletMetadata> = rows
            .into_iter()
            .map(|row| WalletMetadata {
                id: row.get("id"),
                name: row.get("name"),
                quantum_safe: row.get("quantum_safe"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            })
            .collect();

        debug!("✅ Listed {} wallets", wallets.len());
        Ok(wallets)
    }

    pub async fn delete_wallet(&self, name: &str) -> Result<()> {
        debug!("Deleting wallet: {}", name);

        // Get wallet ID first
        let row = sqlx::query("SELECT id FROM wallets WHERE name = ?1")
            .bind(name)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to find wallet: {}", e))?;

        let wallet_id = match row {
            Some(row) => row.get::<String, _>("id"),
            None => {
                return Err(anyhow::anyhow!("Wallet not found: {}", name));
            }
        };

        // Delete wallet
        let result = sqlx::query("DELETE FROM wallets WHERE name = ?1")
            .bind(name)
            .execute(&self.pool)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to delete wallet: {}", e))?;

        if result.rows_affected() == 0 {
            return Err(anyhow::anyhow!("Wallet not found: {}", name));
        }

        // Log the action
        self.log_action(
            &wallet_id,
            "wallet_deleted",
            &format!("Wallet '{}' deleted", name),
            None,
            None,
        )
        .await?;

        warn!("🗑️ Wallet deleted: {}", name);
        Ok(())
    }

    pub async fn store_transaction(&self, tx_data: &TransactionRecord) -> Result<()> {
        debug!("Storing transaction: {}", tx_data.tx_hash);

        sqlx
            ::query(
                r#"
            INSERT INTO transactions (id, wallet_id, tx_hash, network, from_address, to_address, amount, fee, status, created_at, confirmed_at)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)
            "#
            )
            .bind(&tx_data.id)
            .bind(&tx_data.wallet_id)
            .bind(&tx_data.tx_hash)
            .bind(&tx_data.network)
            .bind(&tx_data.from_address)
            .bind(&tx_data.to_address)
            .bind(&tx_data.amount)
            .bind(&tx_data.fee)
            .bind(&tx_data.status)
            .bind(tx_data.created_at)
            .bind(tx_data.confirmed_at)
            .execute(&self.pool).await
            .map_err(|e| anyhow::anyhow!("Failed to store transaction: {}", e))?;

        debug!("鉁?Transaction stored: {}", tx_data.tx_hash);
        Ok(())
    }

    pub async fn get_wallet_transactions(&self, wallet_id: &str) -> Result<Vec<TransactionRecord>> {
        debug!("Getting transactions for wallet: {}", wallet_id);

        let rows = sqlx
            ::query(
                r#"
            SELECT id, wallet_id, tx_hash, network, from_address, to_address, amount, fee, status, created_at, confirmed_at
            FROM transactions 
            WHERE wallet_id = ?1 
            ORDER BY created_at DESC
            "#
            )
            .bind(wallet_id)
            .fetch_all(&self.pool).await
            .map_err(|e| anyhow::anyhow!("Failed to get transactions: {}", e))?;

        let transactions: Vec<TransactionRecord> = rows
            .into_iter()
            .map(|row| TransactionRecord {
                id: row.get("id"),
                wallet_id: row.get("wallet_id"),
                tx_hash: row.get("tx_hash"),
                network: row.get("network"),
                from_address: row.get("from_address"),
                to_address: row.get("to_address"),
                amount: row.get("amount"),
                fee: row.get("fee"),
                status: row.get("status"),
                created_at: row.get("created_at"),
                confirmed_at: row.get("confirmed_at"),
            })
            .collect();

        debug!("✅ Retrieved {} transactions", transactions.len());
        Ok(transactions)
    }

    pub async fn log_action(
        &self,
        wallet_id: &str,
        action: &str,
        details: &str,
        ip_address: Option<&str>,
        user_agent: Option<&str>,
    ) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO audit_logs (wallet_id, action, details, ip_address, user_agent, created_at)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6)
            "#,
        )
        .bind(wallet_id)
        .bind(action)
        .bind(details)
        .bind(ip_address)
        .bind(user_agent)
        .bind(Utc::now().naive_utc())
        .execute(&self.pool)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to log action: {}", e))?;

        Ok(())
    }

    pub async fn get_audit_logs(&self, wallet_id: Option<&str>) -> Result<Vec<AuditLog>> {
        let (query, params): (&str, Vec<&str>) = match wallet_id {
            Some(id) => {
                ("SELECT * FROM audit_logs WHERE wallet_id = ?1 ORDER BY created_at DESC", vec![id])
            }
            None => ("SELECT * FROM audit_logs ORDER BY created_at DESC", vec![]),
        };

        let mut query_builder = sqlx::query(query);
        for param in params {
            query_builder = query_builder.bind(param);
        }

        let rows = query_builder
            .fetch_all(&self.pool)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to get audit logs: {}", e))?;

        let logs = rows
            .into_iter()
            .map(|row| AuditLog {
                id: row.get("id"),
                wallet_id: row.get("wallet_id"),
                action: row.get("action"),
                details: row.get("details"),
                ip_address: row.get("ip_address"),
                user_agent: row.get("user_agent"),
                created_at: row.get("created_at"),
            })
            .collect();

        Ok(logs)
    }
}

// Bridge Transaction Storage
impl WalletStorage {
    pub async fn store_bridge_transaction(&self, tx: &BridgeTransaction) -> Result<()> {
        let status_str = serde_json::to_string(&tx.status)?;
        sqlx::query(
            r#"
            INSERT INTO bridge_transactions (id, from_wallet, from_chain, to_chain, token, amount, status, source_tx_hash, destination_tx_hash, created_at, updated_at, fee_amount, estimated_completion_time)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)
            "#,
        )
        .bind(&tx.id)
        .bind(&tx.from_wallet)
        .bind(&tx.from_chain)
        .bind(&tx.to_chain)
        .bind(&tx.token)
        .bind(&tx.amount)
        .bind(status_str)
        .bind(&tx.source_tx_hash)
        .bind(&tx.destination_tx_hash)
        .bind(tx.created_at)
        .bind(tx.updated_at)
        .bind(&tx.fee_amount)
        .bind(tx.estimated_completion_time)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn get_bridge_transaction(&self, id: &str) -> Result<BridgeTransaction> {
        let row = sqlx::query("SELECT * FROM bridge_transactions WHERE id = ?1")
            .bind(id)
            .fetch_one(&self.pool)
            .await?;

        let status_str: String = row.get("status");
        let status: BridgeTransactionStatus = serde_json::from_str(&status_str)?;

        let tx = BridgeTransaction {
            id: row.get("id"),
            from_wallet: row.get("from_wallet"),
            from_chain: row.get("from_chain"),
            to_chain: row.get("to_chain"),
            token: row.get("token"),
            amount: row.get("amount"),
            status,
            source_tx_hash: row.get("source_tx_hash"),
            destination_tx_hash: row.get("destination_tx_hash"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
            fee_amount: row.get("fee_amount"),
            estimated_completion_time: row.get("estimated_completion_time"),
        };
        Ok(tx)
    }

    pub async fn update_bridge_transaction_status(
        &self,
        id: &str,
        status: BridgeTransactionStatus,
        source_tx_hash: Option<String>,
    ) -> Result<()> {
        let status_str = serde_json::to_string(&status)?;
        let now = Utc::now();
        sqlx::query("UPDATE bridge_transactions SET status = ?1, updated_at = ?2, source_tx_hash = COALESCE(?3, source_tx_hash) WHERE id = ?4")
            .bind(status_str)
            .bind(now)
            .bind(source_tx_hash)
            .bind(id)
            .execute(&self.pool).await?;
        Ok(())
    }
}

impl Clone for WalletStorage {
    fn clone(&self) -> Self {
        // Clone 只克隆连接池，而不是创建新的
        Self { pool: self.pool.clone() }
    }
}

#[derive(Debug, Clone)]
pub struct WalletMetadata {
    pub id: String,
    pub name: String,
    pub quantum_safe: bool,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Clone)]
pub struct TransactionRecord {
    pub id: String,
    pub wallet_id: String,
    pub tx_hash: String,
    pub network: String,
    pub from_address: String,
    pub to_address: String,
    pub amount: String,
    pub fee: String,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub confirmed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone)]
pub struct AuditLog {
    pub id: i64,
    pub wallet_id: Option<String>,
    pub action: String,
    pub details: Option<String>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[async_trait]
pub trait WalletStorageTrait {
    async fn store_wallet(&self, name: &str, data: &[u8], quantum_safe: bool) -> Result<()>;
    async fn load_wallet(&self, name: &str) -> Result<(Vec<u8>, bool)>;
    async fn list_wallets(&self) -> Result<Vec<WalletMetadata>>;
    async fn delete_wallet(&self, name: &str) -> Result<()>;
    async fn store_bridge_transaction(&self, tx: &BridgeTransaction) -> Result<()>;
    async fn get_bridge_transaction(&self, id: &str) -> Result<BridgeTransaction>;
    async fn update_bridge_transaction_status(
        &self,
        id: &str,
        status: BridgeTransactionStatus,
        source_tx_hash: Option<String>,
    ) -> Result<()>;
}

// 让 WalletStorage 实现这个 trait
#[async_trait]
impl WalletStorageTrait for WalletStorage {
    async fn store_wallet(&self, name: &str, data: &[u8], quantum_safe: bool) -> Result<()> {
        // The existing implementation is already correct, we just call it.
        self.store_wallet(name, data, quantum_safe).await
    }

    async fn load_wallet(&self, name: &str) -> Result<(Vec<u8>, bool)> {
        self.load_wallet(name).await
    }

    async fn list_wallets(&self) -> Result<Vec<WalletMetadata>> {
        self.list_wallets().await
    }

    async fn delete_wallet(&self, name: &str) -> Result<()> {
        self.delete_wallet(name).await
    }

    async fn store_bridge_transaction(&self, tx: &BridgeTransaction) -> Result<()> {
        self.store_bridge_transaction(tx).await
    }

    async fn get_bridge_transaction(&self, id: &str) -> Result<BridgeTransaction> {
        self.get_bridge_transaction(id).await
    }

    async fn update_bridge_transaction_status(
        &self,
        id: &str,
        status: BridgeTransactionStatus,
        source_tx_hash: Option<String>,
    ) -> Result<()> {
        self.update_bridge_transaction_status(id, status, source_tx_hash).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[tokio::test]
    async fn test_wallet_storage_operations() {
        // 使用内存数据库进行测试，以避免文件残留并确保测试隔离
        let storage = WalletStorage::new_with_url("sqlite::memory:").await.unwrap();

        // Test store wallet
        let wallet_data = b"test wallet data";
        storage.store_wallet("test-wallet", wallet_data, false).await.unwrap();

        // Test load wallet
        let (loaded_data, quantum_safe) = storage.load_wallet("test-wallet").await.unwrap();
        assert_eq!(loaded_data, wallet_data);
        assert_eq!(quantum_safe, false);

        // Test list wallets
        let wallets = storage.list_wallets().await.unwrap();
        assert!(wallets.len() >= 1);
        assert!(wallets.iter().any(|w| w.name == "test-wallet"));

        // Test delete wallet
        storage.delete_wallet("test-wallet").await.unwrap();

        // Verify deletion
        let result = storage.load_wallet("test-wallet").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_bridge_transaction_storage() {
        let storage = WalletStorage::new_with_url("sqlite::memory:").await.unwrap();

        let tx = BridgeTransaction {
            id: "test-tx-123".to_string(),
            from_wallet: "wallet1".to_string(),
            from_chain: "eth".to_string(),
            to_chain: "solana".to_string(),
            token: "USDC".to_string(),
            amount: "100.0".to_string(),
            status: BridgeTransactionStatus::Initiated, // 使用 Initiated 替换 Pending
            source_tx_hash: None,
            destination_tx_hash: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            fee_amount: Some("1.0".to_string()),
            estimated_completion_time: Some(Utc::now() + chrono::Duration::hours(1)),
        };

        // Store transaction
        storage.store_bridge_transaction(&tx).await.unwrap();

        // Retrieve transaction
        let retrieved = storage.get_bridge_transaction("test-tx-123").await.unwrap();
        assert_eq!(retrieved.id, tx.id);
        assert_eq!(retrieved.status, BridgeTransactionStatus::Initiated);

        // Update status
        storage
            .update_bridge_transaction_status(
                "test-tx-123",
                BridgeTransactionStatus::Completed,
                Some("0x123".to_string()),
            )
            .await
            .unwrap();

        let updated = storage.get_bridge_transaction("test-tx-123").await.unwrap();
        assert_eq!(updated.status, BridgeTransactionStatus::Completed);
        assert_eq!(updated.source_tx_hash, Some("0x123".to_string()));
    }
}
