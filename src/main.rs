use gpui::*;
use gpui_component::button::{Button, ButtonVariants};
use solana_sdk::pubkey::Pubkey;
use solana_client::rpc_client::RpcClient;

actions!(wallet, [Quit]);

struct MainWindow {
    count: usize,
}

impl MainWindow {
    fn new(_window: &mut Window, _cx: &mut Context<Self>) -> Self {
        Self { count: 0 }
    }
}

impl Render for MainWindow {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex()
            .size_full()
            .bg(rgb(0x1e1e1e))
            .items_center()
            .justify_center()
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_4()
                    .items_center()
                    .child(
                        div()
                            .text_2xl()
                            .text_color(rgb(0xffffff))
                            .child("GPUI + Solana SDK 集成成功! 🎉")
                    )
                    .child(
                        div()
                            .text_lg()
                            .text_color(rgb(0xaaaaaa))
                            .child(format!("点击次数: {}", self.count))
                    )
                    .child(
                        div()
                            .text_sm()
                            .text_color(rgb(0x888888))
                            .child("使用 Solana SDK 2.0")
                    )
                    .child(
                        Button::new("increment")
                            .label("Click me!")
                            .primary()
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.count += 1;
                                println!("Button clicked! Count: {}", this.count);
                                cx.notify();
                            }))
                    )
                    .child(
                        Button::new("quit")
                            .label("Quit")
                            .ghost()
                            .on_click(cx.listener(|_, _, _, cx| {
                                cx.quit();
                            }))
                    )
            )
    }
}

fn main() {
    let app = Application::new();
    
    app.run(move |cx: &mut App| {
        // Initialize theme
        gpui_component::init(cx);
        
        cx.activate(true);
        
        cx.open_window(
            gpui::WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(Bounds::centered(
                    None,
                    size(px(800.0), px(600.0)),
                    cx,
                ))),
                titlebar: Some(TitlebarOptions {
                    title: Some("GPUI Solana Wallet".into()),
                    ..Default::default()
                }),
                ..Default::default()
            },
            |window, cx| cx.new(|cx| MainWindow::new(window, cx)),
        )
        .unwrap();
    });
}