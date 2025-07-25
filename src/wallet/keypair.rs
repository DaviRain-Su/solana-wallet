use solana_sdk::{
    pubkey::Pubkey,
    signature::{Keypair, Signature, SeedDerivable},
    signer::Signer,
};
use ed25519_dalek::SecretKey;
use serde::{Deserialize, Serialize};
use anyhow::{Result, anyhow};

#[derive(Debug)]
pub struct WalletKeypair {
    keypair: Keypair,
}

impl WalletKeypair {
    /// Create a new random keypair
    pub fn new() -> Self {
        Self {
            keypair: Keypair::new(),
        }
    }

    /// Create keypair from seed bytes
    pub fn from_seed(seed: &[u8]) -> Result<Self> {
        if seed.len() < 32 {
            return Err(anyhow!("Seed must be at least 32 bytes"));
        }
        
        // For Solana, we use the first 32 bytes of the seed as the secret key seed
        let keypair = Keypair::from_seed(&seed[0..32])
            .map_err(|e| anyhow!("Failed to create keypair from seed: {}", e))?;
        
        Ok(Self { keypair })
    }

    /// Create keypair from base58 private key string
    pub fn from_base58_string(s: &str) -> Result<Self> {
        let keypair = Keypair::from_base58_string(s);
        Ok(Self { keypair })
    }

    /// Get the public key
    pub fn pubkey(&self) -> Pubkey {
        self.keypair.pubkey()
    }

    /// Get the base58 encoded private key
    pub fn to_base58_string(&self) -> String {
        self.keypair.to_base58_string()
    }

    /// Sign a message
    pub fn sign_message(&self, message: &[u8]) -> Signature {
        self.keypair.sign_message(message)
    }

    /// Get the underlying keypair (be careful with this)
    pub(crate) fn inner(&self) -> &Keypair {
        &self.keypair
    }
}

/// Represents a keypair derived from a mnemonic phrase
#[derive(Debug)]
pub struct DerivedKeypair {
    pub keypair: WalletKeypair,
    pub derivation_path: String,
    pub account_index: u32,
}

impl DerivedKeypair {
    pub fn new(keypair: WalletKeypair, derivation_path: String, account_index: u32) -> Self {
        Self {
            keypair,
            derivation_path,
            account_index,
        }
    }
}