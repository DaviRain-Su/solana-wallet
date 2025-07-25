use gpui::*;
use gpui_component::button::{Button, ButtonVariants};

mod wallet;
mod app;

use wallet::{generate_mnemonic, MnemonicPhrase, WalletAccount, WalletStorage, WalletData, AccountData};

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
}

impl MainWindow {
    fn new(_window: &mut Window, _cx: &mut Context<Self>) -> Self {
        let storage = WalletStorage::default_path()
            .ok()
            .and_then(|path| WalletStorage::new(path).ok());
        
        Self {
            view_state: ViewState::Welcome,
            accounts: Vec::new(),
            wallet_name: SharedString::default(),
            password: SharedString::default(),
            confirm_password: SharedString::default(),
            storage,
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
}

impl Render for MainWindow {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // Force window to front
        _window.activate_window();
        
        div()
            .flex()
            .size_full()
            .bg(rgb(0x1e1e1e))
            .child(
                match &self.view_state {
                    ViewState::Welcome => div().child(self.render_welcome_content(cx)),
                    ViewState::CreateWallet { mnemonic, step } => {
                        match step {
                            CreateWalletStep::ShowMnemonic => {
                                div().child(self.render_mnemonic_content(mnemonic, cx))
                            }
                            CreateWalletStep::SetPassword => {
                                div().child(self.render_password_content(mnemonic, cx))
                            }
                        }
                    }
                    ViewState::ImportWallet => div().child(self.render_import_wallet_content(cx)),
                    ViewState::Dashboard { account_index } => {
                        if let Some(account) = self.accounts.get(*account_index) {
                            div().child(self.render_dashboard_content(account, cx))
                        } else {
                            div().child(self.render_welcome_content(cx))
                        }
                    }
                }
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
                            .text_color(rgb(0xffffff))
                            .child("Solana Wallet")
                    )
                    .child(
                        div()
                            .text_lg()
                            .text_color(rgb(0xaaaaaa))
                            .child("基于 GPUI 的高性能桌面钱包")
                    )
            )
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_4()
                    .w(px(300.0))
                    .child(
                        Button::new("create-wallet")
                            .label("创建新钱包")
                            .primary()
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.create_wallet(cx);
                                cx.notify();
                            }))
                    )
                    .child(
                        Button::new("import-wallet")
                            .label("导入已有钱包")
                            .ghost()
                            .on_click(cx.listener(|this, _, _, cx| {
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
            .p(px(40.0))
            .child(
                div()
                    .text_2xl()
                    .text_color(rgb(0xffffff))
                    .child("创建新钱包")
            )
            .child(
                div()
                    .text_color(rgb(0xaaaaaa))
                    .child("请妥善保存您的助记词，这是恢复钱包的唯一方式")
            )
            .child(
                if let Some(mnemonic) = mnemonic {
                    div()
                        .flex()
                        .flex_col()
                        .gap_4()
                        .p(px(20.0))
                        .bg(rgb(0x2a2a2a))
                        .rounded(px(8.0))
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
                                                        .text_color(rgb(0x666666))
                                                        .child(format!("{}.", i + 1))
                                                )
                                                .child(
                                                    div()
                                                        .text_color(rgb(0xffffff))
                                                        .child(word)
                                                )
                                        })
                                )
                        )
                } else {
                    div().child("生成助记词中...")
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
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.view_state = ViewState::Welcome;
                                cx.notify();
                            }))
                    )
                    .child(
                        Button::new("continue")
                            .label("我已保存助记词")
                            .primary()
                            .on_click(cx.listener(|this, _, _, cx| {
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
            .p(px(40.0))
            .child(
                div()
                    .text_2xl()
                    .text_color(rgb(0xffffff))
                    .child("钱包创建成功")
            )
            .child(
                div()
                    .text_color(rgb(0x00ff00))
                    .child("✓ 您的钱包已经创建成功！")
            )
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_2()
                    .p(px(20.0))
                    .bg(rgb(0x2a2a2a))
                    .rounded(px(8.0))
                    .child(
                        div()
                            .text_color(rgb(0xaaaaaa))
                            .child("为了演示，我们使用默认设置：")
                    )
                    .child(
                        div()
                            .text_sm()
                            .text_color(rgb(0xcccccc))
                            .child("钱包名称: 我的钱包")
                    )
                    .child(
                        div()
                            .text_sm()
                            .text_color(rgb(0xcccccc))
                            .child("密码: (已加密存储)")
                    )
            )
            .child(
                Button::new("continue-to-dashboard")
                    .label("进入钱包")
                    .primary()
                    .on_click(cx.listener(|this, _, _, cx| {
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
                    .text_color(rgb(0xffffff))
                    .child("导入钱包")
            )
            .child(
                div()
                    .text_color(rgb(0xaaaaaa))
                    .child("功能开发中...")
            )
            .child(
                Button::new("back")
                    .label("返回")
                    .ghost()
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.view_state = ViewState::Welcome;
                        cx.notify();
                    }))
            )
    }

    fn render_dashboard_content(&self, _account: &WalletAccount, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex()
            .flex_col()
            .size_full()
            .items_center()
            .justify_center()
            .child("Dashboard - TODO")
    }
}

fn main() {
    let app = Application::new();
    
    app.run(move |cx: &mut App| {
        // Initialize theme
        gpui_component::init(cx);
        
        // Handle quit action
        cx.on_action(|_: &Quit, cx| {
            cx.quit();
        });
        
        cx.activate(true);
        
        let window_options = WindowOptions {
            window_bounds: Some(WindowBounds::Windowed(Bounds {
                origin: point(px(200.0), px(200.0)),
                size: size(px(800.0), px(600.0)),
            })),
            titlebar: Some(TitlebarOptions {
                title: Some("GPUI Solana Wallet".into()),
                ..Default::default()
            }),
            ..Default::default()
        };
        
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
    });
}