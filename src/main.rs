use gpui::*;
use gpui_component::button::{Button, ButtonVariants};

mod wallet;
mod app;

use wallet::{generate_mnemonic, MnemonicPhrase, WalletAccount};

actions!(wallet, [Quit, CreateWallet, ImportWallet]);

enum ViewState {
    Welcome,
    CreateWallet { mnemonic: Option<MnemonicPhrase> },
    ImportWallet,
    Dashboard { account_index: usize },
}

struct MainWindow {
    view_state: ViewState,
    accounts: Vec<WalletAccount>,
}

impl MainWindow {
    fn new(_window: &mut Window, _cx: &mut Context<Self>) -> Self {
        Self {
            view_state: ViewState::Welcome,
            accounts: Vec::new(),
        }
    }

    fn create_wallet(&mut self, _cx: &mut Context<Self>) {
        match generate_mnemonic(12) {
            Ok(mnemonic) => {
                self.view_state = ViewState::CreateWallet {
                    mnemonic: Some(mnemonic),
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
                    ViewState::CreateWallet { mnemonic } => div().child(self.render_create_wallet_content(mnemonic, cx)),
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

    fn render_create_wallet_content(&self, mnemonic: &Option<MnemonicPhrase>, cx: &mut Context<Self>) -> impl IntoElement {
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
                                if let ViewState::CreateWallet { mnemonic: Some(ref mnemonic) } = &this.view_state {
                                    // 从助记词派生第一个账户
                                    match mnemonic.derive_keypair(0) {
                                        Ok(derived) => {
                                            let account = WalletAccount::with_derivation_path(
                                                "账户 1".to_string(),
                                                derived.keypair,
                                                derived.derivation_path,
                                            );
                                            this.accounts.push(account);
                                            this.view_state = ViewState::Dashboard { account_index: 0 };
                                            cx.notify();
                                        }
                                        Err(e) => {
                                            println!("Failed to derive keypair: {}", e);
                                        }
                                    }
                                }
                            }))
                    )
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