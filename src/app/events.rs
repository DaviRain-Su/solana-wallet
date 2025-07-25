// use solana_sdk::{pubkey::Pubkey, signature::Signature};
use uuid::Uuid;
use super::state::{SolanaNetwork, Pubkey};

// 临时定义 Signature 类型
#[derive(Clone, Debug)]
pub struct Signature([u8; 64]);

#[derive(Clone, Debug)]
pub enum WalletEvent {
    // Account events
    AccountCreated { account_id: Uuid },
    AccountDeleted { account_id: Uuid },
    AccountSwitched { account_id: Uuid },
    
    // Transaction events
    TransactionSent {
        signature: Signature,
        from: Pubkey,
        to: Pubkey,
        amount: u64,
    },
    TransactionConfirmed {
        signature: Signature,
    },
    TransactionFailed {
        error: String,
    },
    
    // Balance events
    BalanceUpdated {
        account_id: Uuid,
        balance: u64,
    },
    
    // Network events
    NetworkChanged {
        network: SolanaNetwork,
    },
    NetworkConnectionLost,
    NetworkConnectionRestored,
    
    // Security events
    WalletLocked,
    WalletUnlocked,
    UnauthorizedAccess,
}