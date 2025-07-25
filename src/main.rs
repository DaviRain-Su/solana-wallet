use gpui::prelude::FluentBuilder;
use gpui::*;
use gpui_component::button::{Button, ButtonCustomVariant, ButtonVariants};
use gpui_component::{v_flex, Icon, IconName, Sizable};
use gpui::Timer;
use std::sync::Arc;
use std::str::FromStr;
mod app;
mod theme;
//mod ui;
mod wallet;

use theme::{Theme, ThemeMode};
use wallet::{
    generate_mnemonic, AccountData, MnemonicPhrase, RpcManager, SolanaNetwork, WalletAccount,
    WalletData, WalletKeypair, WalletStorage, TransactionRecord, TransactionStatus,
};

actions!(wallet, [Quit, CreateWallet, ImportWallet]);

#[derive(Clone, Copy, PartialEq, Debug)]
enum InputField {
    // Import fields
    Mnemonic,
    PrivateKey,
    WalletName,
    Password,
    ConfirmPassword,
    CustomRpcUrl,
    // Send fields
    SendToAddress,
    SendAmount,
}

// Backwards compatibility
type ImportField = InputField;
type SendField = InputField;

#[derive(Clone, Copy, PartialEq, Debug)]
enum ImportType {
    Mnemonic,
    PrivateKey,
}

#[derive(Clone, PartialEq)]
enum ViewState {
    Welcome,
    CreateWallet {
        mnemonic: Option<MnemonicPhrase>,
        step: CreateWalletStep,
    },
    ImportWallet,
    Dashboard {
        account_index: usize,
    },
    SendTransaction {
        account_index: usize,
    },
    ReceiveTransaction {
        account_index: usize,
    },
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
    current_network: SolanaNetwork,
    show_network_selector: bool,
    show_rpc_config: bool,
    custom_rpc_url: SharedString,
    requesting_airdrop: bool,
    pending_balance_update: Option<std::sync::mpsc::Receiver<Result<f64, anyhow::Error>>>,
    // 导入钱包相关字段
    import_type: ImportType,
    import_mnemonic: SharedString,
    import_private_key: SharedString,
    import_wallet_name: SharedString,
    import_password: SharedString,
    import_confirm_password: SharedString,
    import_error: Option<String>,
    import_focused_field: Option<InputField>,
    // 发送交易相关字段
    send_to_address: SharedString,
    send_amount: SharedString,
    send_error: Option<String>,
    sending_transaction: bool,
    pending_transaction: Option<std::sync::mpsc::Receiver<Result<(solana_sdk::signature::Signature, solana_sdk::pubkey::Pubkey, solana_sdk::pubkey::Pubkey, u64), String>>>,
    // 焦点处理
    focus_handle: FocusHandle,
    rpc_focused: bool,
    // 余额更新定时器
    balance_update_timer: Option<Timer>,
    // 复制成功提示
    show_copy_success: bool,
    copy_success_timer: Option<Timer>,
    // 交易历史记录
    transaction_history: Vec<wallet::TransactionRecord>,
}

fn is_password_field(field: InputField) -> bool {
    matches!(field, InputField::Password | InputField::ConfirmPassword)
}

impl MainWindow {
    fn wrap_button_with_theme(&self, button: Button, is_primary: bool, cx: &App) -> Button {
        // 根据当前主题和按钮类型创建自定义样式
        if is_primary {
            // 主按钮保持原样
            button
        } else if self.theme.mode == ThemeMode::Light {
            // 浅色主题下的 ghost 按钮需要深色文字
            let custom_style = ButtonCustomVariant::new(cx)
                .foreground(rgb(0x1a1a1a).into()) // 深色文字
                .color(rgb(0xf0f0f0).into()) // 浅灰背景
                .hover(rgb(0xe0e0e0).into()) // hover 时的背景
                .active(rgb(0xd0d0d0).into()) // 点击时的背景
                .border(self.theme.border.into());
            button.custom(custom_style)
        } else {
            // 深色主题保持原样
            button
        }
    }

