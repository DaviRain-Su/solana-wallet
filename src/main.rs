use gpui::*;
use gpui_component::button::{Button, ButtonVariants};
use std::sync::Arc;

mod wallet;
mod app;
mod theme;

use wallet::{
    generate_mnemonic, MnemonicPhrase, WalletAccount, WalletStorage, 
    WalletData, AccountData, RpcManager, SolanaNetwork
};
use theme::{Theme, ThemeMode};

actions!(wallet, [Quit, CreateWallet, ImportWallet]);

enum ViewState {
    Welcome,
    CreateWallet { 
        mnemonic: Option<MnemonicPhrase>,
        step: CreateWalletStep,
    },
    ImportWallet,
    Dashboard { account_index: usize },
}

#[derive(Clone, PartialEq)]
enum CreateWalletStep {
    ShowMnemonic,
    SetPassword,
}

struct MainWindow {
    view_state: ViewState,
    accounts: Vec<WalletAccount>,
    wallet_name: SharedString,
    password: SharedString,
    confirm_password: SharedString,
    storage: Option<WalletStorage>,
    rpc_manager: Arc<RpcManager>,
    balance: Option<f64>,
    loading_balance: bool,
    theme: Theme,
}

impl MainWindow {
    fn new(_window: &mut Window, _cx: &mut Context<Self>) -> Self {
        println!("Creating MainWindow...");
        
        let storage = WalletStorage::default_path()
            .ok()
            .and_then(|path| {
                println!("Storage path: {:?}", path);
                WalletStorage::new(path).ok()
            });
        
        let rpc_manager = Arc::new(RpcManager::new(SolanaNetwork::Devnet));
        println!("RPC manager created for Devnet");
        
        Self {
            view_state: ViewState::Welcome,
            accounts: Vec::new(),
            wallet_name: SharedString::default(),
            password: SharedString::default(),
            confirm_password: SharedString::default(),
            storage,
            rpc_manager,
            balance: None,
            loading_balance: false,
            theme: Theme::dark(),
        }
    }

    fn create_wallet(&mut self, _cx: &mut Context<Self>) {
        match generate_mnemonic(12) {
            Ok(mnemonic) => {
                self.view_state = ViewState::CreateWallet {
                    mnemonic: Some(mnemonic),
                    step: CreateWalletStep::ShowMnemonic,
                };
            }
            Err(e) => {
                println!("Failed to generate mnemonic: {}", e);
            }
        }
    }

    fn import_wallet(&mut self, _cx: &mut Context<Self>) {
        self.view_state = ViewState::ImportWallet;
    }
    
    fn save_wallet(&mut self, cx: &mut Context<Self>) {
        // 验证输入
        if self.wallet_name.is_empty() {
            println!("钱包名称不能为空");
            return;
        }
        
        if self.password.is_empty() {
            println!("密码不能为空");
            return;
        }
        
        if self.password != self.confirm_password {
            println!("两次输入的密码不一致");
            return;
        }
        
        if let ViewState::CreateWallet { mnemonic: Some(ref mnemonic), .. } = &self.view_state {
            if let Some(ref storage) = self.storage {
                // 创建钱包数据
                let mut wallet_data = WalletData {
                    mnemonic: mnemonic.phrase(),
                    accounts: vec![],
                    created_at: chrono::Utc::now(),
                    modified_at: chrono::Utc::now(),
                };
                
                // 派生第一个账户
                match mnemonic.derive_keypair(0) {
                    Ok(derived) => {
                        let account_data = AccountData {
                            name: "账户 1".to_string(),
                            derivation_path: derived.derivation_path.clone(),
                            pubkey: derived.keypair.pubkey().to_string(),
                        };
                        wallet_data.accounts.push(account_data);
                        
                        // 保存钱包
                        match storage.save_wallet(&self.wallet_name, &wallet_data, &self.password) {
                            Ok(_) => {
                                // 创建内存中的账户
                                let account = WalletAccount::with_derivation_path(
                                    "账户 1".to_string(),
                                    derived.keypair,
                                    derived.derivation_path,
                                );
                                self.accounts.push(account);
                                
                                // 清空密码
                                self.password = SharedString::default();
                                self.confirm_password = SharedString::default();
                                
                                // 跳转到仪表板
                                self.view_state = ViewState::Dashboard { account_index: 0 };
                                // 获取余额
                                self.fetch_balance(0, cx);
                                cx.notify();
                            }
                            Err(e) => {
                                println!("保存钱包失败: {}", e);
                            }
                        }
                    }
                    Err(e) => {
                        println!("派生密钥失败: {}", e);
                    }
                }
            } else {
                println!("存储未初始化");
            }
        }
    }
    
    fn toggle_theme(&mut self, cx: &mut Context<Self>) {
        self.theme = match self.theme.mode {
            ThemeMode::Light => Theme::dark(),
            ThemeMode::Dark => Theme::light(),
        };
        cx.notify();
    }

