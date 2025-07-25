use super::state::SolanaNetwork;
use solana_sdk::{pubkey::Pubkey, signature::Signature};
use uuid::Uuid;

#[derive(Clone, Debug)]
pub enum WalletEvent {
    // Account events
    AccountCreated {
        account_id: Uuid,
    },
    AccountDeleted {
        account_id: Uuid,
    },
    AccountSwitched {
        account_id: Uuid,
    },

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
