use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use solana_sdk::{
    pubkey::Pubkey,
    transaction::Transaction,
};
use std::fmt;

/// 签名请求的来源
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SigningRequestSource {
    /// 来自 DApp 的请求
    DApp {
        name: String,
        url: String,
        icon: Option<String>,
    },
    /// 来自内部操作的请求
    Internal {
        operation: String,
    },
}

/// 签名请求状态
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SigningRequestStatus {
    /// 等待用户确认
    Pending,
    /// 用户已批准
    Approved,
    /// 用户已拒绝
    Rejected,
    /// 签名完成
    Signed,
    /// 出错
    Failed(String),
}

/// 交易签名请求
#[derive(Debug, Clone)]
pub struct SigningRequest {
    /// 请求 ID
    pub id: String,
    /// 请求来源
    pub source: SigningRequestSource,
    /// 要签名的交易
    pub transaction: Transaction,
    /// 请求时间
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// 请求状态
    pub status: SigningRequestStatus,
    /// 预估的交易费用
    pub estimated_fee: Option<u64>,
    /// 请求消息（可选）
    pub message: Option<String>,
}

impl SigningRequest {
    /// 创建新的签名请求
    pub fn new(
        source: SigningRequestSource,
        transaction: Transaction,
        message: Option<String>,
    ) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            source,
            transaction,
            created_at: chrono::Utc::now(),
            status: SigningRequestStatus::Pending,
            estimated_fee: None,
            message,
        }
    }

    /// 设置预估费用
    pub fn with_estimated_fee(mut self, fee: u64) -> Self {
        self.estimated_fee = Some(fee);
        self
    }

    /// 批准请求
    pub fn approve(&mut self) {
        self.status = SigningRequestStatus::Approved;
    }

    /// 拒绝请求
    pub fn reject(&mut self) {
        self.status = SigningRequestStatus::Rejected;
    }

    /// 标记为已签名
    pub fn mark_signed(&mut self) {
        self.status = SigningRequestStatus::Signed;
    }

    /// 标记为失败
    pub fn mark_failed(&mut self, error: String) {
        self.status = SigningRequestStatus::Failed(error);
    }

    /// 获取交易详情
    pub fn get_transaction_details(&self) -> TransactionDetails {
        let message = &self.transaction.message;
        
        // 解析指令
        let instructions: Vec<InstructionInfo> = message
            .instructions
            .iter()
            .map(|ix| {
                let program_id = message.account_keys[ix.program_id_index as usize];
                InstructionInfo {
                    program_id,
                    accounts: ix
                        .accounts
                        .iter()
                        .map(|&idx| message.account_keys[idx as usize])
                        .collect(),
                    data: ix.data.clone(),
                    instruction_type: identify_instruction_type(&program_id, &ix.data),
                }
            })
            .collect();

        TransactionDetails {
            fee_payer: message.account_keys[0],
            signatures_required: message.header.num_required_signatures,
            readonly_signers: message.header.num_readonly_signed_accounts,
            readonly_nonsigners: message.header.num_readonly_unsigned_accounts,
            instructions,
            recent_blockhash: message.recent_blockhash,
        }
    }
}

/// 交易详情
#[derive(Debug)]
pub struct TransactionDetails {
    pub fee_payer: Pubkey,
    pub signatures_required: u8,
    pub readonly_signers: u8,
    pub readonly_nonsigners: u8,
    pub instructions: Vec<InstructionInfo>,
    pub recent_blockhash: solana_sdk::hash::Hash,
}

/// 指令信息
#[derive(Debug)]
pub struct InstructionInfo {
    pub program_id: Pubkey,
    pub accounts: Vec<Pubkey>,
    pub data: Vec<u8>,
    pub instruction_type: InstructionType,
}

/// 指令类型
#[derive(Debug, Clone)]
pub enum InstructionType {
    /// 系统转账
    SystemTransfer { from: Pubkey, to: Pubkey, lamports: u64 },
    /// SPL Token 转账
    TokenTransfer { amount: u64 },
    /// 未知指令
    Unknown,
}

impl fmt::Display for InstructionType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            InstructionType::SystemTransfer { from, to, lamports } => {
                write!(
                    f,
                    "转账 {} SOL 从 {}... 到 {}...",
                    *lamports as f64 / 1_000_000_000.0,
                    &from.to_string()[..8],
                    &to.to_string()[..8]
                )
            }
            InstructionType::TokenTransfer { amount } => {
                write!(f, "代币转账: {} 个单位", amount)
            }
            InstructionType::Unknown => write!(f, "未知操作"),
        }
    }
}

