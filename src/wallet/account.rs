use solana_sdk::pubkey::Pubkey;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use anyhow::Result;
use crate::wallet::keypair::WalletKeypair;

/// Represents a wallet account
#[derive(Debug, Serialize, Deserialize)]
pub struct WalletAccount {
    pub id: Uuid,
    pub name: String,
    pub pubkey: Pubkey,
    #[serde(skip)]
    pub keypair: Option<WalletKeypair>,
    pub balance: u64,
    pub created_at: DateTime<Utc>,
    pub is_imported: bool,
    pub derivation_path: Option<String>,
}

impl WalletAccount {
    /// Create a new account from a keypair
    pub fn new(name: String, keypair: WalletKeypair, is_imported: bool) -> Self {
        Self {
            id: Uuid::new_v4(),
            name,
            pubkey: keypair.pubkey(),
            keypair: Some(keypair),
            balance: 0,
            created_at: Utc::now(),
            is_imported,
            derivation_path: None,
        }
    }

    /// Create an account with derivation path
    pub fn with_derivation_path(
        name: String,
        keypair: WalletKeypair,
        derivation_path: String,
    ) -> Self {
        let mut account = Self::new(name, keypair, false);
        account.derivation_path = Some(derivation_path);
        account
    }

    /// Create a watch-only account (no private key)
    pub fn watch_only(name: String, pubkey: Pubkey) -> Self {
        Self {
            id: Uuid::new_v4(),
            name,
            pubkey,
            keypair: None,
            balance: 0,
            created_at: Utc::now(),
            is_imported: true,
            derivation_path: None,
        }
    }

    /// Check if this is a watch-only account
    pub fn is_watch_only(&self) -> bool {
        self.keypair.is_none()
    }

    /// Update the balance
    pub fn update_balance(&mut self, balance: u64) {
        self.balance = balance;
    }

    /// Get account info for display
    pub fn info(&self) -> AccountInfo {
        AccountInfo {
            id: self.id,
            name: self.name.clone(),
            pubkey: self.pubkey,
            balance: self.balance,
            is_watch_only: self.is_watch_only(),
        }
    }
}

/// Simplified account info for UI display
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountInfo {
    pub id: Uuid,
    pub name: String,
    pub pubkey: Pubkey,
    pub balance: u64,
    pub is_watch_only: bool,
}

impl AccountInfo {
    /// Get shortened pubkey for display (first 4 and last 4 chars)
    pub fn short_pubkey(&self) -> String {
        let pubkey_str = self.pubkey.to_string();
        if pubkey_str.len() > 8 {
            format!(
                "{}...{}",
                &pubkey_str[..4],
                &pubkey_str[pubkey_str.len() - 4..]
            )
        } else {
            pubkey_str
        }
    }

    /// Format balance in SOL
    pub fn balance_in_sol(&self) -> f64 {
        self.balance as f64 / 1_000_000_000.0
    }
}