mod state;
mod events;

pub use state::AppState;
pub use events::WalletEvent;

use anyhow::Result;
// Temporarily comment out GPUI imports
// use gpui::*;
// use crate::ui::MainWindow;

pub struct SolanaWalletApp {
    state: AppState,
}

impl SolanaWalletApp {
    pub fn new() -> Result<Self> {
        // Initialize app state
        let state = AppState::new();

        Ok(Self { state })
    }
}