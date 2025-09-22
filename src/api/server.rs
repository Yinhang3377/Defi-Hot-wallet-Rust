use anyhow::Result;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
    routing::{delete, get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::net::TcpListener;
use tower::ServiceBuilder;
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use tracing::{info, warn};

use crate::core::config::WalletConfig;
use crate::core::WalletManager;
use crate::monitoring::{
    get_metrics, get_security_monitor, SecurityEvent, SecurityEventType, SecuritySeverity,
};

#[derive(Clone)]
pub struct WalletServer {
    host: String,
    port: u16,
    wallet_manager: Arc<WalletManager>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateWalletRequest {
    pub name: String,
    pub quantum_safe: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SendTransactionRequest {
    pub to_address: String,
    pub amount: String,
    pub network: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WalletResponse {
    pub id: String,
    pub name: String,
    pub quantum_safe: bool,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BalanceResponse {
    pub balance: String,
    pub network: String,
    pub symbol: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TransactionResponse {
    pub tx_hash: String,
    pub status: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub error: String,
    pub code: String,
}

#[derive(Debug, Deserialize)]
pub struct BalanceQuery {
    pub network: String,
}

impl WalletServer {
    pub async fn new(host: String, port: u16, config: WalletConfig) -> Result<Self> {
        info!("🚀 Initializing wallet server on {}:{}", host, port);

        let wallet_manager = Arc::new(WalletManager::new(&config).await?);

        Ok(Self {
            host,
            port,
            wallet_manager,
        })
    }

    pub async fn start(self) -> Result<()> {
        let addr = format!("{}:{}", self.host, self.port);
        let app = self.create_router().await;

        let listener = TcpListener::bind(&addr).await?;

        info!("🌐 Wallet server starting on http://{}", addr);
        info!("📚 API Documentation:");
        info!("  POST   /api/wallets           - Create wallet");
        info!("  GET    /api/wallets           - List wallets");
        info!("  DELETE /api/wallets/:name     - Delete wallet");
        info!("  GET    /api/wallets/:name/balance - Get balance");
        info!("  POST   /api/wallets/:name/send - Send transaction");
        info!("  GET    /api/health           - Health check");
        info!("  GET    /api/metrics          - Prometheus metrics");

        axum::serve(listener, app).await?;

        Ok(())
    }

    async fn create_router(self) -> Router {
        Router::new()
            .route("/api/health", get(health_check))
            .route("/api/metrics", get(metrics_handler))
            .route("/api/wallets", post(create_wallet))
            .route("/api/wallets", get(list_wallets))
            .route("/api/wallets/:name", delete(delete_wallet))
            .route("/api/wallets/:name/balance", get(get_balance))
            .route("/api/wallets/:name/send", post(send_transaction))
            .with_state(self.wallet_manager.clone())
            .layer(
                ServiceBuilder::new()
                    .layer(TraceLayer::new_for_http())
                    .layer(CorsLayer::permissive()),
            )
    }
}

async fn health_check() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "healthy",
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "version": env!("CARGO_PKG_VERSION")
    }))
}

async fn metrics_handler() -> Result<String, StatusCode> {
    match get_metrics() {
        Some(metrics) => match metrics.export_metrics() {
            Ok(metrics_string) => Ok(metrics_string),
            Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
        },
        None => Err(StatusCode::SERVICE_UNAVAILABLE),
    }
}

async fn create_wallet(
    State(wallet_manager): State<Arc<WalletManager>>,
    Json(request): Json<CreateWalletRequest>,
) -> Result<Json<WalletResponse>, (StatusCode, Json<ErrorResponse>)> {
    info!("🔧 Creating wallet: {}", request.name);

    let quantum_safe = request.quantum_safe.unwrap_or(true);

    match wallet_manager
        .create_wallet(&request.name, quantum_safe)
        .await
    {
        Ok(wallet_info) => {
            // Record metrics
            if let Some(metrics) = get_metrics() {
                metrics.record_wallet_created();
                if quantum_safe {
                    metrics.record_quantum_encryption();
                }
            }

            Ok(Json(WalletResponse {
                id: wallet_info.id.to_string(),
                name: wallet_info.name,
                quantum_safe: wallet_info.quantum_safe,
                created_at: wallet_info.created_at.to_rfc3339(),
            }))
        }
        Err(e) => {
            warn!("Failed to create wallet {}: {}", request.name, e);
            Err((
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    error: e.to_string(),
                    code: "WALLET_CREATION_FAILED".to_string(),
                }),
            ))
        }
    }
}

