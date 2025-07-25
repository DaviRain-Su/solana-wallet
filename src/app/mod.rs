mod events;
mod state;

pub use events::WalletEvent;
pub use state::AppState;

use anyhow::Result;
// Temporarily comment out GPUI imports
use gpui::*;

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
