# Solana 钱包技术架构设计 - 基于 GPUI Component

## 1. 项目概述

### 1.1 项目目标
构建一个高性能、安全、用户友好的桌面端 Solana 钱包应用，利用 GPUI 框架的高性能渲染能力和 gpui-component 提供的丰富 UI 组件。

### 1.2 技术栈选择
- **编程语言**: Rust
- **UI 框架**: GPUI + gpui-component
- **区块链交互**: solana-sdk, solana-client
- **密码学库**: ed25519-dalek, aes-gcm, argon2
- **数据存储**: SQLite (通过 sqlx) + 本地加密文件存储
- **网络通信**: tokio + reqwest
- **序列化**: serde + bincode

## 2. 系统架构

### 2.1 分层架构设计

```
┌─────────────────────────────────────────┐
│         Presentation Layer              │
│    (GPUI Components + Custom UI)        │
├─────────────────────────────────────────┤
│         Application Layer               │
│    (Business Logic & Controllers)       │
├─────────────────────────────────────────┤
│         Domain Layer                    │
│    (Core Wallet Logic & Models)         │
├─────────────────────────────────────────┤
│         Infrastructure Layer            │
│ (Blockchain, Storage, Cryptography)     │
└─────────────────────────────────────────┘
```

### 2.2 核心模块设计

#### 2.2.1 钱包核心模块 (wallet_core)
```rust
// 密钥对管理
pub struct Keypair {
    public_key: Pubkey,
    secret_key: SecretKey,
}

// 钱包账户
pub struct WalletAccount {
    id: Uuid,
    name: String,
    keypair: EncryptedKeypair,
    balance: Balance,
    tokens: Vec<TokenAccount>,
    creation_date: DateTime<Utc>,
}

// 助记词管理
pub struct MnemonicManager {
    word_list: Vec<String>,
    derivation_path: String,
}
```

#### 2.2.2 交易模块 (transaction)
```rust
pub struct TransactionBuilder {
    from: Pubkey,
    to: Pubkey,
    amount: u64,
    token_mint: Option<Pubkey>,
    fee_payer: Pubkey,
}

pub struct TransactionHistory {
    signature: Signature,
    status: TransactionStatus,
    timestamp: DateTime<Utc>,
    amount: u64,
    fee: u64,
}
```

#### 2.2.3 安全模块 (security)
```rust
// 主密码管理
pub struct MasterPassword {
    hash: ArgonHash,
    salt: [u8; 32],
}

// 密钥加密存储
pub struct KeyVault {
    encryption_key: DerivedKey,
    storage_path: PathBuf,
}

// 硬件钱包接口
pub trait HardwareWallet {
    async fn get_pubkey(&self) -> Result<Pubkey>;
    async fn sign_transaction(&self, tx: Transaction) -> Result<Signature>;
}
```

#### 2.2.4 RPC 通信模块 (rpc)
```rust
pub struct RpcClient {
    endpoints: Vec<String>,
    current_endpoint: usize,
    client: Client,
}

impl RpcClient {
    pub async fn get_balance(&self, pubkey: &Pubkey) -> Result<u64>;
    pub async fn send_transaction(&self, tx: &Transaction) -> Result<Signature>;
    pub async fn get_token_accounts(&self, owner: &Pubkey) -> Result<Vec<TokenAccount>>;
}
```

## 3. UI/UX 设计

### 3.1 页面结构
```
主窗口 (MainWindow)
├── 侧边栏 (Sidebar) - 使用 Dock 组件
│   ├── 账户列表 (AccountList)
│   ├── 导航菜单 (Navigation)
│   └── 设置入口 (Settings)
├── 主内容区 (ContentArea)
│   ├── 仪表板 (Dashboard)
│   ├── 发送/接收 (SendReceive)
│   ├── 交易历史 (TransactionHistory)
│   ├── Token 管理 (TokenManager)
│   └── Staking 界面 (StakingView)
└── 状态栏 (StatusBar)
```

### 3.2 关键 UI 组件映射

- **账户管理**: 使用 `List` + `Avatar` + `Badge` 组件
- **余额显示**: 使用 `Card` + `Typography` 组件
- **交易表单**: 使用 `Form` + `Input` + `Select` 组件
- **交易历史**: 使用 `Table` (虚拟化) + `Pagination` 组件
- **图表展示**: 使用内置的 Chart 组件
- **通知提醒**: 使用 `Toast` + `Alert` 组件
- **加载状态**: 使用 `Spinner` + `Skeleton` 组件

## 4. 数据流设计