/// 识别指令类型
fn identify_instruction_type(program_id: &Pubkey, data: &[u8]) -> InstructionType {
    // 系统程序
    if program_id == &solana_sdk::system_program::id() {
        // 系统转账指令的标识符是 2
        if data.len() >= 12 && data[0..4] == [2, 0, 0, 0] {
            // 解析 lamports (little-endian u64)
            let lamports = u64::from_le_bytes([
                data[4], data[5], data[6], data[7],
                data[8], data[9], data[10], data[11],
            ]);
            
            // 注意：from 和 to 地址需要从账户列表中获取
            // 这里简化处理，实际使用时需要结合指令的账户信息
            return InstructionType::SystemTransfer {
                from: Pubkey::default(),
                to: Pubkey::default(),
                lamports,
            };
        }
    }
    
    // SPL Token 程序
    if program_id == &spl_token::id() {
        // Token 转账指令
        if data.len() >= 9 && data[0] == 3 {
            let amount = u64::from_le_bytes([
                data[1], data[2], data[3], data[4],
                data[5], data[6], data[7], data[8],
            ]);
            return InstructionType::TokenTransfer { amount };
        }
    }
    
    InstructionType::Unknown
}

/// 签名请求管理器
pub struct SigningRequestManager {
    /// 待处理的请求
    pending_requests: Vec<SigningRequest>,
    /// 历史请求
    history: Vec<SigningRequest>,
}

impl SigningRequestManager {
    /// 创建新的管理器
    pub fn new() -> Self {
        Self {
            pending_requests: Vec::new(),
            history: Vec::new(),
        }
    }

    /// 添加新的签名请求
    pub fn add_request(&mut self, request: SigningRequest) -> String {
        let id = request.id.clone();
        self.pending_requests.push(request);
        id
    }

    /// 获取待处理的请求
    pub fn get_pending_requests(&self) -> &[SigningRequest] {
        &self.pending_requests
    }

    /// 获取特定请求
    pub fn get_request(&self, id: &str) -> Option<&SigningRequest> {
        self.pending_requests.iter().find(|r| r.id == id)
    }

    /// 获取可变的特定请求
    pub fn get_request_mut(&mut self, id: &str) -> Option<&mut SigningRequest> {
        self.pending_requests.iter_mut().find(|r| r.id == id)
    }

    /// 处理请求（批准或拒绝）
    pub fn process_request(&mut self, id: &str, approve: bool) -> Result<()> {
        let index = self
            .pending_requests
            .iter()
            .position(|r| r.id == id)
            .ok_or_else(|| anyhow!("Request not found"))?;

        let mut request = self.pending_requests.remove(index);
        
        if approve {
            request.approve();
        } else {
            request.reject();
        }
        
        self.history.push(request);
        Ok(())
    }

    /// 获取历史记录
    pub fn get_history(&self) -> &[SigningRequest] {
        &self.history
    }

    /// 清理过期的请求（超过5分钟）
    pub fn cleanup_expired(&mut self) {
        let now = chrono::Utc::now();
        let mut i = 0;
        while i < self.pending_requests.len() {
            if now.signed_duration_since(self.pending_requests[i].created_at).num_minutes() > 5 {
                let mut request = self.pending_requests.remove(i);
                request.mark_failed("Request expired".to_string());
                self.history.push(request);
            } else {
                i += 1;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_signing_request_creation() {
        let source = SigningRequestSource::Internal {
            operation: "Transfer SOL".to_string(),
        };
        
        let transaction = Transaction::default();
        let request = SigningRequest::new(source, transaction, Some("Test transfer".to_string()));
        
        assert_eq!(request.status, SigningRequestStatus::Pending);
        assert!(request.message.is_some());
    }

    #[test]
    fn test_request_manager() {
        let mut manager = SigningRequestManager::new();
        
        let source = SigningRequestSource::DApp {
            name: "Test DApp".to_string(),
            url: "https://test.com".to_string(),
            icon: None,
        };
        
        let request = SigningRequest::new(source, Transaction::default(), None);
        let id = manager.add_request(request);
        
        assert_eq!(manager.get_pending_requests().len(), 1);
        
        // 批准请求
        manager.process_request(&id, true).unwrap();
        
        assert_eq!(manager.get_pending_requests().len(), 0);
        assert_eq!(manager.get_history().len(), 1);
        assert_eq!(manager.get_history()[0].status, SigningRequestStatus::Approved);
    }
}