    fn process_import_wallet(&mut self, cx: &mut Context<Self>) {
        // 清空错误
        self.import_error = None;

        // 验证通用字段
        if self.import_wallet_name.is_empty() {
            self.import_error = Some("请输入钱包名称".to_string());
            cx.notify();
            return;
        }

        if self.import_password.is_empty() {
            self.import_error = Some("请输入密码".to_string());
            cx.notify();
            return;
        }

        if self.import_password != self.import_confirm_password {
            self.import_error = Some("两次密码输入不一致".to_string());
            cx.notify();
            return;
        }

        if let Some(ref storage) = self.storage {
            match self.import_type {
                ImportType::Mnemonic => {
                    // 验证助记词
                    if self.import_mnemonic.is_empty() {
                        self.import_error = Some("请输入助记词".to_string());
                        cx.notify();
                        return;
                    }

                    match MnemonicPhrase::from_phrase(&self.import_mnemonic) {
                        Ok(mnemonic) => {
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
                                        name: "导入账户 1".to_string(),
                                        derivation_path: derived.derivation_path.clone(),
                                        pubkey: derived.keypair.pubkey().to_string(),
                                    };
                                    wallet_data.accounts.push(account_data);

                                    // 保存钱包
                                    match storage.save_wallet(
                                        &self.import_wallet_name,
                                        &wallet_data,
                                        &self.import_password,
                                    ) {
                                        Ok(_) => {
                                            // 创建内存中的账户
                                            let account = WalletAccount::with_derivation_path(
                                                "导入账户 1".to_string(),
                                                derived.keypair,
                                                derived.derivation_path,
                                            );
                                            self.accounts.push(account);

                                            // 跳转到仪表板
                                            self.view_state =
                                                ViewState::Dashboard { account_index: 0 };
                                            self.fetch_balance(0, cx);
                                            self.start_balance_update_timer(0, cx);
                                            cx.notify();
                                        }
                                        Err(e) => {
                                            self.import_error =
                                                Some(format!("保存钱包失败: {}", e));
                                            cx.notify();
                                        }
                                    }
                                }
                                Err(e) => {
                                    self.import_error = Some(format!("派生密钥失败: {}", e));
                                    cx.notify();
                                }
                            }
                        }
                        Err(e) => {
                            self.import_error = Some(format!("无效的助记词: {}", e));
                            cx.notify();
                        }
                    }
                }
                ImportType::PrivateKey => {
                    // 验证私钥
                    if self.import_private_key.is_empty() {
                        self.import_error = Some("请输入私钥".to_string());
                        cx.notify();
                        return;
                    }

                    // 尝试解析私钥 - 先清理空白字符
                    let cleaned_private_key = self.import_private_key.trim();
                    match WalletKeypair::from_base58_string(cleaned_private_key) {
                        Ok(wallet_keypair) => {
                            // 创建钱包数据（对于私钥导入，我们生成一个占位助记词）
                            let mut wallet_data = WalletData {
                                mnemonic: "IMPORTED_FROM_PRIVATE_KEY".to_string(),
                                accounts: vec![],
                                created_at: chrono::Utc::now(),
                                modified_at: chrono::Utc::now(),
                            };

                            let account_data = AccountData {
                                name: "导入账户".to_string(),
                                derivation_path: "m/imported".to_string(),
                                pubkey: wallet_keypair.pubkey().to_string(),
                            };
                            wallet_data.accounts.push(account_data);

                            // 保存钱包
                            match storage.save_wallet(
                                &self.import_wallet_name,
                                &wallet_data,
                                &self.import_password,
                            ) {
                                Ok(_) => {
                                    // 创建内存中的账户
                                    let account = WalletAccount::new(
                                        "导入账户".to_string(),
                                        wallet_keypair,
                                        true, // is_imported = true for private key import
                                    );
                                    self.accounts.push(account);

                                    // 跳转到仪表板
                                    self.view_state = ViewState::Dashboard { account_index: 0 };
                                    self.fetch_balance(0, cx);
                                    self.start_balance_update_timer(0, cx);
                                    cx.notify();
                                }
                                Err(e) => {
                                    self.import_error = Some(format!("保存钱包失败: {}", e));
                                    cx.notify();
                                }
                            }
                        }
                        Err(e) => {
                            self.import_error = Some(format!("无效的私钥: {}", e));
                            cx.notify();
                        }
                    }
                }
            }
        } else {
            self.import_error = Some("存储未初始化".to_string());
            cx.notify();
        }
    }

    fn render_input_field(
        &self,
        value: &SharedString,
        placeholder: &str,
        field: InputField,
        is_password: bool,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let is_focused = self.import_focused_field == Some(field);
        let border_color = if is_focused {
            self.theme.primary
        } else {
            self.theme.border
        };

        // Show cursor when focused
        let display_text = if value.is_empty() {
            placeholder.to_string()
        } else if is_password {
            "•".repeat(value.len())
        } else {
            value.to_string()
        };

        let display_with_cursor = if is_focused {
            format!("{}_", display_text)
        } else {
            display_text
        };

        div()
            .w_full()
            .h(px(40.0))
            .px(px(12.0))
            .bg(self.theme.surface)
            .rounded(px(8.0))
            .border_1()
            .border_color(border_color)
            .flex()
            .items_center()
            .cursor_text()
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(move |this, _, _, cx| {
                    this.import_focused_field = Some(field);
                    cx.notify();
                }),
            )
            .overflow_hidden()
            .child(
                div()
                    .flex_shrink()
                    .min_w_0()
                    .overflow_hidden()
                    .text_ellipsis()
                    .text_color(if value.is_empty() {
                        self.theme.text_disabled
                    } else {
                        self.theme.text_primary
                    })
                    .child(display_with_cursor),
            )
    }

    fn render_textarea_field(
        &self,
        value: &SharedString,
        placeholder: &str,
        field: ImportField,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let is_focused = self.import_focused_field == Some(field);
        let border_color = if is_focused {
            self.theme.primary
        } else {
            self.theme.border
        };

        // Show cursor when focused
        let display_text = if value.is_empty() {
            placeholder.to_string()
        } else {
            value.to_string()
        };

        let display_with_cursor = if is_focused {
            format!("{}_", display_text)
        } else {
            display_text
        };

        div()
            .w_full()
            .h(px(100.0))
            .p(px(12.0))
            .bg(self.theme.surface)
            .rounded(px(8.0))
            .border_1()
            .border_color(border_color)
            .cursor_text()
            .overflow_hidden()
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(move |this, _, _, cx| {
                    this.import_focused_field = Some(field);
                    cx.notify();
                }),
            )
            .overflow_hidden()
            .child(
                div()
                    .flex_shrink()
                    .min_w_0()
                    .overflow_hidden()
                    .text_ellipsis()
                    .text_color(if value.is_empty() {
                        self.theme.text_disabled
                    } else {
                        self.theme.text_primary
                    })
                    .child(display_with_cursor),
            )
    }

    fn handle_key_event(&mut self, event: &KeyDownEvent, cx: &mut Context<Self>) {
        // Handle RPC config dialog keyboard events
        if self.show_rpc_config && self.rpc_focused {
            self.handle_rpc_key_event(event, cx);
            return;
        }

        // Handle different views
        match &self.view_state {
            ViewState::ImportWallet => self.handle_import_key_event(event, cx),
            ViewState::SendTransaction { .. } => self.handle_send_key_event(event, cx),
            _ => {}
        }
    }

    fn handle_send_key_event(&mut self, event: &KeyDownEvent, cx: &mut Context<Self>) {
        if let Some(field) = self.import_focused_field {
            let keystroke = &event.keystroke;
            
            // Get the field value to modify
            let field_value = match field {
                InputField::SendToAddress => &mut self.send_to_address,
                InputField::SendAmount => &mut self.send_amount,
                _ => return, // Import fields handled elsewhere
            };

            // Check for copy/paste commands
            let is_cmd_or_ctrl = if cfg!(target_os = "macos") {
                keystroke.modifiers.platform
            } else {
                keystroke.modifiers.control
            };

            if is_cmd_or_ctrl {
                match keystroke.key.as_str() {
                    "c" => {
                        // Copy current field value to clipboard
                        if !field_value.is_empty() {
                            cx.write_to_clipboard(ClipboardItem::new_string(
                                field_value.to_string(),
                            ));
                        }
                        return;
                    }
                    "v" => {
                        // Paste from clipboard
                        if let Some(clipboard_text) = cx.read_from_clipboard() {
                            if let Some(text) = clipboard_text.text() {
                                *field_value = text.trim().to_string().into();
                                cx.notify();
                            }
                        }
                        return;
                    }
                    "a" => {
                        // Select all
                        return;
                    }
                    _ => {}
                }
            }

            // Handle different key inputs
            match keystroke.key.as_str() {
                "backspace" => {
                    let mut val = field_value.to_string();
                    val.pop();
                    *field_value = val.into();
                    cx.notify();
                }
                "tab" => {
                    // Move between address and amount fields
                    self.import_focused_field = Some(match field {
                        InputField::SendToAddress => InputField::SendAmount,
                        InputField::SendAmount => InputField::SendToAddress,
                        _ => field,
                    });
                    cx.notify();
                }
                "escape" => {
                    // Clear focus
                    self.import_focused_field = None;
                    cx.notify();
                }
                "enter" => {
                    // Submit transaction if on amount field
                    if field == InputField::SendAmount && !self.sending_transaction {
                        self.process_send_transaction(cx);
                    }
                }
                key => {
                    if key.len() == 1 {
                        let c = key.chars().next().unwrap();
                        
                        // For amount field, only allow numbers and decimal point
                        if field == InputField::SendAmount {
                            if c.is_numeric() || (c == '.' && !field_value.to_string().contains('.')) {
                                let mut val = field_value.to_string();
                                val.push(c);
                                *field_value = val.into();
                                cx.notify();
                            }
                        } else {
                            // For address field, allow all characters
                            let mut val = field_value.to_string();
                            val.push(c);
                            *field_value = val.into();
                            cx.notify();
                        }
                    }
                }
            }
        }
    }

    fn handle_import_key_event(&mut self, event: &KeyDownEvent, cx: &mut Context<Self>) {

        if let Some(field) = self.import_focused_field {
            let keystroke = &event.keystroke;
            // Get the field value to modify
            let field_value = match field {
                InputField::Mnemonic => &mut self.import_mnemonic,
                InputField::PrivateKey => &mut self.import_private_key,
                InputField::WalletName => &mut self.import_wallet_name,
                InputField::Password => &mut self.import_password,
                InputField::ConfirmPassword => &mut self.import_confirm_password,
                InputField::CustomRpcUrl => return, // Handled separately
                _ => return, // Send fields handled elsewhere
            };

            // Check for copy/paste commands
            let is_cmd_or_ctrl = if cfg!(target_os = "macos") {
                keystroke.modifiers.platform
            } else {
                keystroke.modifiers.control
            };

            if is_cmd_or_ctrl {
                match keystroke.key.as_str() {
                    "c" => {
                        // Copy current field value to clipboard
                        if !field_value.is_empty() && !is_password_field(field) {
                            cx.write_to_clipboard(ClipboardItem::new_string(
                                field_value.to_string(),
                            ));
                        }
                        return;
                    }
                    "v" => {
                        // Paste from clipboard
                        if let Some(clipboard_text) = cx.read_from_clipboard() {
                            if let Some(text) = clipboard_text.text() {
                                // Clean the text by trimming whitespace for private key field
                                let cleaned_text = if field == InputField::PrivateKey {
                                    text.trim().to_string()
                                } else {
                                    text.to_string()
                                };
                                *field_value = cleaned_text.into();
                                cx.notify();
                            }
                        }
                        return;
                    }
                    "a" => {
                        // Select all - we can't visually show selection, but we could store it for copy
                        // For now, just return
                        return;
                    }
                    _ => {}
                }
            }

            // Handle different key inputs
            match keystroke.key.as_str() {
                "backspace" => {
                    let mut val = field_value.to_string();
                    val.pop();
                    *field_value = val.into();
                    cx.notify();
                }
                "tab" => {
                    // Move to next field
                    self.import_focused_field = Some(match field {
                        InputField::Mnemonic => InputField::WalletName,
                        InputField::PrivateKey => InputField::WalletName,
                        InputField::WalletName => InputField::Password,
                        InputField::Password => InputField::ConfirmPassword,
                        InputField::ConfirmPassword => {
                            if self.import_type == ImportType::Mnemonic {
                                InputField::Mnemonic
                            } else {
                                InputField::PrivateKey
                            }
                        }
                        InputField::CustomRpcUrl => InputField::CustomRpcUrl,
                        _ => field, // Keep same field for others
                    });
                    cx.notify();
                }
                "escape" => {
                    // Clear focus
                    self.import_focused_field = None;
                    cx.notify();
                }
                "enter" => {
                    // Submit form if on last field
                    if field == InputField::ConfirmPassword {
                        self.process_import_wallet(cx);
                    }
                }
                key => {
                    // Handle regular character input
                    if key.len() == 1 && !key.chars().any(|c| c.is_control()) {
                        let new_val = format!("{}{}", field_value, key);
                        *field_value = new_val.into();
                        cx.notify();
                    } else if key == "space" {
                        // Handle space key
                        let new_val = format!("{} ", field_value);
                        *field_value = new_val.into();
                        cx.notify();
                    }
                }
            }
        }
    }

    fn new(_window: &mut Window, cx: &mut Context<Self>) -> Self {
        println!("Creating MainWindow...");

        let storage = WalletStorage::default_path().ok().and_then(|path| {
            println!("Storage path: {:?}", path);
            WalletStorage::new(path).ok()
        });

        let current_network = SolanaNetwork::Devnet;
        let rpc_manager = Arc::new(RpcManager::new(current_network.clone()));
        println!("RPC manager created for Devnet");

        let focus_handle = cx.focus_handle();

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
            current_network,
            show_network_selector: false,
            show_rpc_config: false,
            custom_rpc_url: SharedString::default(),
            requesting_airdrop: false,
            pending_balance_update: None,
            import_type: ImportType::Mnemonic,
            import_mnemonic: SharedString::default(),
            import_private_key: SharedString::default(),
            import_wallet_name: SharedString::default(),
            import_password: SharedString::default(),
            import_confirm_password: SharedString::default(),
            import_error: None,
            import_focused_field: None,
            send_to_address: SharedString::default(),
            send_amount: SharedString::default(),
            send_error: None,
            sending_transaction: false,
            pending_transaction: None,
            focus_handle,
            rpc_focused: false,
            balance_update_timer: None,
            show_copy_success: false,
            copy_success_timer: None,
            transaction_history: Vec::new(),
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
        self.import_focused_field = Some(if self.import_type == ImportType::Mnemonic {
            ImportField::Mnemonic
        } else {
            ImportField::PrivateKey
        });
        // Focus will be set when user clicks on an input field
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

        if let ViewState::CreateWallet {
            mnemonic: Some(ref mnemonic),
            ..
        } = &self.view_state
        {
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
                                // 获取余额并启动定时更新
                                self.fetch_balance(0, cx);
                                self.start_balance_update_timer(0, cx);
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

    fn toggle_network_selector(&mut self, cx: &mut Context<Self>) {
        self.show_network_selector = !self.show_network_selector;
        cx.notify();
    }

    fn show_rpc_config_dialog(&mut self, cx: &mut Context<Self>) {
        self.show_rpc_config = true;
        self.rpc_focused = true;
        // Set current RPC URL
        if let SolanaNetwork::Custom(url) = &self.current_network {
            self.custom_rpc_url = url.clone().into();
        } else {
            self.custom_rpc_url = self.current_network.rpc_url().into();
        }
        cx.notify();
    }

    fn apply_custom_rpc(&mut self, cx: &mut Context<Self>) {
        if !self.custom_rpc_url.is_empty() {
            let network = SolanaNetwork::Custom(self.custom_rpc_url.to_string());
            self.switch_network(network, cx);
            self.show_rpc_config = false;
        }
    }

    fn switch_network(&mut self, network: SolanaNetwork, cx: &mut Context<Self>) {
        self.current_network = network.clone();
        self.show_network_selector = false;

        // 切换RPC网络
        let rpc = self.rpc_manager.clone();
        std::thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                if let Err(e) = rpc.switch_network(network.clone()).await {
                    println!("切换网络失败: {}", e);
                } else {
                    println!("成功切换到网络: {}", network.name());
                }
            });
        });

        // 刷新余额
        if let ViewState::Dashboard { account_index } = self.view_state {
            self.fetch_balance(account_index, cx);
        }

        cx.notify();
    }

    fn request_airdrop(&mut self, cx: &mut Context<Self>) {
        if let ViewState::Dashboard { account_index } = self.view_state {
            if let Some(account) = self.accounts.get(account_index) {
                let pubkey = account.pubkey;
                let rpc = self.rpc_manager.clone();

                self.requesting_airdrop = true;
                cx.notify();

                std::thread::spawn(move || {
                    let rt = tokio::runtime::Runtime::new().unwrap();
                    rt.block_on(async {
                        // 请求 1 SOL 的空投
                        let result = rpc.request_airdrop(&pubkey, 1_000_000_000).await;

                        match &result {
                            Ok(signature) => {
                                println!("空投成功! 签名: {}", signature);
                            }
                            Err(e) => {
                                println!("空投失败: {}", e);
                            }
                        }
                    });
                });

                // 5秒后重置状态
                let account_idx = account_index;
                std::thread::spawn(|| {
                    std::thread::sleep(std::time::Duration::from_secs(5));
                    // 状态会在下次用户交互时重置
                });

                // 立即重置状态，让用户可以再次点击
                self.requesting_airdrop = false;
            }
        }
    }

    fn fetch_balance(&mut self, account_index: usize, cx: &mut Context<Self>) {
        if let Some(account) = self.accounts.get(account_index) {
            let pubkey = account.pubkey;
            let rpc = self.rpc_manager.clone();

            self.loading_balance = true;
            self.balance = None;
            cx.notify();

            // 创建通道来接收结果
            let (tx, rx) = std::sync::mpsc::channel();
            self.pending_balance_update = Some(rx);

            // 在后台线程中执行异步任务
            std::thread::spawn(move || {
                let rt = tokio::runtime::Runtime::new().unwrap();
                let balance_result = rt.block_on(async { rpc.get_balance_in_sol(&pubkey).await });

                match balance_result {
                    Ok(balance) => {
                        println!("获取余额成功: {} SOL", balance);
                    }
                    Err(ref e) => {
                        println!("获取余额失败: {}", e);
                    }
                }

                // 发送结果
                let _ = tx.send(balance_result);
            });
        }
    }

    fn start_balance_update_timer(&mut self, _account_index: usize, _cx: &mut Context<Self>) {
        // 停止现有的定时器
        self.stop_balance_update_timer();
        
        // For now, we'll skip the balance update timer implementation
        // This will be fixed in a future update
    }

    fn stop_balance_update_timer(&mut self) {
        if let Some(timer) = self.balance_update_timer.take() {
            let _ = timer;
        }
    }

    fn check_balance_update(&mut self, cx: &mut Context<Self>) {
        if let Some(rx) = &self.pending_balance_update {
            if let Ok(result) = rx.try_recv() {
                match result {
                    Ok(balance) => {
                        self.balance = Some(balance);
                    }
                    Err(_) => {
                        self.balance = None;
                    }
                }
                self.loading_balance = false;
                self.pending_balance_update = None;
                cx.notify();
            }
        }
    }
    
    fn check_transaction_update(&mut self, cx: &mut Context<Self>) {
        if let Some(rx) = &self.pending_transaction {
            if let Ok(result) = rx.try_recv() {
                self.pending_transaction = None;
                self.sending_transaction = false;
                
                match result {
                    Ok((signature, from_pubkey, to_pubkey, lamports)) => {
                        // 添加交易记录
                        let mut tx_record = TransactionRecord::new(
                            signature,
                            from_pubkey,
                            Some(to_pubkey),
                            lamports,
                            5000, // 估算的手续费
                        );
                        tx_record.status = TransactionStatus::Confirmed;
                        self.transaction_history.insert(0, tx_record);
                        
                        // 返回仪表板
                        if let ViewState::SendTransaction { account_index } = self.view_state {
                            self.view_state = ViewState::Dashboard { account_index };
                            // 清空输入
                            self.send_to_address = SharedString::default();
                            self.send_amount = SharedString::default();
                            // 刷新余额
                            self.fetch_balance(account_index, cx);
                        }
                    }
                    Err(error) => {
                        self.send_error = Some(error);
                    }
                }
                cx.notify();
            }
        }
    }
}