    fn fetch_balance(&mut self, account_index: usize, _cx: &mut Context<Self>) {
        if let Some(account) = self.accounts.get(account_index) {
            let pubkey = account.pubkey;
            let rpc = self.rpc_manager.clone();
            
            self.loading_balance = true;
            self.balance = None;
            
            // 使用 std::thread 来运行异步任务，避免类型推断问题
            std::thread::spawn(move || {
                let rt = tokio::runtime::Runtime::new().unwrap();
                let balance_result = rt.block_on(async {
                    rpc.get_balance_in_sol(&pubkey).await
                });
                
                match balance_result {
                    Ok(balance) => {
                        println!("获取余额成功: {} SOL", balance);
                        // TODO: 需要一种方式来更新UI
                    }
                    Err(e) => {
                        println!("获取余额失败: {}", e);
                    }
                }
            });
            
            // 暂时使用假数据来测试UI
            self.balance = Some(0.0);
            self.loading_balance = false;
        }
    }
}

impl Render for MainWindow {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // Force window to front
        _window.activate_window();
        
        div()
            .flex()
            .flex_col()
            .size_full()
            .bg(self.theme.background)
            .child(
                // 顶部工具栏
                div()
                    .flex()
                    .w_full()
                    .h(px(50.0))
                    .px(px(20.0))
                    .items_center()
                    .justify_end()
                    .bg(self.theme.surface)
                    .border_b_1()
                    .border_color(self.theme.border)
                    .child(
                        Button::new("theme-toggle")
                            .label(if self.theme.mode == ThemeMode::Dark { "🌞" } else { "🌙" })
                            .ghost()
                            .on_click(cx.listener(|this, _, _window, cx| {
                                this.toggle_theme(cx);
                            }))
                    )
            )
            .child(
                // 主内容区域
                div()
                    .flex()
                    .flex_1()
                    .w_full()
                    .overflow_hidden()
                    .child(
                        match &self.view_state {
                            ViewState::Welcome => div().size_full().child(self.render_welcome_content(cx)),
                            ViewState::CreateWallet { mnemonic, step } => {
                                match step {
                                    CreateWalletStep::ShowMnemonic => {
                                        div().size_full().child(self.render_mnemonic_content(mnemonic, cx))
                                    }
                                    CreateWalletStep::SetPassword => {
                                        div().size_full().child(self.render_password_content(mnemonic, cx))
                                    }
                                }
                            }
                            ViewState::ImportWallet => div().size_full().child(self.render_import_wallet_content(cx)),
                            ViewState::Dashboard { account_index } => {
                                if let Some(account) = self.accounts.get(*account_index) {
                                    div().size_full().child(self.render_dashboard_content(account, cx))
                                } else {
                                    div().size_full().child(self.render_welcome_content(cx))
                                }
                            }
                        }
                    )
            )
    }
}