async fn list_wallets(
    State(_wallet_manager): State<Arc<WalletManager>>,
) -> Result<Json<Vec<WalletResponse>>, (StatusCode, Json<ErrorResponse>)> {
    // For demo purposes, return empty list
    // In a real implementation, this would query the storage layer
    Ok(Json(vec![]))
}

async fn delete_wallet(
    State(_wallet_manager): State<Arc<WalletManager>>,
    Path(name): Path<String>,
) -> Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    warn!("🗑️ Deleting wallet: {}", name);

    // Record security event
    if let Some(monitor) = get_security_monitor() {
        let event = SecurityEvent {
            event_type: SecurityEventType::UnauthorizedAccess,
            description: format!("Wallet deletion requested: {}", name),
            severity: SecuritySeverity::Medium,
            timestamp: chrono::Utc::now(),
            source_ip: None, // In a real implementation, extract from request
            wallet_id: Some(name.clone()),
        };
        monitor.report_security_event(event).await;
    }

    // Record metrics
    if let Some(metrics) = get_metrics() {
        metrics.record_wallet_deleted();
    }

    Ok(StatusCode::NO_CONTENT)
}

async fn get_balance(
    State(wallet_manager): State<Arc<WalletManager>>,
    Path(name): Path<String>,
    Query(params): Query<BalanceQuery>,
) -> Result<Json<BalanceResponse>, (StatusCode, Json<ErrorResponse>)> {
    info!(
        "💰 Getting balance for wallet: {} on network: {}",
        name, params.network
    );

    match wallet_manager.get_balance(&name, &params.network).await {
        Ok(balance) => {
            // Record metrics
            if let Some(metrics) = get_metrics() {
                metrics.record_wallet_accessed();
            }

            let symbol = match params.network.as_str() {
                "eth" | "ethereum" | "sepolia" => "ETH",
                "solana" | "solana-devnet" => "SOL",
                _ => "UNKNOWN",
            };

            Ok(Json(BalanceResponse {
                balance,
                network: params.network,
                symbol: symbol.to_string(),
            }))
        }
        Err(e) => {
            warn!("Failed to get balance for wallet {}: {}", name, e);
            Err((
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    error: e.to_string(),
                    code: "BALANCE_FETCH_FAILED".to_string(),
                }),
            ))
        }
    }
}

async fn send_transaction(
    State(wallet_manager): State<Arc<WalletManager>>,
    Path(name): Path<String>,
    Json(request): Json<SendTransactionRequest>,
) -> Result<Json<TransactionResponse>, (StatusCode, Json<ErrorResponse>)> {
    info!(
        "💸 Sending transaction from wallet: {} to: {} amount: {} on: {}",
        name, request.to_address, request.amount, request.network
    );

    // Check for suspicious activity
    if let Some(monitor) = get_security_monitor() {
        // Parse amount to check for suspiciously large transactions
        if let Ok(amount_f64) = request.amount.parse::<f64>() {
            if amount_f64 > 100.0 {
                // Arbitrary threshold
                let event = SecurityEvent {
                    event_type: SecurityEventType::SuspiciousTransaction,
                    description: format!(
                        "Large transaction: {} {} from wallet {}",
                        request.amount, request.network, name
                    ),
                    severity: SecuritySeverity::Medium,
                    timestamp: chrono::Utc::now(),
                    source_ip: None,
                    wallet_id: Some(name.clone()),
                };
                monitor.report_security_event(event).await;
            }
        }
    }

    match wallet_manager
        .send_transaction(
            &name,
            &request.to_address,
            &request.amount,
            &request.network,
        )
        .await
    {
        Ok(tx_hash) => {
            // Record metrics
            if let Some(metrics) = get_metrics() {
                let amount_f64 = request.amount.parse::<f64>().unwrap_or(0.0);
                let fee = 0.001; // Simplified fee calculation
                metrics.record_transaction_sent(amount_f64, fee);
            }

            Ok(Json(TransactionResponse {
                tx_hash,
                status: "sent".to_string(),
            }))
        }
        Err(e) => {
            warn!("Failed to send transaction from wallet {}: {}", name, e);

            // Record failed transaction
            if let Some(metrics) = get_metrics() {
                metrics.record_transaction_failed();
            }

            Err((
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    error: e.to_string(),
                    code: "TRANSACTION_FAILED".to_string(),
                }),
            ))
        }
    }
}