impl Focusable for MainWindow {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for MainWindow {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // Force window to front
        window.activate_window();

        // 检查余额更新
        self.check_balance_update(cx);
        // 检查交易更新
        self.check_transaction_update(cx);

        div()
            .flex()
            .flex_col()
            .key_context("ImportWallet")
            .track_focus(&self.focus_handle)
            .on_key_down(cx.listener(|this, event, _, cx| {
                this.handle_key_event(event, cx);
            }))
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
                        self.wrap_button_with_theme(
                            Button::new("theme-toggle")
                                .label(if self.theme.mode == ThemeMode::Dark {
                                    "🌞"
                                } else {
                                    "🌙"
                                })
                                .ghost()
                                .on_click(cx.listener(|this, _, _window, cx| {
                                    this.toggle_theme(cx);
                                })),
                            false,
                            cx,
                        ),
                    ),
            )
            .child(
                // 主内容区域
                div()
                    .flex()
                    .flex_1()
                    .w_full()
                    .overflow_hidden()
                    .child(match &self.view_state {
                        ViewState::Welcome => {
                            div().size_full().child(self.render_welcome_content(cx))
                        }
                        ViewState::CreateWallet { mnemonic, step } => match step {
                            CreateWalletStep::ShowMnemonic => div()
                                .size_full()
                                .child(self.render_mnemonic_content(mnemonic, cx)),
                            CreateWalletStep::SetPassword => div()
                                .size_full()
                                .child(self.render_password_content(mnemonic, cx)),
                        },
                        ViewState::ImportWallet => div()
                            .size_full()
                            .child(self.render_import_wallet_content(cx)),
                        ViewState::Dashboard { account_index } => {
                            if let Some(account) = self.accounts.get(*account_index) {
                                div()
                                    .size_full()
                                    .child(self.render_dashboard_content(account, cx))
                            } else {
                                div().size_full().child(self.render_welcome_content(cx))
                            }
                        }
                        ViewState::SendTransaction { account_index } => {
                            if let Some(account) = self.accounts.get(*account_index) {
                                div()
                                    .size_full()
                                    .child(self.render_send_transaction_content(account, cx))
                            } else {
                                div().size_full().child(self.render_welcome_content(cx))
                            }
                        }
                        ViewState::ReceiveTransaction { account_index } => {
                            if let Some(account) = self.accounts.get(*account_index) {
                                div()
                                    .size_full()
                                    .child(self.render_receive_transaction_content(account, cx))
                            } else {
                                div().size_full().child(self.render_welcome_content(cx))
                            }
                        }
                    }),
            )
            .when(self.show_rpc_config, |this| {
                this.child(self.render_rpc_config_dialog(cx))
            })
    }
}

