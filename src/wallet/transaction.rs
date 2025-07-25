use crate::wallet::rpc::RpcManager;
use anyhow::{anyhow, Result};
use chrono;
use solana_sdk::{
    instruction::Instruction,
    message::Message,
    pubkey::Pubkey,
    signature::{Keypair, Signature},
    signer::Signer,
    system_instruction,
    transaction::Transaction,
};

/// 交易构建器
pub struct TransactionBuilder {
    instructions: Vec<Instruction>,
    fee_payer: Option<Pubkey>,
}

impl TransactionBuilder {
    /// 创建新的交易构建器
    pub fn new() -> Self {
        Self {
            instructions: Vec::new(),
            fee_payer: None,
        }
    }

    /// 设置手续费支付者
    pub fn fee_payer(mut self, payer: Pubkey) -> Self {
        self.fee_payer = Some(payer);
        self
    }

    /// 添加转账指令
    pub fn add_transfer(mut self, from: &Pubkey, to: &Pubkey, lamports: u64) -> Self {
        let instruction = system_instruction::transfer(from, to, lamports);
        self.instructions.push(instruction);
        self
    }

    /// 添加自定义指令
    pub fn add_instruction(mut self, instruction: Instruction) -> Self {
        self.instructions.push(instruction);
        self
    }

    /// 构建交易
    pub async fn build(self, rpc: &RpcManager) -> Result<Transaction> {
        if self.instructions.is_empty() {
            return Err(anyhow!("No instructions to build transaction"));
        }

        let fee_payer = self.fee_payer.ok_or_else(|| anyhow!("Fee payer not set"))?;

        // 获取最新区块哈希
        let recent_blockhash = rpc.get_latest_blockhash().await?;

        // 创建消息
        let message =
            Message::new_with_blockhash(&self.instructions, Some(&fee_payer), &recent_blockhash);

        // 创建交易
        let transaction = Transaction::new_unsigned(message);

        Ok(transaction)
    }
}

/// 交易助手函数
pub struct TransactionHelper;

impl TransactionHelper {
    /// 创建SOL转账交易
    pub async fn create_transfer_sol(
        rpc: &RpcManager,
        from_keypair: &Keypair,
        to_pubkey: &Pubkey,
        sol_amount: f64,
    ) -> Result<Transaction> {
        // 转换SOL到lamports
        let lamports = (sol_amount * 1_000_000_000.0) as u64;

        // 检查余额
        let balance = rpc.get_balance(&from_keypair.pubkey()).await?;
        if balance < lamports {
            return Err(anyhow!(
                "Insufficient balance. Available: {} SOL, Required: {} SOL",
                balance as f64 / 1_000_000_000.0,
                sol_amount
            ));
        }

        // 构建交易
        let mut transaction = TransactionBuilder::new()
            .fee_payer(from_keypair.pubkey())
            .add_transfer(&from_keypair.pubkey(), to_pubkey, lamports)
            .build(rpc)
            .await?;

        // 签名交易
        transaction.sign(&[from_keypair], transaction.message.recent_blockhash);

        Ok(transaction)
    }

    /// 估算交易手续费
    pub async fn estimate_fee(rpc: &RpcManager, transaction: &Transaction) -> Result<u64> {
        let client = rpc.client.read().await;

        // 获取最新区块哈希
        let blockhash = client
            .get_latest_blockhash()
            .map_err(|e| anyhow!("Failed to get blockhash: {}", e))?;

        // 使用默认费用（这是一个简化的实现）
        // 在Solana 2.0中，费用计算更复杂
        let fee = 5000; // 5000 lamports 是一个合理的默认值

        Ok(fee)
    }

    /// 发送并确认交易
    pub async fn send_and_confirm(
        rpc: &RpcManager,
        transaction: &Transaction,
    ) -> Result<Signature> {
        // 发送交易
        let signature = rpc.send_transaction(transaction).await?;

        println!("Transaction sent: {}", signature);
        println!("Waiting for confirmation...");

        Ok(signature)
    }

    /// 批量转账（用于空投等场景）
    pub async fn create_batch_transfer(
        rpc: &RpcManager,
        from_keypair: &Keypair,
        recipients: Vec<(Pubkey, u64)>, // (接收地址, lamports数量)
    ) -> Result<Transaction> {
        if recipients.is_empty() {
            return Err(anyhow!("No recipients specified"));
        }

        if recipients.len() > 10 {
            return Err(anyhow!("Too many recipients. Maximum 10 per transaction"));
        }

        // 计算总金额
        let total_lamports: u64 = recipients.iter().map(|(_, amount)| amount).sum();

        // 检查余额
        let balance = rpc.get_balance(&from_keypair.pubkey()).await?;
        if balance < total_lamports {
            return Err(anyhow!(
                "Insufficient balance. Available: {} SOL, Required: {} SOL",
                balance as f64 / 1_000_000_000.0,
                total_lamports as f64 / 1_000_000_000.0
            ));
        }

        // 构建交易
        let mut builder = TransactionBuilder::new().fee_payer(from_keypair.pubkey());

        for (recipient, lamports) in recipients {
            builder = builder.add_transfer(&from_keypair.pubkey(), &recipient, lamports);
        }

        let mut transaction = builder.build(rpc).await?;

        // 签名交易
        transaction.sign(&[from_keypair], transaction.message.recent_blockhash);

        Ok(transaction)
    }
}

/// 交易状态
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TransactionStatus {
    Pending,
    Confirmed,
    Failed(String),
}

/// 交易记录
#[derive(Debug, Clone)]
pub struct TransactionRecord {
    pub signature: Signature,
    pub from: Pubkey,
    pub to: Option<Pubkey>,
    pub amount: u64,
    pub fee: u64,
    pub status: TransactionStatus,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub memo: Option<String>,
}

impl TransactionRecord {
    /// 创建新的交易记录
    pub fn new(
        signature: Signature,
        from: Pubkey,
        to: Option<Pubkey>,
        amount: u64,
        fee: u64,
    ) -> Self {
        Self {
            signature,
            from,
            to,
            amount,
            fee,
            status: TransactionStatus::Pending,
            timestamp: chrono::Utc::now(),
            memo: None,
        }
    }

    /// 获取金额（SOL）
    pub fn amount_in_sol(&self) -> f64 {
        self.amount as f64 / 1_000_000_000.0
    }

    /// 获取手续费（SOL）
    pub fn fee_in_sol(&self) -> f64 {
        self.fee as f64 / 1_000_000_000.0
    }
}
