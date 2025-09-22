use anyhow::Result;
use sqlx::{sqlite::SqlitePool, Row};
use tracing::{debug, info, warn};

#[derive(Debug)]
pub struct WalletStorage {
    pool: SqlitePool,
}

impl WalletStorage {
    pub async fn new() -> Result<Self> {
        // 使用标准前缀与 data 目录，确保文件可创建
        Self::new_with_url("sqlite://./data/wallet.db?mode=rwc").await
    }

    pub async fn new_with_url(database_url: &str) -> Result<Self> {
        info!("🗄️ Initializing wallet storage: {}", database_url);

        // 1) 规范化 sqlite URL: sqlite: -> sqlite://
        let mut db_url = database_url.to_string();
        if db_url.starts_with("sqlite:") && !db_url.starts_with("sqlite://") {
            db_url = db_url.replacen("sqlite:", "sqlite://", 1);
        }

        // 2) 为基于文件的 sqlite 创建父目录
        if let Some(path) = db_url.strip_prefix("sqlite://") {
            let path_only = path.split('?').next().unwrap_or(path);
            if path_only != ":memory:" && !path_only.is_empty() {
                if let Some(parent) = std::path::Path::new(path_only).parent() {
                    if !parent.as_os_str().is_empty() {
                        // 忽略已存在等非致命错误
                        if let Err(e) = std::fs::create_dir_all(parent) {
                            warn!("Failed to create database dir {:?}: {}", parent, e);
                        }
                    }
                }
            }
        }

        // 3) 连接使用规范化后的 db_url
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

        debug!("✅ Database schema initialized");
        Ok(())
    }

    pub async fn store_wallet(&self, name: &str, encrypted_data: &[u8]) -> Result<()> {
        debug!("Storing wallet: {}", name);

        let wallet_id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now();

        sqlx::query(
            r#"
            INSERT INTO wallets (id, name, encrypted_data, quantum_safe, created_at, updated_at)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6)
            "#,
        )
        .bind(&wallet_id)
        .bind(name)
        .bind(encrypted_data)
        .bind(true) // Assume quantum safe for now
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

        debug!("✅ Wallet stored: {}", name);
        Ok(())
    }

    pub async fn load_wallet(&self, name: &str) -> Result<Vec<u8>> {
        debug!("Loading wallet: {}", name);

        let row = sqlx::query("SELECT id, encrypted_data FROM wallets WHERE name = ?1")
            .bind(name)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to load wallet: {}", e))?;

        match row {
            Some(row) => {
                let wallet_id: String = row.get("id");
                let encrypted_data: Vec<u8> = row.get("encrypted_data");

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
                Ok(encrypted_data)
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

        debug!("✅ Transaction stored: {}", tx_data.tx_hash);
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
        .bind(chrono::Utc::now())
        .execute(&self.pool)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to log action: {}", e))?;

        Ok(())
    }

    pub async fn get_audit_logs(&self, wallet_id: Option<&str>) -> Result<Vec<AuditLog>> {
        let (query, params): (&str, Vec<&str>) = match wallet_id {
            Some(id) => (
                "SELECT * FROM audit_logs WHERE wallet_id = ?1 ORDER BY created_at DESC",
                vec![id],
            ),
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

#[derive(Debug, Clone)]
pub struct WalletMetadata {
    pub id: String,
    pub name: String,
    pub quantum_safe: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
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
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub confirmed_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Clone)]
pub struct AuditLog {
    pub id: i64,
    pub wallet_id: Option<String>,
    pub action: String,
    pub details: Option<String>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}
