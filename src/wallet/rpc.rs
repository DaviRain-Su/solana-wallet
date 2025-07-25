use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    commitment_config::CommitmentConfig,
    pubkey::Pubkey,
    signature::Signature,
    transaction::Transaction,
};
use anyhow::{Result, anyhow};
use std::sync::Arc;
use tokio::sync::RwLock;

/// Solana网络类型
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SolanaNetwork {
    Mainnet,
    Devnet,
    Testnet,
    Localnet,
    Custom(String), // 自定义RPC URL
}

impl SolanaNetwork {
    /// 获取RPC端点URL
    pub fn rpc_url(&self) -> String {
        match self {
            SolanaNetwork::Mainnet => "https://api.mainnet-beta.solana.com".to_string(),
            SolanaNetwork::Devnet => "https://api.devnet.solana.com".to_string(),
            SolanaNetwork::Testnet => "https://api.testnet.solana.com".to_string(),
            SolanaNetwork::Localnet => "http://localhost:8899".to_string(),
            SolanaNetwork::Custom(url) => url.clone(),
        }
    }
    
    /// 获取网络名称
    pub fn name(&self) -> &str {
        match self {
            SolanaNetwork::Mainnet => "Mainnet Beta",
            SolanaNetwork::Devnet => "Devnet",
            SolanaNetwork::Testnet => "Testnet",
            SolanaNetwork::Localnet => "Localnet",
            SolanaNetwork::Custom(_) => "Custom",
        }
    }
}

/// RPC客户端管理器
pub struct RpcManager {
    pub(crate) client: Arc<RwLock<RpcClient>>,
    network: Arc<RwLock<SolanaNetwork>>,
}

impl RpcManager {
    /// 创建新的RPC管理器
    pub fn new(network: SolanaNetwork) -> Self {
        let client = RpcClient::new_with_commitment(
            network.rpc_url(),
            CommitmentConfig::confirmed(),
        );
        
        Self {
            client: Arc::new(RwLock::new(client)),
            network: Arc::new(RwLock::new(network)),
        }
    }
    
    /// 切换网络
    pub async fn switch_network(&self, network: SolanaNetwork) -> Result<()> {
        let new_client = RpcClient::new_with_commitment(
            network.rpc_url(),
            CommitmentConfig::confirmed(),
        );
        
        *self.client.write().await = new_client;
        *self.network.write().await = network;
        
        Ok(())
    }
    
    /// 获取当前网络
    pub async fn current_network(&self) -> SolanaNetwork {
        self.network.read().await.clone()
    }
    
    /// 获取账户余额（单位：lamports）
    pub async fn get_balance(&self, pubkey: &Pubkey) -> Result<u64> {
        let client = self.client.read().await;
        client.get_balance(pubkey)
            .map_err(|e| anyhow!("Failed to get balance: {}", e))
    }
    
    /// 获取账户余额（单位：SOL）
    pub async fn get_balance_in_sol(&self, pubkey: &Pubkey) -> Result<f64> {
        let balance_lamports = self.get_balance(pubkey).await?;
        Ok(balance_lamports as f64 / 1_000_000_000.0)
    }
    
    /// 获取最新区块高度
    pub async fn get_block_height(&self) -> Result<u64> {
        let client = self.client.read().await;
        client.get_block_height()
            .map_err(|e| anyhow!("Failed to get block height: {}", e))
    }
    
    /// 获取最新区块哈希
    pub async fn get_latest_blockhash(&self) -> Result<solana_sdk::hash::Hash> {
        let client = self.client.read().await;
        client.get_latest_blockhash()
            .map_err(|e| anyhow!("Failed to get latest blockhash: {}", e))
    }
    
    /// 发送交易
    pub async fn send_transaction(&self, transaction: &Transaction) -> Result<Signature> {
        let client = self.client.read().await;
        client.send_and_confirm_transaction(transaction)
            .map_err(|e| anyhow!("Failed to send transaction: {}", e))
    }
    
