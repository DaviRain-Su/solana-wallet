// use gpui::*;
use std::collections::VecDeque;
use uuid::Uuid;
use chrono::{DateTime, Utc};
// use solana_sdk::pubkey::Pubkey;

// 临时定义 Pubkey 类型
#[derive(Clone, Debug, PartialEq)]
pub struct Pubkey([u8; 32]);

impl Pubkey {
    pub fn new_unique() -> Self {
        let mut bytes = [0u8; 32];
        // 简单的随机生成
        for i in 0..32 {
            bytes[i] = (uuid::Uuid::new_v4().as_bytes()[i % 16]) as u8;
        }
        Self(bytes)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum SolanaNetwork {
    MainnetBeta,
    Devnet,
    Testnet,
    Localnet,
}

impl SolanaNetwork {
    pub fn rpc_url(&self) -> &str {
        match self {
            SolanaNetwork::MainnetBeta => "https://api.mainnet-beta.solana.com",
            SolanaNetwork::Devnet => "https://api.devnet.solana.com",
            SolanaNetwork::Testnet => "https://api.testnet.solana.com",
            SolanaNetwork::Localnet => "http://localhost:8899",
        }
    }
}

#[derive(Clone, Debug)]
pub struct WalletAccount {
    pub id: Uuid,
    pub name: String,
    pub pubkey: Pubkey,
    pub balance: u64,
    pub creation_date: DateTime<Utc>,
}

#[derive(Clone, Debug)]
pub struct Notification {
    pub id: Uuid,
    pub message: String,
    pub kind: NotificationKind,
    pub timestamp: DateTime<Utc>,
}

#[derive(Clone, Debug)]
pub enum NotificationKind {
    Success,
    Error,
    Warning,
    Info,
}

pub struct AppState {
    pub accounts: Vec<WalletAccount>,
    pub current_account: Option<Uuid>,
    pub network: SolanaNetwork,
    pub notifications: VecDeque<Notification>,
    pub is_locked: bool,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            accounts: Vec::new(),
            current_account: None,
            network: SolanaNetwork::MainnetBeta,
            notifications: VecDeque::new(),
            is_locked: false,
        }
    }

    pub fn add_account(&mut self, account: WalletAccount) {
        let account_id = account.id;
        self.accounts.push(account);
        if self.current_account.is_none() {
            self.current_account = Some(account_id);
        }
    }

    pub fn get_current_account(&self) -> Option<&WalletAccount> {
        self.current_account
            .and_then(|id| self.accounts.iter().find(|acc| acc.id == id))
    }

    pub fn add_notification(&mut self, message: String, kind: NotificationKind) {
        let notification = Notification {
            id: Uuid::new_v4(),
            message,
            kind,
            timestamp: Utc::now(),
        };
        self.notifications.push_back(notification);
        
        // Keep only last 10 notifications
        while self.notifications.len() > 10 {
            self.notifications.pop_front();
        }
    }

    pub fn switch_network(&mut self, network: SolanaNetwork) {
        self.network = network;
    }

    pub fn lock(&mut self) {
        self.is_locked = true;
    }

    pub fn unlock(&mut self) {
        self.is_locked = false;
    }
}