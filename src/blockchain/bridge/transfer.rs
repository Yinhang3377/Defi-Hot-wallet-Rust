// filepath: src/blockchain/bridge/transfer.rs
use crate::blockchain::traits::Bridge;
use crate::core::wallet_info::SecureWalletData;
use tracing::info;

pub async fn initiate_bridge_transfer(
    bridge: &dyn Bridge,
    from_chain: &str,
    to_chain: &str,
    token: &str,
    amount: &str,
    wallet_data: &SecureWalletData,
) -> anyhow::Result<String> {
    info!(
        "Initiating bridge transfer of {} {} from {} to {} via bridge",
        amount, token, from_chain, to_chain
    );
    bridge.transfer_across_chains(from_chain, to_chain, token, amount, wallet_data).await
}