    /// 获取交易信息
    pub async fn get_transaction(&self, signature: &Signature) -> Result<Option<solana_transaction_status::EncodedConfirmedTransactionWithStatusMeta>> {
        let client = self.client.read().await;
        match client.get_transaction(signature, solana_transaction_status::UiTransactionEncoding::Json) {
            Ok(tx) => Ok(Some(tx)),
            Err(e) => {
                if e.to_string().contains("TransactionNotFound") {
                    Ok(None)
                } else {
                    Err(anyhow!("Failed to get transaction: {}", e))
                }
            }
        }
    }
    
    /// 检查连接状态
    pub async fn check_connection(&self) -> Result<()> {
        let client = self.client.read().await;
        let version = client.get_version()
            .map_err(|e| anyhow!("Failed to connect to RPC: {}", e))?;
        
        println!("Connected to Solana {} ({})", 
            self.network.read().await.name(), 
            version.solana_core
        );
        
        Ok(())
    }
    
    /// 请求空投（仅限测试网）
    pub async fn request_airdrop(&self, pubkey: &Pubkey, lamports: u64) -> Result<Signature> {
        let network = self.network.read().await.clone();
        match network {
            SolanaNetwork::Devnet | SolanaNetwork::Testnet | SolanaNetwork::Localnet | SolanaNetwork::Custom(_) => {
                let client = self.client.read().await;
                let signature = client.request_airdrop(pubkey, lamports)
                    .map_err(|e| anyhow!("Failed to request airdrop: {}", e))?;
                
                // 等待确认
                let recent_blockhash = client.get_latest_blockhash()?;
                client.confirm_transaction_with_spinner(
                    &signature,
                    &recent_blockhash,
                    CommitmentConfig::confirmed(),
                ).map_err(|e| anyhow!("Failed to confirm airdrop: {}", e))?;
                
                Ok(signature)
            }
            SolanaNetwork::Mainnet => {
                Err(anyhow!("Airdrop is not available on mainnet"))
            }
        }
    }
}

/// 获取账户信息
#[derive(Debug, Clone)]
pub struct AccountInfo {
    pub pubkey: Pubkey,
    pub balance: u64,
    pub executable: bool,
    pub owner: Pubkey,
}

impl RpcManager {
    /// 获取账户详细信息
    pub async fn get_account_info(&self, pubkey: &Pubkey) -> Result<Option<AccountInfo>> {
        let client = self.client.read().await;
        
        match client.get_account(pubkey) {
            Ok(account) => {
                Ok(Some(AccountInfo {
                    pubkey: *pubkey,
                    balance: account.lamports,
                    executable: account.executable,
                    owner: account.owner,
                }))
            }
            Err(e) => {
                // 账户不存在时返回None而不是错误
                if e.to_string().contains("AccountNotFound") {
                    Ok(None)
                } else {
                    Err(anyhow!("Failed to get account info: {}", e))
                }
            }
        }
    }
    
    /// 批量获取账户余额
    pub async fn get_multiple_balances(&self, pubkeys: &[Pubkey]) -> Result<Vec<(Pubkey, u64)>> {
        let client = self.client.read().await;
        let mut results = Vec::new();
        
        // Solana RPC批量请求有限制，分批处理
        for chunk in pubkeys.chunks(100) {
            for pubkey in chunk {
                match client.get_balance(pubkey) {
                    Ok(balance) => results.push((*pubkey, balance)),
                    Err(_) => results.push((*pubkey, 0)), // 账户不存在时余额为0
                }
            }
        }
        
        Ok(results)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_network_urls() {
        assert_eq!(SolanaNetwork::Mainnet.rpc_url(), "https://api.mainnet-beta.solana.com");
        assert_eq!(SolanaNetwork::Devnet.rpc_url(), "https://api.devnet.solana.com");
        assert_eq!(SolanaNetwork::Testnet.rpc_url(), "https://api.testnet.solana.com");
        assert_eq!(SolanaNetwork::Localnet.rpc_url(), "http://localhost:8899");
        assert_eq!(SolanaNetwork::Custom("http://localhost:9999".to_string()).rpc_url(), "http://localhost:9999");
    }
    
    #[tokio::test]
    async fn test_rpc_manager_creation() {
        let manager = RpcManager::new(SolanaNetwork::Devnet);
        assert_eq!(manager.current_network().await, SolanaNetwork::Devnet);
    }
}