impl MainWindow {
    fn render_rpc_input_field(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let border_color = if self.rpc_focused {
            self.theme.primary
        } else {
            self.theme.border
        };

        let display_text = if self.custom_rpc_url.is_empty() {
            SharedString::from("https://api.mainnet-beta.solana.com")
        } else {
            self.custom_rpc_url.clone()
        };

        div()
            .w_full()
            .px_3()
            .py_2()
            .bg(self.theme.surface)
            .border_1()
            .border_color(border_color)
            .rounded(px(6.0))
            .flex()
            .items_center()
            .cursor_text()
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(|this, _, _, cx| {
                    this.rpc_focused = true;
                    cx.notify();
                }),
            )
            .overflow_hidden()
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .overflow_hidden()
                    .text_ellipsis()
                    .text_color(if self.custom_rpc_url.is_empty() {
                        self.theme.text_disabled
                    } else {
                        self.theme.text_primary
                    })
                    .child(if self.rpc_focused {
                        format!("{}_", display_text)
                    } else {
                        display_text.to_string()
                    }),
            )
            .when(self.rpc_focused && !self.custom_rpc_url.is_empty(), |el| {
                el.child(
                    div().w(px(1.0)).h(px(20.0)).bg(self.theme.primary), // Cursor animation
                )
            })
    }

    fn handle_rpc_key_event(&mut self, event: &KeyDownEvent, cx: &mut Context<Self>) {
        let keystroke = &event.keystroke;

        // Check for copy/paste commands
        let is_cmd_or_ctrl = if cfg!(target_os = "macos") {
            keystroke.modifiers.platform
        } else {
            keystroke.modifiers.control
        };

        if is_cmd_or_ctrl {
            match keystroke.key.as_str() {
                "c" => {
                    // Copy current field value to clipboard
                    if !self.custom_rpc_url.is_empty() {
                        cx.write_to_clipboard(ClipboardItem::new_string(
                            self.custom_rpc_url.to_string(),
                        ));
                    }
                    return;
                }
                "v" => {
                    // Paste from clipboard
                    if let Some(clipboard_text) = cx.read_from_clipboard() {
                        if let Some(text) = clipboard_text.text() {
                            self.custom_rpc_url = text.trim().to_string().into();
                            cx.notify();
                        }
                    }
                    return;
                }
                _ => {}
            }
        }

        match keystroke.key.as_str() {
            "backspace" => {
                if !self.custom_rpc_url.is_empty() {
                    let mut url = self.custom_rpc_url.to_string();
                    url.pop();
                    self.custom_rpc_url = url.into();
                    cx.notify();
                }
            }
            "escape" => {
                self.show_rpc_config = false;
                self.rpc_focused = false;
                cx.notify();
            }
            "enter" => {
                self.apply_custom_rpc(cx);
            }
            "space" => {
                let mut url = self.custom_rpc_url.to_string();
                url.push(' ');
                self.custom_rpc_url = url.into();
                cx.notify();
            }
            key => {
                if key.len() == 1 && !key.chars().any(|c| c.is_control()) {
                    let mut url = self.custom_rpc_url.to_string();
                    url.push_str(key);
                    self.custom_rpc_url = url.into();
                    cx.notify();
                }
            }
        }
    }

    fn render_rpc_config_dialog(&self, cx: &mut Context<Self>) -> impl IntoElement {
        // 创建一个覆盖整个窗口的半透明背景
        div()
            .absolute()
            .top_0()
            .left_0()
            .size_full()
            .bg(rgba(0x00000080))
            .flex()
            .items_center()
            .justify_center()
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(|this, _, _, cx| {
                    this.show_rpc_config = false;
                    this.rpc_focused = false;
                    cx.notify();
                }),
            )
            .child(
                // 对话框容器
                div()
                    .bg(self.theme.background)
                    .rounded(px(12.0))
                    .border_1()
                    .border_color(self.theme.border)
                    .p(px(24.0))
                    .w(px(500.0))
                    .shadow_lg()
                    .on_mouse_down(MouseButton::Left, |_, _, _| {
                        // 阻止事件冒泡，防止点击对话框内容时关闭
                    })
                    .child(
                        v_flex()
                            .gap_4()
                            .child(
                                div()
                                    .text_xl()
                                    .text_color(self.theme.text_primary)
                                    .mb(px(8.0))
                                    .child("配置 RPC 端点"),
                            )
                            .child(
                                div()
                                    .text_sm()
                                    .text_color(self.theme.text_secondary)
                                    .child("输入自定义 RPC URL 以连接到不同的节点"),
                            )
                            .child(
                                div()
                                    .flex()
                                    .flex_col()
                                    .gap_2()
                                    .mt(px(16.0))
                                    .child(
                                        div()
                                            .text_sm()
                                            .text_color(self.theme.text_secondary)
                                            .child("RPC URL"),
                                    )
                                    .child(self.render_rpc_input_field(cx)),
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(self.theme.text_disabled)
                                    .mt(px(8.0))
                                    .child(
                                        "常用 RPC 提供商：Alchemy, QuickNode, Helius, Triton 等",
                                    ),
                            )
                            .child(
                                div()
                                    .flex()
                                    .gap_3()
                                    .justify_end()
                                    .mt(px(20.0))
                                    .child(self.wrap_button_with_theme(
                                        Button::new("cancel-rpc").label("取消").ghost().on_click(
                                            cx.listener(|this, _, _, cx| {
                                                this.show_rpc_config = false;
                                                this.rpc_focused = false;
                                                cx.notify();
                                            }),
                                        ),
                                        false,
                                        cx,
                                    ))
                                    .child(self.wrap_button_with_theme(
                                        Button::new("apply-rpc").label("应用").primary().on_click(
                                            cx.listener(|this, _, _, cx| {
                                                this.apply_custom_rpc(cx);
                                            }),
                                        ),
                                        true,
                                        cx,
                                    )),
                            ),
                    ),
            )
    }

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
                            .child("Solana Wallet"),
                    )
                    .child(
                        div()
                            .text_lg()
                            .text_color(self.theme.text_secondary)
                            .child("基于 GPUI 的高性能桌面钱包"),
                    ),
            )
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_4()
                    .w_full()
                    .max_w(px(300.0))
                    .child(
                        self.wrap_button_with_theme(
                            Button::new("create-wallet")
                                .label("创建新钱包")
                                .primary()
                                .on_click(cx.listener(|this, _, _window, cx| {
                                    this.create_wallet(cx);
                                    cx.notify();
                                })),
                            true,
                            cx,
                        ),
                    )
                    .child(
                        self.wrap_button_with_theme(
                            Button::new("import-wallet")
                                .label("导入已有钱包")
                                .ghost()
                                .on_click(cx.listener(|this, _, _window, cx| {
                                    this.import_wallet(cx);
                                    cx.notify();
                                })),
                            false,
                            cx,
                        ),
                    ),
            )
    }

    fn render_mnemonic_content(
        &self,
        mnemonic: &Option<MnemonicPhrase>,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
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
                    .child("创建新钱包"),
            )
            .child(
                div()
                    .text_color(self.theme.text_secondary)
                    .text_center()
                    .max_w(px(600.0))
                    .child("请妥善保存您的助记词，这是恢复钱包的唯一方式"),
            )
            .child(if let Some(mnemonic) = mnemonic {
                div()
                    .flex()
                    .flex_col()
                    .gap_4()
                    .p(px(20.0))
                    .bg(self.theme.surface)
                    .rounded(px(8.0))
                    .max_w(px(600.0))
                    .child(div().flex().flex_wrap().gap_3().children(
                        mnemonic.words().into_iter().enumerate().map(|(i, word)| {
                            div()
                                .flex()
                                .gap_2()
                                .child(
                                    div()
                                        .text_sm()
                                        .text_color(self.theme.text_disabled)
                                        .child(format!("{}.", i + 1)),
                                )
                                .child(div().text_color(self.theme.text_primary).child(word))
                        }),
                    ))
            } else {
                div()
                    .text_color(self.theme.text_secondary)
                    .child("生成助记词中...")
            })
            .child(
                div()
                    .flex()
                    .gap_4()
                    .child(
                        self.wrap_button_with_theme(
                            Button::new("back")
                                .label("返回")
                                .ghost()
                                .on_click(cx.listener(|this, _, _window, cx| {
                                    this.view_state = ViewState::Welcome;
                                    cx.notify();
                                })),
                            false,
                            cx,
                        ),
                    )
                    .child(
                        self.wrap_button_with_theme(
                            Button::new("continue")
                                .label("我已保存助记词")
                                .primary()
                                .on_click(cx.listener(|this, _, _window, cx| {
                                    if let ViewState::CreateWallet { mnemonic, .. } =
                                        &this.view_state
                                    {
                                        this.view_state = ViewState::CreateWallet {
                                            mnemonic: mnemonic.clone(),
                                            step: CreateWalletStep::SetPassword,
                                        };
                                        cx.notify();
                                    }
                                })),
                            true,
                            cx,
                        ),
                    ),
            )
    }

    fn render_password_content(
        &self,
        mnemonic: &Option<MnemonicPhrase>,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
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
                    .child("钱包创建成功"),
            )
            .child(
                div()
                    .text_color(self.theme.success)
                    .child("✓ 您的钱包已经创建成功！"),
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
                            .child("为了演示，我们使用默认设置："),
                    )
                    .child(
                        div()
                            .text_sm()
                            .text_color(self.theme.text_primary)
                            .child("钱包名称: 我的钱包"),
                    )
                    .child(
                        div()
                            .text_sm()
                            .text_color(self.theme.text_primary)
                            .child("密码: (已加密存储)"),
                    ),
            )
            .child(
                self.wrap_button_with_theme(
                    Button::new("continue-to-dashboard")
                        .label("进入钱包")
                        .primary()
                        .on_click(cx.listener(|this, _, _window, cx| {
                            // 使用默认值保存钱包
                            this.wallet_name = "我的钱包".into();
                            this.password = "password123".into();
                            this.confirm_password = "password123".into();
                            this.save_wallet(cx);
                        })),
                    true,
                    cx,
                ),
            )
    }

    fn render_import_wallet_content(&self, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex()
            .flex_col()
            .size_full()
            .items_center()
            .justify_center()
            .gap_6()
            .p(px(20.0))
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(|this, _, _, cx| {
                    // Set initial focus to mnemonic field if nothing is focused
                    if this.import_focused_field.is_none() {
                        this.import_focused_field = Some(ImportField::Mnemonic);
                        cx.notify();
                    }
                }),
            )
            .child(
                div()
                    .text_2xl()
                    .text_color(self.theme.text_primary)
                    .child("导入钱包"),
            )
            .child(
                // 导入类型切换
                div()
                    .flex()
                    .gap_4()
                    .mb(px(20.0))
                    .child(
                        self.wrap_button_with_theme(
                            Button::new("import-type-mnemonic")
                                .label("助记词")
                                .when(self.import_type == ImportType::Mnemonic, |b| b.primary())
                                .when(self.import_type != ImportType::Mnemonic, |b| b.ghost())
                                .on_click(cx.listener(|this, _, _, cx| {
                                    this.import_type = ImportType::Mnemonic;
                                    this.import_focused_field = Some(ImportField::Mnemonic);
                                    // 清空私钥
                                    this.import_private_key = SharedString::default();
                                    cx.notify();
                                })),
                            self.import_type == ImportType::Mnemonic,
                            cx,
                        ),
                    )
                    .child(
                        self.wrap_button_with_theme(
                            Button::new("import-type-private-key")
                                .label("私钥")
                                .when(self.import_type == ImportType::PrivateKey, |b| b.primary())
                                .when(self.import_type != ImportType::PrivateKey, |b| b.ghost())
                                .on_click(cx.listener(|this, _, _, cx| {
                                    this.import_type = ImportType::PrivateKey;
                                    this.import_focused_field = Some(ImportField::PrivateKey);
                                    // 清空助记词
                                    this.import_mnemonic = SharedString::default();
                                    cx.notify();
                                })),
                            self.import_type == ImportType::PrivateKey,
                            cx,
                        ),
                    ),
            )
            .child(
                div()
                    .text_color(self.theme.text_secondary)
                    .text_center()
                    .max_w(px(500.0))
                    .child(if self.import_type == ImportType::Mnemonic {
                        "请输入您的12个或24个助记词，用空格分隔"
                    } else {
                        "请输入您的Base58格式私钥"
                    }),
            )
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_4()
                    .w_full()
                    .max_w(px(500.0))
                    .child(
                        // 助记词或私钥输入框
                        if self.import_type == ImportType::Mnemonic {
                            div()
                                .flex()
                                .flex_col()
                                .gap_2()
                                .child(
                                    div()
                                        .text_sm()
                                        .text_color(self.theme.text_secondary)
                                        .child("助记词"),
                                )
                                .child(self.render_textarea_field(
                                    &self.import_mnemonic,
                                    "输入您的12个或者更多助记词...",
                                    ImportField::Mnemonic,
                                    cx,
                                ))
                        } else {
                            div()
                                .flex()
                                .flex_col()
                                .gap_2()
                                .child(
                                    div()
                                        .text_sm()
                                        .text_color(self.theme.text_secondary)
                                        .child("私钥"),
                                )
                                .child(self.render_input_field(
                                    &self.import_private_key,
                                    "输入您的私钥...",
                                    ImportField::PrivateKey,
                                    false,
                                    cx,
                                ))
                        },
                    )
                    .child(
                        // 钱包名称输入框
                        div()
                            .flex()
                            .flex_col()
                            .gap_2()
                            .child(
                                div()
                                    .text_sm()
                                    .text_color(self.theme.text_secondary)
                                    .child("钱包名称"),
                            )
                            .child(self.render_input_field(
                                &self.import_wallet_name,
                                "输入钱包名称...",
                                ImportField::WalletName,
                                false,
                                cx,
                            )),
                    )
                    .child(
                        // 密码输入框
                        div()
                            .flex()
                            .flex_col()
                            .gap_2()
                            .child(
                                div()
                                    .text_sm()
                                    .text_color(self.theme.text_secondary)
                                    .child("密码"),
                            )
                            .child(self.render_input_field(
                                &self.import_password,
                                "输入密码...",
                                ImportField::Password,
                                true,
                                cx,
                            )),
                    )
                    .child(
                        // 确认密码输入框
                        div()
                            .flex()
                            .flex_col()
                            .gap_2()
                            .child(
                                div()
                                    .text_sm()
                                    .text_color(self.theme.text_secondary)
                                    .child("确认密码"),
                            )
                            .child(self.render_input_field(
                                &self.import_confirm_password,
                                "确认密码...",
                                ImportField::ConfirmPassword,
                                true,
                                cx,
                            )),
                    ),
            )
            .child(
                // 错误提示
                if let Some(error) = &self.import_error {
                    div()
                        .text_sm()
                        .text_color(self.theme.error)
                        .mt(px(10.0))
                        .child(error.clone())
                } else {
                    div()
                },
            )
            .child(
                // 按钮组
                div()
                    .flex()
                    .gap_4()
                    .child(
                        self.wrap_button_with_theme(
                            Button::new("back")
                                .label("返回")
                                .ghost()
                                .on_click(cx.listener(|this, _, _window, cx| {
                                    // 清空输入
                                    this.import_mnemonic = SharedString::default();
                                    this.import_private_key = SharedString::default();
                                    this.import_wallet_name = SharedString::default();
                                    this.import_password = SharedString::default();
                                    this.import_confirm_password = SharedString::default();
                                    this.import_error = None;
                                    this.import_focused_field = None;
                                    this.view_state = ViewState::Welcome;
                                    cx.notify();
                                })),
                            false,
                            cx,
                        ),
                    )
                    .child(
                        self.wrap_button_with_theme(
                            Button::new("import").label("导入钱包").primary().on_click(
                                cx.listener(|this, _, _window, cx| {
                                    this.process_import_wallet(cx);
                                }),
                            ),
                            true,
                            cx,
                        ),
                    ),
            )
    }

    fn render_dashboard_content(
        &self,
        account: &WalletAccount,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
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
                            .child("钱包仪表板"),
                    )
                    .child(
                        div()
                            .flex()
                            .gap_3()
                            .items_center()
                            .child(
                                div()
                                    .text_sm()
                                    .text_color(self.theme.text_secondary)
                                    .child("网络:"),
                            )
                            .child(
                                div()
                                    .flex()
                                    .gap_2()
                                    .child(self.wrap_button_with_theme(
                                        if self.current_network == SolanaNetwork::Mainnet {
                                            Button::new("network-mainnet")
                                                .label("主网")
                                                .primary()
                                                .on_click(cx.listener(|this, _, _window, cx| {
                                                    this.switch_network(SolanaNetwork::Mainnet, cx);
                                                }))
                                        } else {
                                            Button::new("network-mainnet")
                                                .label("主网")
                                                .ghost()
                                                .on_click(cx.listener(|this, _, _window, cx| {
                                                    this.switch_network(SolanaNetwork::Mainnet, cx);
                                                }))
                                        },
                                        self.current_network == SolanaNetwork::Mainnet,
                                        cx,
                                    ))
                                    .child(self.wrap_button_with_theme(
                                        if self.current_network == SolanaNetwork::Devnet {
                                            Button::new("network-devnet")
                                                .label("开发网")
                                                .primary()
                                                .on_click(cx.listener(|this, _, _window, cx| {
                                                    this.switch_network(SolanaNetwork::Devnet, cx);
                                                }))
                                        } else {
                                            Button::new("network-devnet")
                                                .label("开发网")
                                                .ghost()
                                                .on_click(cx.listener(|this, _, _window, cx| {
                                                    this.switch_network(SolanaNetwork::Devnet, cx);
                                                }))
                                        },
                                        self.current_network == SolanaNetwork::Devnet,
                                        cx,
                                    ))
                                    .child(self.wrap_button_with_theme(
                                        if self.current_network == SolanaNetwork::Testnet {
                                            Button::new("network-testnet")
                                                .label("测试网")
                                                .primary()
                                                .on_click(cx.listener(|this, _, _window, cx| {
                                                    this.switch_network(SolanaNetwork::Testnet, cx);
                                                }))
                                        } else {
                                            Button::new("network-testnet")
                                                .label("测试网")
                                                .ghost()
                                                .on_click(cx.listener(|this, _, _window, cx| {
                                                    this.switch_network(SolanaNetwork::Testnet, cx);
                                                }))
                                        },
                                        self.current_network == SolanaNetwork::Testnet,
                                        cx,
                                    ))
                                    .child(self.wrap_button_with_theme(
                                        Button::new("rpc-config").label("⚙️").ghost().on_click(
                                            cx.listener(|this, _, _window, cx| {
                                                this.show_rpc_config_dialog(cx);
                                            }),
                                        ),
                                        false,
                                        cx,
                                    )),
                            ),
                    ),
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
                                    .child(account.name.clone()),
                            )
                            .child(
                                self.wrap_button_with_theme(
                                    Button::new("copy-address")
                                        .label(if self.show_copy_success { "已复制!" } else { "复制地址" })
                                        .ghost()
                                        .on_click(cx.listener(move |this, _, _window, cx| {
                                            if let ViewState::Dashboard { account_index } = this.view_state {
                                                if let Some(account) = this.accounts.get(account_index) {
                                                    let address = account.pubkey.to_string();
                                                    cx.write_to_clipboard(ClipboardItem::new_string(address.clone()));
                                                    println!("已复制地址: {}", address);
                                                    
                                                    // 显示复制成功提示
                                                    this.show_copy_success = true;
                                                    cx.notify();
                                                    
                                                    // 停止之前的定时器
                                                    if let Some(timer) = this.copy_success_timer.take() {
                                                        let _ = timer;
                                                    }
                                                    
                                                    // For now, we won't auto-hide the copy success message
                                                    // It will be cleared when the user copies again or navigates away
                                                }
                                            }
                                        })),
                                    false,
                                    cx,
                                ),
                            ),
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
                                    .child("地址:"),
                            )
                            .child(
                                div()
                                    .text_sm()
                                    .text_color(self.theme.text_primary)
                                    .truncate()
                                    .child(account.pubkey.to_string()),
                            ),
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
                                    .child("余额:"),
                            )
                            .child(if self.loading_balance {
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
                                            .child(format!("{:.6}", balance)),
                                    )
                                    .child(
                                        div()
                                            .text_lg()
                                            .text_color(self.theme.text_secondary)
                                            .child("SOL"),
                                    )
                            } else {
                                div()
                                    .text_2xl()
                                    .text_color(self.theme.error)
                                    .child("获取失败")
                            }),
                    ),
            )
            .child(
                // 操作按钮
                div()
                    .flex()
                    .flex_wrap()
                    .gap_3()
                    .w_full()
                    .child(
                        self.wrap_button_with_theme(
                            Button::new("send")
                                .label("发送")
                                .primary()
                                .on_click(cx.listener(move |this, _, _window, cx| {
                                    if let ViewState::Dashboard { account_index } = this.view_state
                                    {
                                        // 停止余额更新定时器
                                        this.stop_balance_update_timer();
                                        this.view_state =
                                            ViewState::SendTransaction { account_index };
                                        cx.notify();
                                    }
                                })),
                            true,
                            cx,
                        ),
                    )
                    .child(
                        self.wrap_button_with_theme(
                            Button::new("receive")
                                .label("接收")
                                .ghost()
                                .on_click(cx.listener(move |this, _, _window, cx| {
                                    if let ViewState::Dashboard { account_index } = this.view_state {
                                        // 停止余额更新定时器
                                        this.stop_balance_update_timer();
                                        this.view_state = ViewState::ReceiveTransaction { account_index };
                                        cx.notify();
                                    }
                                })),
                            false,
                            cx,
                        ),
                    )
                    .child(
                        self.wrap_button_with_theme(
                            Button::new("refresh")
                                .label("刷新余额")
                                .ghost()
                                .on_click(cx.listener(|this, _, _window, cx| {
                                    if let ViewState::Dashboard { account_index } = this.view_state
                                    {
                                        this.fetch_balance(account_index, cx);
                                    }
                                })),
                            false,
                            cx,
                        ),
                    )
                    .child(if self.current_network != SolanaNetwork::Mainnet {
                        self.wrap_button_with_theme(
                            Button::new("airdrop")
                                .label(if self.requesting_airdrop {
                                    "请求中..."
                                } else {
                                    "🪂 空投"
                                })
                                .ghost()
                                .on_click(cx.listener(|this, _, _window, cx| {
                                    if !this.requesting_airdrop {
                                        this.request_airdrop(cx);
                                    }
                                })),
                            false,
                            cx,
                        )
                    } else {
                        self.wrap_button_with_theme(
                            Button::new("airdrop-disabled")
                                .label("空投(仅测试网)")
                                .ghost()
                                .on_click(cx.listener(|_, _, _window, _cx| {})),
                            false,
                            cx,
                        )
                    }),
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
                            .child("交易历史"),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .w_full()
                            .max_h(px(400.0))
                            .bg(self.theme.surface)
                            .rounded(px(8.0))
                            .border_1()
                            .border_color(self.theme.border)
                            .overflow_hidden()
                            .child(
                                if self.transaction_history.is_empty() {
                                    div()
                                        .flex()
                                        .w_full()
                                        .h(px(200.0))
                                        .items_center()
                                        .justify_center()
                                        .child(
                                            div()
                                                .text_color(self.theme.text_disabled)
                                                .child("暂无交易记录"),
                                        )
                                } else {
                                    div()
                                        .flex()
                                        .flex_col()
                                        .w_full()
                                        .children(
                                            self.transaction_history.iter().map(|tx| {
                                                self.render_transaction_item(tx, cx)
                                            }),
                                        )
                                },
                            ),
                    ),
            )
    }

    fn render_transaction_item(
        &self,
        tx: &TransactionRecord,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let is_sent = self.accounts.iter()
            .any(|acc| acc.pubkey == tx.from);
        
        div()
            .flex()
            .w_full()
            .p(px(12.0))
            .border_b_1()
            .border_color(self.theme.border)
            .hover(|style| style.bg(self.theme.surface_hover))
            .child(
                div()
                    .flex()
                    .w_full()
                    .items_center()
                    .justify_between()
                    .child(
                        div()
                            .flex()
                            .gap_3()
                            .items_center()
                            .child(
                                // 交易图标
                                div()
                                    .size(px(40.0))
                                    .rounded_full()
                                    .bg(if is_sent {
                                        rgba(0xff666619) // 添加 alpha 通道
                                    } else {
                                        rgba(0x00ff0019) // 添加 alpha 通道
                                    })
                                    .flex()
                                    .items_center()
                                    .justify_center()
                                    .child(
                                        div()
                                            .text_color(if is_sent {
                                                self.theme.error
                                            } else {
                                                self.theme.success
                                            })
                                            .child(if is_sent { "↑" } else { "↓" }),
                                    ),
                            )
                            .child(
                                div()
                                    .flex()
                                    .flex_col()
                                    .gap_1()
                                    .child(
                                        div()
                                            .text_sm()
                                            .text_color(self.theme.text_primary)
                                            .child(if is_sent { "发送" } else { "接收" }),
                                    )
                                    .child(
                                        div()
                                            .text_xs()
                                            .text_color(self.theme.text_secondary)
                                            .child(format!(
                                                "{}", 
                                                tx.timestamp.format("%Y-%m-%d %H:%M:%S")
                                            )),
                                    ),
                            ),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .items_end()
                            .gap_1()
                            .child(
                                div()
                                    .text_sm()
                                    .text_color(if is_sent {
                                        self.theme.error
                                    } else {
                                        self.theme.success
                                    })
                                    .child(format!(
                                        "{}{:.4} SOL",
                                        if is_sent { "-" } else { "+" },
                                        tx.amount_in_sol()
                                    )),
                            )
                            .child(
                                match &tx.status {
                                    TransactionStatus::Confirmed => div()
                                        .text_xs()
                                        .text_color(self.theme.success)
                                        .child("已确认"),
                                    TransactionStatus::Pending => div()
                                        .text_xs()
                                        .text_color(self.theme.warning)
                                        .child("待确认"),
                                    TransactionStatus::Failed(err) => div()
                                        .text_xs()
                                        .text_color(self.theme.error)
                                        .child(format!("失败: {}", err)),
                                },
                            ),
                    ),
            )
    }

    fn render_send_transaction_content(
        &self,
        account: &WalletAccount,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        div()
            .flex()
            .flex_col()
            .size_full()
            .p(px(20.0))
            .gap_6()
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
                            .child("发送 SOL"),
                    )
                    .child(
                        self.wrap_button_with_theme(
                            Button::new("back-to-dashboard")
                                .label("返回")
                                .ghost()
                                .on_click(cx.listener(move |this, _, _window, cx| {
                                    if let ViewState::SendTransaction { account_index } =
                                        this.view_state
                                    {
                                        this.view_state = ViewState::Dashboard { account_index };
                                        // 清空输入
                                        this.send_to_address = SharedString::default();
                                        this.send_amount = SharedString::default();
                                        this.send_error = None;
                                        // 恢复余额更新定时器
                                        this.fetch_balance(account_index, cx);
                                        this.start_balance_update_timer(account_index, cx);
                                        cx.notify();
                                    }
                                })),
                            false,
                            cx,
                        ),
                    ),
            )
            .child(
                // 发送表单
                div()
                    .flex()
                    .flex_col()
                    .gap_4()
                    .w_full()
                    .max_w(px(500.0))
                    .child(
                        // 从地址（只读）
                        div()
                            .flex()
                            .flex_col()
                            .gap_2()
                            .child(
                                div()
                                    .text_sm()
                                    .text_color(self.theme.text_secondary)
                                    .child("从地址"),
                            )
                            .child(
                                div()
                                    .w_full()
                                    .h(px(40.0))
                                    .px(px(12.0))
                                    .bg(self.theme.surface)
                                    .rounded(px(8.0))
                                    .border_1()
                                    .border_color(self.theme.border)
                                    .flex()
                                    .items_center()
                                    .child(
                                        div()
                                            .text_sm()
                                            .text_color(self.theme.text_disabled)
                                            .truncate()
                                            .child(account.pubkey.to_string()),
                                    ),
                            ),
                    )
                    .child(
                        // 余额显示
                        div()
                            .flex()
                            .flex_col()
                            .gap_2()
                            .child(
                                div()
                                    .text_sm()
                                    .text_color(self.theme.text_secondary)
                                    .child("可用余额"),
                            )
                            .child(div().text_lg().text_color(self.theme.text_primary).child(
                                if let Some(balance) = self.balance {
                                    format!("{:.6} SOL", balance)
                                } else {
                                    "0.000000 SOL".to_string()
                                },
                            )),
                    )
                    .child(
                        // 目标地址输入
                        div()
                            .flex()
                            .flex_col()
                            .gap_2()
                            .child(
                                div()
                                    .text_sm()
                                    .text_color(self.theme.text_secondary)
                                    .child("接收地址"),
                            )
                            .child(self.render_input_field(
                                &self.send_to_address,
                                "输入接收地址...",
                                InputField::SendToAddress,
                                false,
                                cx,
                            )),
                    )
                    .child(
                        // 金额输入
                        div()
                            .flex()
                            .flex_col()
                            .gap_2()
                            .child(
                                div()
                                    .text_sm()
                                    .text_color(self.theme.text_secondary)
                                    .child("发送金额"),
                            )
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .gap_2()
                                    .child(
                                        div()
                                            .flex_1()
                                            .child(self.render_input_field(
                                                &self.send_amount,
                                                "0.00",
                                                InputField::SendAmount,
                                                false,
                                                cx,
                                            )),
                                    )
                                    .child(
                                        div()
                                            .text_color(self.theme.text_secondary)
                                            .pr(px(12.0))
                                            .child("SOL"),
                                    ),
                            ),
                    )
                    .child(
                        // 预估费用
                        div()
                            .flex()
                            .flex_col()
                            .gap_2()
                            .child(
                                div()
                                    .text_sm()
                                    .text_color(self.theme.text_secondary)
                                    .child("预估网络费用"),
                            )
                            .child(
                                div()
                                    .text_sm()
                                    .text_color(self.theme.text_primary)
                                    .child("~0.000005 SOL"),
                            ),
                    ),
            )
            .child(
                // 错误提示
                if let Some(error) = &self.send_error {
                    div()
                        .text_sm()
                        .text_color(self.theme.error)
                        .child(error.clone())
                } else {
                    div()
                },
            )
            .child(
                // 发送按钮
                div().flex().justify_center().w_full().child(
                    self.wrap_button_with_theme(
                        Button::new("confirm-send")
                            .label(if self.sending_transaction {
                                "发送中..."
                            } else {
                                "确认发送"
                            })
                            .primary()
                            .on_click(cx.listener(|this, _, _window, cx| {
                                if !this.sending_transaction {
                                    this.process_send_transaction(cx);
                                }
                            })),
                        true,
                        cx,
                    ),
                ),
            )
    }

    fn process_send_transaction(&mut self, cx: &mut Context<Self>) {
        // 验证输入
        if self.send_to_address.is_empty() {
            self.send_error = Some("请输入接收地址".to_string());
            cx.notify();
            return;
        }

        if self.send_amount.is_empty() {
            self.send_error = Some("请输入发送金额".to_string());
            cx.notify();
            return;
        }

        // 验证地址格式
        let recipient_address = self.send_to_address.to_string();
        if let Err(_) = solana_sdk::pubkey::Pubkey::from_str(&recipient_address) {
            self.send_error = Some("无效的地址格式".to_string());
            cx.notify();
            return;
        }

        // 验证金额
        let amount: f64 = match self.send_amount.parse() {
            Ok(amt) if amt > 0.0 => amt,
            _ => {
                self.send_error = Some("请输入有效的金额".to_string());
                cx.notify();
                return;
            }
        };

        // 检查余额是否足够
        if let Some(balance) = self.balance {
            if amount > balance {
                self.send_error = Some(format!("余额不足。可用: {:.6} SOL", balance));
                cx.notify();
                return;
            }
        }

        self.send_error = None;
        self.sending_transaction = true;
        cx.notify();

        // 获取当前账户
        if let ViewState::SendTransaction { account_index } = self.view_state {
            if let Some(account) = self.accounts.get(account_index) {
                if let Some(keypair) = &account.keypair {
                    // 准备发送真实交易
                    let rpc_manager = self.rpc_manager.clone();
                    let from_pubkey = keypair.pubkey();
                    let to_pubkey = solana_sdk::pubkey::Pubkey::from_str(&recipient_address).unwrap();
                    let lamports = (amount * 1_000_000_000.0) as u64;
                    let keypair_clone = keypair.inner().insecure_clone();

                    // 创建通道来接收结果
                    let (tx, rx) = std::sync::mpsc::channel();
                    self.pending_transaction = Some(rx);
                    
                    // 在后台线程中执行异步任务
                    std::thread::spawn(move || {
                        let rt = tokio::runtime::Runtime::new().unwrap();
                        let result = rt.block_on(async {
                            use wallet::TransactionHelper;
                            
                            match TransactionHelper::create_transfer_sol(
                                &rpc_manager,
                                &keypair_clone,
                                &to_pubkey,
                                amount,
                            ).await {
                                Ok(transaction) => {
                                    // 发送交易
                                    match rpc_manager.send_transaction(&transaction).await {
                                        Ok(signature) => {
                                            println!("交易成功! 签名: {}", signature);
                                            Ok((signature, from_pubkey, to_pubkey, lamports))
                                        }
                                        Err(e) => {
                                            println!("发送交易失败: {}", e);
                                            Err(format!("发送失败: {}", e))
                                        }
                                    }
                                }
                                Err(e) => {
                                    println!("创建交易失败: {}", e);
                                    Err(format!("创建交易失败: {}", e))
                                }
                            }
                        });
                        
                        // 发送结果
                        let _ = tx.send(result);
                    });
                } else {
                    self.send_error = Some("当前账户没有私钥，无法发送交易".to_string());
                    self.sending_transaction = false;
                    cx.notify();
                }
            }
        }
    }

    fn render_receive_transaction_content(
        &self,
        account: &WalletAccount,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        use qrcode::QrCode;
        
        // Generate QR code for the wallet address
        let address = account.pubkey.to_string();
        let qr_code = QrCode::new(&address).unwrap_or_else(|_| QrCode::new("ERROR").unwrap());
        
        // Convert QR code to text representation using Unicode blocks
        let qr_modules = qr_code.to_colors();
        let size = qr_code.width();
        
        // Create a grid of divs to represent QR code
        let mut qr_rows = Vec::new();
        
        // Add white border around QR code
        let border_size = 2;
        let total_size = size + (border_size * 2);
        
        for y in 0..total_size {
            let mut row_cells = Vec::new();
            
            for x in 0..total_size {
                // Check if we're in the border area
                let is_border = x < border_size || x >= size + border_size || 
                               y < border_size || y >= size + border_size;
                
                let is_dark = if is_border {
                    false // Border is always white
                } else {
                    let qr_x = x - border_size;
                    let qr_y = y - border_size;
                    let idx = qr_y * size + qr_x;
                    qr_modules[idx] == qrcode::Color::Dark
                };
                
                let cell = div()
                    .w(px(5.0))
                    .h(px(5.0))
                    .bg(if is_dark { rgb(0x000000) } else { rgb(0xFFFFFF) });
                
                row_cells.push(cell);
            }
            
            let row = div()
                .flex()
                .flex_row()
                .children(row_cells);
            
            qr_rows.push(row);
        }
        
        div()
            .flex()
            .flex_col()
            .size_full()
            .p(px(20.0))
            .gap_6()
            .child(
                // Header
                div()
                    .flex()
                    .w_full()
                    .items_center()
                    .justify_between()
                    .child(
                        div()
                            .text_2xl()
                            .text_color(self.theme.text_primary)
                            .child("接收 SOL"),
                    )
                    .child(
                        self.wrap_button_with_theme(
                            Button::new("back-to-dashboard-receive")
                                .label("返回")
                                .ghost()
                                .on_click(cx.listener(move |this, _, _window, cx| {
                                    if let ViewState::ReceiveTransaction { account_index } =
                                        this.view_state
                                    {
                                        this.view_state = ViewState::Dashboard { account_index };
                                        // 恢复余额更新定时器
                                        this.fetch_balance(account_index, cx);
                                        this.start_balance_update_timer(account_index, cx);
                                        cx.notify();
                                    }
                                })),
                            false,
                            cx,
                        ),
                    ),
            )
            .child(
                // Content
                div()
                    .flex()
                    .flex_col()
                    .items_center()
                    .gap_6()
                    .bg(self.theme.card_background())
                    .rounded(px(12.0))
                    .p(px(24.0))
                    .child(
                        // QR Code display using grid of divs
                        div()
                            .p(px(16.0))
                            .bg(rgb(0xffffff))
                            .rounded(px(8.0))
                            .border_1()
                            .border_color(self.theme.border)
                            .child(
                                div()
                                    .flex()
                                    .flex_col()
                                    .gap_0()
                                    .children(qr_rows)
                            ),
                    )
                    .child(
                        // Address display
                        div()
                            .flex()
                            .flex_col()
                            .items_center()
                            .gap_2()
                            .child(
                                div()
                                    .text_sm()
                                    .text_color(self.theme.text_secondary)
                                    .child("您的钱包地址"),
                            )
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .gap_2()
                                    .px(px(4.0))
                                    .py(px(2.0))
                                    .bg(self.theme.surface_hover)
                                    .rounded(px(6.0))
                                    .child(
                                        div()
                                            .text_sm()
                                            .text_color(self.theme.text_primary)
                                            .font_family("monospace")
                                            .child(address.clone()),
                                    )
                                    .child(
                                        self.wrap_button_with_theme(
                                            Button::new("copy-address-receive")
                                                .icon(Icon::new(IconName::Copy))
                                                .ghost()
                                                .xsmall()
                                                .on_click(cx.listener(move |this, _, _window, cx| {
                                                    if let ViewState::ReceiveTransaction { account_index } = this.view_state {
                                                        if let Some(account) = this.accounts.get(account_index) {
                                                            let addr = account.pubkey.to_string();
                                                            cx.write_to_clipboard(ClipboardItem::new_string(addr.clone()));
                                                            this.show_copy_success = true;
                                                            cx.notify();
                                                            
                                                            // For now, we won't auto-hide the copy success message
                                                            // It will be cleared when the user copies again or navigates away
                                                        }
                                                    }
                                                })),
                                            false,
                                            cx,
                                        ),
                                    ),
                            ),
                    )
                    .child(
                        // Copy success message
                        if self.show_copy_success {
                            div()
                                .text_sm()
                                .text_color(self.theme.success)
                                .child("✓ 地址已复制")
                        } else {
                            div()
                        }
                    )
                    .child(
                        // Instructions
                        div()
                            .flex()
                            .flex_col()
                            .gap_2()
                            .mt(px(4.0))
                            .child(
                                div()
                                    .text_sm()
                                    .text_color(self.theme.text_secondary)
                                    .text_center()
                                    .child("使用此地址接收 SOL 和 SPL 代币"),
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(self.theme.text_disabled)
                                    .text_center()
                                    .child("请确保发送方使用正确的 Solana 网络"),
                            ),
                    ),
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
        let window_handle = cx
            .open_window(window_options, |window, cx| {
                window.activate_window();
                window.set_window_title("GPUI Solana Wallet");
                cx.new(|cx| MainWindow::new(window, cx))
            })
            .unwrap();

        // Ensure the window is visible
        window_handle
            .update(cx, |_, window, _| {
                window.activate_window();
            })
            .unwrap();

        println!("Window opened successfully!");
    });
}