impl MainWindow {
    fn render_welcome_content(&self, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex()
            .flex_col()
            .size_full()
            .items_center()
            .justify_center()
            .gap_8()
            .child(
                div()
                    .flex()
                    .flex_col()
                    .items_center()
                    .gap_4()
                    .child(
                        div()
                            .text_3xl()
                            .text_color(self.theme.text_primary)
                            .child("Solana Wallet")
                    )
                    .child(
                        div()
                            .text_lg()
                            .text_color(self.theme.text_secondary)
                            .child("基于 GPUI 的高性能桌面钱包")
                    )
            )
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_4()
                    .w_full()
                    .max_w(px(300.0))
                    .child(
                        Button::new("create-wallet")
                            .label("创建新钱包")
                            .primary()
                            .on_click(cx.listener(|this, _, _window, cx| {
                                this.create_wallet(cx);
                                cx.notify();
                            }))
                    )
                    .child(
                        Button::new("import-wallet")
                            .label("导入已有钱包")
                            .ghost()
                            .on_click(cx.listener(|this, _, _window, cx| {
                                this.import_wallet(cx);
                                cx.notify();
                            }))
                    )
            )
    }

    fn render_mnemonic_content(&self, mnemonic: &Option<MnemonicPhrase>, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex()
            .flex_col()
            .size_full()
            .items_center()
            .justify_center()
            .gap_6()
            .p(px(20.0))
            .child(
                div()
                    .text_2xl()
                    .text_color(self.theme.text_primary)
                    .child("创建新钱包")
            )
            .child(
                div()
                    .text_color(self.theme.text_secondary)
                    .text_center()
                    .max_w(px(600.0))
                    .child("请妥善保存您的助记词，这是恢复钱包的唯一方式")
            )
            .child(
                if let Some(mnemonic) = mnemonic {
                    div()
                        .flex()
                        .flex_col()
                        .gap_4()
                        .p(px(20.0))
                        .bg(self.theme.surface)
                        .rounded(px(8.0))
                        .max_w(px(600.0))
                        .child(
                            div()
                                .flex()
                                .flex_wrap()
                                .gap_3()
                                .children(
                                    mnemonic.words()
                                        .into_iter()
                                        .enumerate()
                                        .map(|(i, word)| {
                                            div()
                                                .flex()
                                                .gap_2()
                                                .child(
                                                    div()
                                                        .text_sm()
                                                        .text_color(self.theme.text_disabled)
                                                        .child(format!("{}.", i + 1))
                                                )
                                                .child(
                                                    div()
                                                        .text_color(self.theme.text_primary)
                                                        .child(word)
                                                )
                                        })
                                )
                        )
                } else {
                    div().text_color(self.theme.text_secondary).child("生成助记词中...")
                }
            )
            .child(
                div()
                    .flex()
                    .gap_4()
                    .child(
                        Button::new("back")
                            .label("返回")
                            .ghost()
                            .on_click(cx.listener(|this, _, _window, cx| {
                                this.view_state = ViewState::Welcome;
                                cx.notify();
                            }))
                    )
                    .child(
                        Button::new("continue")
                            .label("我已保存助记词")
                            .primary()
                            .on_click(cx.listener(|this, _, _window, cx| {
                                if let ViewState::CreateWallet { mnemonic, .. } = &this.view_state {
                                    this.view_state = ViewState::CreateWallet {
                                        mnemonic: mnemonic.clone(),
                                        step: CreateWalletStep::SetPassword,
                                    };
                                    cx.notify();
                                }
                            }))
                    )
            )
    }

    fn render_password_content(&self, mnemonic: &Option<MnemonicPhrase>, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex()
            .flex_col()
            .size_full()
            .items_center()
            .justify_center()
            .gap_6()
            .p(px(20.0))
            .child(
                div()
                    .text_2xl()
                    .text_color(self.theme.text_primary)
                    .child("钱包创建成功")
            )
            .child(
                div()
                    .text_color(self.theme.success)
                    .child("✓ 您的钱包已经创建成功！")
            )
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_2()
                    .p(px(20.0))
                    .bg(self.theme.surface)
                    .rounded(px(8.0))
                    .max_w(px(400.0))
                    .child(
                        div()
                            .text_color(self.theme.text_secondary)
                            .child("为了演示，我们使用默认设置：")
                    )
                    .child(
                        div()
                            .text_sm()
                            .text_color(self.theme.text_primary)
                            .child("钱包名称: 我的钱包")
                    )
                    .child(
                        div()
                            .text_sm()
                            .text_color(self.theme.text_primary)
                            .child("密码: (已加密存储)")
                    )
            )
            .child(
                Button::new("continue-to-dashboard")
                    .label("进入钱包")
                    .primary()
                    .on_click(cx.listener(|this, _, _window, cx| {
                        // 使用默认值保存钱包
                        this.wallet_name = "我的钱包".into();
                        this.password = "password123".into();
                        this.confirm_password = "password123".into();
                        this.save_wallet(cx);
                    }))
            )
    }

    fn render_import_wallet_content(&self, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex()
            .flex_col()
            .size_full()
            .items_center()
            .justify_center()
            .gap_4()
            .child(
                div()
                    .text_2xl()
                    .text_color(self.theme.text_primary)
                    .child("导入钱包")
            )
            .child(
                div()
                    .text_color(self.theme.text_secondary)
                    .child("功能开发中...")
            )
            .child(
                Button::new("back")
                    .label("返回")
                    .ghost()
                    .on_click(cx.listener(|this, _, _window, cx| {
                        this.view_state = ViewState::Welcome;
                        cx.notify();
                    }))
            )
    }

    fn render_dashboard_content(&self, account: &WalletAccount, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex()
            .flex_col()
            .size_full()
            .p(px(20.0))
            .gap_4()
            .child(
                // 头部
                div()
                    .flex()
                    .w_full()
                    .items_center()
                    .justify_between()
                    .child(
                        div()
                            .text_2xl()
                            .text_color(self.theme.text_primary)
                            .child("钱包仪表板")
                    )
                    .child(
                        div()
                            .flex()
                            .gap_2()
                            .items_center()
                            .child(
                                div()
                                    .text_sm()
                                    .text_color(self.theme.text_secondary)
                                    .child("网络:")
                            )
                            .child(
                                div()
                                    .text_color(self.theme.success)
                                    .child("Devnet")
                            )
                    )
            )
            .child(
                // 账户信息卡片
                div()
                    .flex()
                    .flex_col()
                    .w_full()
                    .gap_4()
                    .p(px(24.0))
                    .bg(self.theme.surface)
                    .rounded(px(12.0))
                    .border_1()
                    .border_color(self.theme.border)
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .justify_between()
                            .child(
                                div()
                                    .text_lg()
                                    .text_color(self.theme.text_primary)
                                    .child(account.name.clone())
                            )
                            .child(
                                Button::new("copy-address")
                                    .label("复制地址")
                                    .ghost()
                                    .on_click(cx.listener(move |_, _, _window, _cx| {
                                        // TODO: 实现复制功能
                                        println!("复制地址");
                                    }))
                            )
                    )
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap_2()
                            .child(
                                div()
                                    .text_sm()
                                    .text_color(self.theme.text_secondary)
                                    .child("地址:")
                            )
                            .child(
                                div()
                                    .text_sm()
                                    .text_color(self.theme.text_primary)
                                    .truncate()
                                    .child(account.pubkey.to_string())
                            )
                    )
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap_2()
                            .mt(px(12.0))
                            .child(
                                div()
                                    .text_sm()
                                    .text_color(self.theme.text_secondary)
                                    .child("余额:")
                            )
                            .child(
                                if self.loading_balance {
                                    div()
                                        .text_2xl()
                                        .text_color(self.theme.text_primary)
                                        .child("加载中...")
                                } else if let Some(balance) = self.balance {
                                    div()
                                        .flex()
                                        .items_baseline()
                                        .gap_2()
                                        .child(
                                            div()
                                                .text_2xl()
                                                .text_color(self.theme.text_primary)
                                                .child(format!("{:.6}", balance))
                                        )
                                        .child(
                                            div()
                                                .text_lg()
                                                .text_color(self.theme.text_secondary)
                                                .child("SOL")
                                        )
                                } else {
                                    div()
                                        .text_2xl()
                                        .text_color(self.theme.error)
                                        .child("获取失败")
                                }
                            )
                    )
            )
            .child(
                // 操作按钮
                div()
                    .flex()
                    .flex_wrap()
                    .gap_3()
                    .w_full()
                    .child(
                        Button::new("send")
                            .label("发送")
                            .primary()
                            .on_click(cx.listener(|_, _, _window, _cx| {
                                println!("发送功能待实现");
                            }))
                    )
                    .child(
                        Button::new("receive")
                            .label("接收")
                            .ghost()
                            .on_click(cx.listener(|_, _, _window, _cx| {
                                println!("接收功能待实现");
                            }))
                    )
                    .child(
                        Button::new("refresh")
                            .label("刷新余额")
                            .ghost()
                            .on_click(cx.listener(|this, _, _window, cx| {
                                if let ViewState::Dashboard { account_index } = this.view_state {
                                    this.fetch_balance(account_index, cx);
                                }
                            }))
                    )
            )
            .child(
                // 交易历史占位
                div()
                    .flex()
                    .flex_col()
                    .w_full()
                    .mt(px(20.0))
                    .gap_4()
                    .child(
                        div()
                            .text_lg()
                            .text_color(self.theme.text_primary)
                            .child("交易历史")
                    )
                    .child(
                        div()
                            .flex()
                            .w_full()
                            .h(px(200.0))
                            .items_center()
                            .justify_center()
                            .bg(self.theme.surface)
                            .rounded(px(8.0))
                            .border_1()
                            .border_color(self.theme.border)
                            .child(
                                div()
                                    .text_color(self.theme.text_disabled)
                                    .child("暂无交易记录")
                            )
                    )
            )
    }
}

