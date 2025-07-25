pub mod components;
pub mod views;
pub mod theme;

use gpui::*;
use gpui_component::*;
use crate::app::AppState;

pub struct MainWindow {
    state: Model<AppState>,
    active_view: ActiveView,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ActiveView {
    Dashboard,
    SendReceive,
    Transactions,
    Tokens,
    Settings,
}

impl MainWindow {
    pub fn new(state: Model<AppState>, cx: &mut ViewContext<Self>) -> Self {
        cx.observe(&state, |_, _, cx| {
            cx.notify();
        })
        .detach();

        Self {
            state,
            active_view: ActiveView::Dashboard,
        }
    }

    fn render_sidebar(&self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        let theme = theme::get_theme(cx);
        
        v_flex()
            .w(px(250.0))
            .h_full()
            .bg(theme.sidebar_bg)
            .border_r_1()
            .border_color(theme.border)
            .child(
                // Logo area
                h_flex()
                    .h(px(60.0))
                    .items_center()
                    .justify_center()
                    .border_b_1()
                    .border_color(theme.border)
                    .child(
                        text("Solana Wallet")
                            .size(px(20.0))
                            .weight(FontWeight::BOLD)
                            .color(theme.text_primary)
                    )
            )
            .child(
                // Navigation
                v_flex()
                    .p(px(16.0))
                    .gap(px(8.0))
                    .child(self.nav_item("Dashboard", ActiveView::Dashboard, cx))
                    .child(self.nav_item("Send/Receive", ActiveView::SendReceive, cx))
                    .child(self.nav_item("Transactions", ActiveView::Transactions, cx))
                    .child(self.nav_item("Tokens", ActiveView::Tokens, cx))
            )
            .child(
                // Bottom section
                v_flex()
                    .flex_1()
                    .justify_end()
                    .p(px(16.0))
                    .child(self.nav_item("Settings", ActiveView::Settings, cx))
            )
    }

    fn nav_item(
        &self,
        label: &str,
        view: ActiveView,
        cx: &mut ViewContext<Self>,
    ) -> impl IntoElement {
        let theme = theme::get_theme(cx);
        let is_active = self.active_view == view;
        
        h_flex()
            .h(px(40.0))
            .items_center()
            .px(px(12.0))
            .rounded(px(8.0))
            .cursor_pointer()
            .when(is_active, |this| {
                this.bg(theme.nav_active_bg)
                    .text_color(theme.nav_active_text)
            })
            .when(!is_active, |this| {
                this.hover(|style| style.bg(theme.nav_hover_bg))
            })
            .on_click(cx.listener(move |this, _, cx| {
                this.active_view = view;
                cx.notify();
            }))
            .child(
                text(label)
                    .size(px(14.0))
                    .color(if is_active {
                        theme.nav_active_text
                    } else {
                        theme.text_secondary
                    })
            )
    }

    fn render_content(&self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        let theme = theme::get_theme(cx);
        
        v_flex()
            .flex_1()
            .bg(theme.content_bg)
            .child(
                match self.active_view {
                    ActiveView::Dashboard => self.render_dashboard(cx),
                    ActiveView::SendReceive => self.render_send_receive(cx),
                    ActiveView::Transactions => self.render_transactions(cx),
                    ActiveView::Tokens => self.render_tokens(cx),
                    ActiveView::Settings => self.render_settings(cx),
                }
            )
    }

    fn render_dashboard(&self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        v_flex()
            .p(px(32.0))
            .child(
                text("Dashboard")
                    .size(px(24.0))
                    .weight(FontWeight::BOLD)
            )
            .child(
                text("Welcome to your Solana Wallet")
                    .size(px(16.0))
                    .color(rgb(0x666666))
            )
    }

    fn render_send_receive(&self, _cx: &mut ViewContext<Self>) -> impl IntoElement {
        v_flex()
            .p(px(32.0))
            .child(
                text("Send/Receive")
                    .size(px(24.0))
                    .weight(FontWeight::BOLD)
            )
    }

    fn render_transactions(&self, _cx: &mut ViewContext<Self>) -> impl IntoElement {
        v_flex()
            .p(px(32.0))
            .child(
                text("Transaction History")
                    .size(px(24.0))
                    .weight(FontWeight::BOLD)
            )
    }

    fn render_tokens(&self, _cx: &mut ViewContext<Self>) -> impl IntoElement {
        v_flex()
            .p(px(32.0))
            .child(
                text("Token Management")
                    .size(px(24.0))
                    .weight(FontWeight::BOLD)
            )
    }

    fn render_settings(&self, _cx: &mut ViewContext<Self>) -> impl IntoElement {
        v_flex()
            .p(px(32.0))
            .child(
                text("Settings")
                    .size(px(24.0))
                    .weight(FontWeight::BOLD)
            )
    }
}

impl Render for MainWindow {
    fn render(&mut self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        h_flex()
            .size_full()
            .font_family("Inter")
            .child(self.render_sidebar(cx))
            .child(self.render_content(cx))
    }
}