### 4.1 状态管理
```rust
// 全局应用状态
pub struct AppState {
    accounts: Vec<WalletAccount>,
    current_account: Option<Uuid>,
    network: SolanaNetwork,
    theme: Theme,
    notifications: VecDeque<Notification>,
}

// 使用 GPUI 的 Model 系统
impl Model for AppState {
    fn update(&mut self, cx: &mut ModelContext<Self>) {
        // 状态更新逻辑
    }
}
```

### 4.2 事件系统
```rust
// 定义事件类型
pub enum WalletEvent {
    AccountCreated(WalletAccount),
    TransactionSent(TransactionResult),
    BalanceUpdated(Pubkey, u64),
    NetworkChanged(SolanaNetwork),
}

// 事件处理
impl EventHandler<WalletEvent> for MainWindow {
    fn handle_event(&mut self, event: &WalletEvent, cx: &mut ViewContext<Self>) {
        match event {
            WalletEvent::TransactionSent(result) => {
                self.show_transaction_result(result, cx);
            }
            // 其他事件处理
        }
    }
}
```

## 5. 安全架构

### 5.1 密钥安全
- **密钥派生**: 使用 BIP39/BIP44 标准
- **加密存储**: AES-256-GCM 加密私钥
- **内存安全**: 使用 `zeroize` 清理敏感数据
- **硬件钱包**: 支持 Ledger 集成

### 5.2 通信安全
- **RPC 通信**: HTTPS only
- **本地存储**: SQLCipher 加密数据库
- **会话管理**: 自动锁定和超时机制

### 5.3 安全特性
```rust
// 自动锁定
pub struct AutoLock {
    timeout: Duration,
    last_activity: Instant,
}

// 交易确认
pub struct TransactionConfirmation {
    simulation_result: SimulationResult,
    fee_estimate: u64,
    warnings: Vec<SecurityWarning>,
}
```

## 6. 性能优化

### 6.1 渲染优化
- 利用 GPUI 的增量渲染
- 使用虚拟化列表显示大量交易
- 懒加载 Token 元数据

### 6.2 数据优化
- 本地缓存账户余额和交易历史
- 批量查询 RPC 接口
- WebSocket 订阅实时更新

## 7. 项目结构

```
solana-wallet/
├── Cargo.toml
├── src/
│   ├── main.rs
│   ├── app/
│   │   ├── mod.rs
│   │   ├── state.rs
│   │   └── events.rs
│   ├── ui/
│   │   ├── mod.rs
│   │   ├── components/
│   │   ├── views/
│   │   └── theme.rs
│   ├── wallet/
│   │   ├── mod.rs
│   │   ├── account.rs
│   │   ├── keypair.rs
│   │   └── transaction.rs
│   ├── security/
│   │   ├── mod.rs
│   │   ├── encryption.rs
│   │   └── vault.rs
│   ├── rpc/
│   │   ├── mod.rs
│   │   └── client.rs
│   └── storage/
│       ├── mod.rs
│       └── database.rs
├── assets/
│   ├── icons/
│   └── fonts/
└── tests/
```

## 8. 依赖配置

```toml
[dependencies]
# UI Framework
gpui = { git = "https://github.com/zed-industries/zed" }
gpui-component = { git = "https://github.com/longbridge/gpui-component" }

# Solana
solana-sdk = "1.18"
solana-client = "1.18"
spl-token = "4.0"

# Cryptography
ed25519-dalek = "2.1"
aes-gcm = "0.10"
argon2 = "0.5"
zeroize = "1.7"

# Storage
sqlx = { version = "0.7", features = ["sqlite", "runtime-tokio"] }

# Async
tokio = { version = "1", features = ["full"] }
futures = "0.3"

# Serialization
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
bincode = "1.3"

# Utils
uuid = { version = "1.6", features = ["v4", "serde"] }
chrono = { version = "0.4", features = ["serde"] }
anyhow = "1.0"
```

## 9. 开发计划

### Phase 1: 基础框架搭建
1. 初始化项目结构
2. 集成 GPUI 和 gpui-component
3. 实现基础 UI 布局

### Phase 2: 核心功能开发
1. 钱包创建和导入
2. 账户管理
3. 余额查询
4. 基础转账功能

### Phase 3: 高级功能
1. Token 管理
2. 交易历史
3. Staking 功能
4. DeFi 集成

### Phase 4: 安全和优化
1. 硬件钱包支持
2. 性能优化
3. 安全审计
4. 多语言支持

## 10. 测试策略

### 10.1 单元测试
- 密钥生成和加密
- 交易构建
- 签名验证

### 10.2 集成测试
- RPC 通信
- 数据库操作
- UI 交互流程

### 10.3 安全测试
- 密钥安全性
- 加密强度
- 内存泄漏检测