//! Defines the main application entry point.

use crate::service::WalletService;

/// The main application struct, holding the service registry.
#[derive(Debug, Default)]
pub struct Application {
    services: WalletService,
}

impl Application {
    /// Creates a new `Application`.
    pub fn new() -> Self {
        Self::default()
    }

    /// 杩斿洖瀵规湇鍔℃敞鍐岃〃鐨勫紩鐢ㄣ€?    pub fn services(&self) -> &WalletService {
        &self.services
    }
}
