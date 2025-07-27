# GPUI Solana Wallet

一个基于 GPUI（GPU 加速 UI 框架）构建的现代化 Solana 桌面钱包应用。

![Rust](https://img.shields.io/badge/rust-%23000000.svg?style=for-the-badge&logo=rust&logoColor=white)
![Solana](https://img.shields.io/badge/Solana-9945FF?style=for-the-badge&logo=solana&logoColor=white)

## 功能特性

### 🔐 钱包管理
- **创建钱包**：使用 BIP39 标准生成安全的 12 词助记词
- **导入钱包**：支持助记词和私钥导入
- **加密存储**：使用 AES-256-GCM 加密保护您的私钥
- **密钥派生**：使用 BIP44 标准派生路径管理密钥

### 💸 交易功能
- **发送 SOL**：简单直观的转账界面
- **接收 SOL**：生成二维码方便接收资金
- **交易历史**：查看所有交易记录和状态
- **余额查询**：实时显示账户余额

### 🌐 网络支持
- **多网络切换**：支持主网、测试网、开发网
- **自定义 RPC**：配置自定义 RPC 端点
- **空投功能**：在测试网络请求测试代币

### 🎨 用户体验
- **主题切换**：支持亮色/暗色主题
- **响应式设计**：窗口大小自适应
- **复制粘贴**：完整的剪贴板支持
- **实时更新**：自动刷新余额和交易状态

## 安装

### 系统要求
- Rust 1.70 或更高版本
- macOS、Linux 或 Windows

### 构建步骤

1. 克隆仓库：
```bash
git clone https://github.com/yourusername/davirain.git
cd davirain
```

2. 安装依赖并构建：
```bash
cargo build --release
```

3. 运行应用：
```bash
cargo run
```

## 使用指南

### 创建新钱包
1. 启动应用后，在欢迎界面点击"创建新钱包"
2. 系统将生成 12 个助记词，请妥善保存
3. 点击"我已保存助记词"继续
4. 输入密码保护您的钱包
5. 完成！您可以开始使用钱包了

### 导入现有钱包
1. 在欢迎界面点击"导入钱包"
2. 选择导入方式：
   - **助记词**：输入您的 12/24 词助记词
   - **私钥**：输入 Base58 格式的私钥
3. 设置密码并完成导入

### 发送交易
1. 在主界面点击"发送"按钮
2. 输入接收地址（支持粘贴）
3. 输入发送金额（支持小数）
4. 点击"发送交易"确认
5. 等待交易确认

### 接收资金
1. 点击"接收"按钮
2. 显示您的钱包地址和二维码
3. 点击"复制地址"或让对方扫描二维码

## 技术架构

### 核心依赖
- **[GPUI](https://github.com/zed-industries/gpui)** - GPU 加速的 UI 框架
- **[Solana SDK](https://github.com/solana-labs/solana)** - Solana 区块链交互
- **[tokio](https://tokio.rs/)** - 异步运行时
- **[gpui-component](https://github.com/longbridgeapp/gpui-component)** - GPUI 组件库

### 项目结构
```
src/
├── main.rs              # 主程序入口和 UI 实现
├── wallet/              # 钱包核心功能
│   ├── keypair.rs       # 密钥对管理
│   ├── mnemonic.rs      # 助记词处理
│   ├── account.rs       # 账户管理
│   ├── storage.rs       # 加密存储
│   ├── rpc.rs          # RPC 通信
│   └── transaction.rs   # 交易构建
├── theme/               # 主题系统
│   └── mod.rs          # 主题定义
└── app/                # 应用状态管理
```

### 安全特性
- 使用 Argon2 进行密码哈希
- AES-256-GCM 加密存储私钥
- 助记词生成使用密码学安全的随机数
- 所有敏感数据在内存中安全处理

## 开发指南

### 运行测试
```bash
cargo test
```

### 代码检查
```bash
cargo clippy
cargo fmt
```

### 调试模式
```bash
RUST_LOG=debug cargo run
```

## 路线图

- [x] 基础钱包功能
- [x] 交易发送和接收
- [x] 多网络支持
- [x] 主题系统
- [x] 交易历史
- [ ] 多账户管理
- [ ] SPL 代币支持
- [ ] 硬件钱包集成
- [ ] DApp 连接支持

## 贡献

欢迎贡献代码！请遵循以下步骤：

1. Fork 本仓库
2. 创建功能分支 (`git checkout -b feature/amazing-feature`)
3. 提交更改 (`git commit -m '添加某个功能'`)
4. 推送到分支 (`git push origin feature/amazing-feature`)
5. 创建 Pull Request

## 许可证

本项目采用 MIT 许可证 - 详见 [LICENSE](LICENSE) 文件

## 致谢

- [GPUI](https://github.com/zed-industries/gpui) 团队提供的优秀 UI 框架
- [Solana](https://solana.com) 提供的高性能区块链平台
- 所有贡献者和支持者

## 免责声明

本软件按"原样"提供，不提供任何形式的保证。使用本软件的风险由您自行承担。请妥善保管您的助记词和私钥，丢失将无法恢复。

---

**注意**：这是一个实验性项目，主要用于学习和演示目的。在生产环境使用前，请进行充分的安全审计。