fn main() {
    println!("Starting Solana Wallet...");
    
    let app = Application::new();
    
    app.run(move |cx: &mut App| {
        println!("Initializing application...");
        
        // Initialize theme
        gpui_component::init(cx);
        
        // Handle quit action
        cx.on_action(|_: &Quit, cx| {
            println!("Quitting application...");
            cx.quit();
        });
        
        cx.activate(true);
        
        let window_options = WindowOptions {
            window_bounds: Some(WindowBounds::Windowed(Bounds {
                origin: point(px(100.0), px(100.0)),
                size: size(px(1000.0), px(700.0)),
            })),
            window_min_size: Some(size(px(600.0), px(400.0))),
            titlebar: Some(TitlebarOptions {
                title: Some("GPUI Solana Wallet".into()),
                ..Default::default()
            }),
            kind: WindowKind::Normal,
            is_movable: true,
            focus: true,
            show: true,
            ..Default::default()
        };
        
        println!("Opening window...");
        let window_handle = cx.open_window(window_options, |window, cx| {
            window.activate_window();
            window.set_window_title("GPUI Solana Wallet");
            cx.new(|cx| MainWindow::new(window, cx))
        })
        .unwrap();
        
        // Ensure the window is visible
        window_handle.update(cx, |_, window, _| {
            window.activate_window();
        }).unwrap();
        
        println!("Window opened successfully!");
    });
}