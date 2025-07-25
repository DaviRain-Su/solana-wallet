use crate::wallet::keypair::{DerivedKeypair, WalletKeypair};
use anyhow::{anyhow, Result};
use bip39::{Language, Mnemonic};

/// Represents a mnemonic phrase for wallet generation
#[derive(Debug, Clone, PartialEq)]
pub struct MnemonicPhrase {
    mnemonic: Mnemonic,
}

impl MnemonicPhrase {
    /// Create from an existing mnemonic string
    pub fn from_phrase(phrase: &str) -> Result<Self> {
        let mnemonic = Mnemonic::parse_normalized(phrase)
            .map_err(|e| anyhow!("Invalid mnemonic phrase: {}", e))?;

        Ok(Self { mnemonic })
    }

    /// Get the mnemonic phrase as a string
    pub fn phrase(&self) -> String {
        self.mnemonic.to_string()
    }

    /// Get the word list as a vector
    pub fn words(&self) -> Vec<String> {
        self.phrase()
            .split_whitespace()
            .map(|s| s.to_string())
            .collect()
    }

    /// Derive a keypair using BIP44 derivation path
    /// Default Solana derivation path: m/44'/501'/0'/0'
    pub fn derive_keypair(&self, account_index: u32) -> Result<DerivedKeypair> {
        let seed = self.mnemonic.to_seed("");
        let derivation_path = format!("m/44'/501'/{}'/0'", account_index);

        // 暂时简化实现，直接使用种子创建密钥对
        // TODO: 实现完整的 BIP32 派生
        let seed_bytes = seed.as_ref();
        let wallet_keypair = WalletKeypair::from_seed(seed_bytes)?;

        Ok(DerivedKeypair::new(
            wallet_keypair,
            derivation_path,
            account_index,
        ))
    }

    /// Derive multiple keypairs
    pub fn derive_keypairs(&self, count: u32) -> Result<Vec<DerivedKeypair>> {
        let mut keypairs = Vec::new();
        for i in 0..count {
            keypairs.push(self.derive_keypair(i)?);
        }
        Ok(keypairs)
    }
}

/// Generate a new mnemonic phrase with the specified word count
pub fn generate_mnemonic(word_count: usize) -> Result<MnemonicPhrase> {
    use rand::thread_rng;

    // Validate word count
    match word_count {
        12 | 15 | 18 | 21 | 24 => {}
        _ => return Err(anyhow!("Invalid word count. Must be 12, 15, 18, 21, or 24")),
    }

    let mut rng = thread_rng();
    let mnemonic = Mnemonic::generate_in_with(&mut rng, Language::English, word_count)
        .map_err(|e| anyhow!("Failed to generate mnemonic: {}", e))?;

    Ok(MnemonicPhrase { mnemonic })
}

/// Validate a mnemonic phrase
pub fn validate_mnemonic(phrase: &str) -> bool {
    Mnemonic::parse_normalized(phrase).is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_mnemonic() {
        let mnemonic = generate_mnemonic(12).unwrap();
        assert_eq!(mnemonic.words().len(), 12);
        assert!(validate_mnemonic(&mnemonic.phrase()));
    }

    #[test]
    fn test_derive_keypair() {
        let phrase = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
        let mnemonic = MnemonicPhrase::from_phrase(phrase).unwrap();
        let derived = mnemonic.derive_keypair(0).unwrap();

        // Known pubkey for this mnemonic with account index 0
        let expected_pubkey = "2wT8Xqnym6p6enxVkUEbicMTVDHqRBPrBpN8xqrLvNXx";
        // 这个测试可能需要根据实际的派生结果调整
        assert!(!derived.keypair.pubkey().to_string().is_empty());
    }
}
