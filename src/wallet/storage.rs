use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce, Key,
};
use argon2::Argon2;
use rand::{thread_rng, RngCore};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use anyhow::{Result, anyhow};
use zeroize::Zeroize;

const NONCE_SIZE: usize = 12;
const SALT_SIZE: usize = 16;

/// Encrypted wallet data structure
#[derive(Debug, Serialize, Deserialize)]
pub struct EncryptedWallet {
    pub salt: Vec<u8>,
    pub nonce: Vec<u8>,
    pub ciphertext: Vec<u8>,
    pub version: u32,
}

/// Wallet data that will be encrypted
#[derive(Debug, Serialize, Deserialize)]
pub struct WalletData {
    pub mnemonic: String,
    pub accounts: Vec<AccountData>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub modified_at: chrono::DateTime<chrono::Utc>,
}

// 手动实现 Drop 以清除敏感数据
impl Drop for WalletData {
    fn drop(&mut self) {
        self.mnemonic.zeroize();
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Zeroize)]
#[zeroize(drop)]
pub struct AccountData {
    pub name: String,
    pub derivation_path: String,
    pub pubkey: String,
}

/// Manages encrypted wallet storage
pub struct WalletStorage {
    storage_path: PathBuf,
}

impl WalletStorage {
    /// Create new wallet storage manager
    pub fn new(storage_dir: impl AsRef<Path>) -> Result<Self> {
        let storage_path = storage_dir.as_ref().to_path_buf();
        
        // Create directory if it doesn't exist
        if !storage_path.exists() {
            std::fs::create_dir_all(&storage_path)?;
        }
        
        Ok(Self { storage_path })
    }
    
    /// Get default storage path
    pub fn default_path() -> Result<PathBuf> {
        let home = dirs::home_dir().ok_or_else(|| anyhow!("Could not find home directory"))?;
        Ok(home.join(".solana-wallet"))
    }
    
    /// Encrypt and save wallet data
    pub fn save_wallet(&self, wallet_name: &str, data: &WalletData, password: &str) -> Result<()> {
        // Serialize wallet data
        let plaintext = serde_json::to_vec(data)?;
        
        // Generate salt
        let mut salt = vec![0u8; SALT_SIZE];
        thread_rng().fill_bytes(&mut salt);
        
        // Derive key from password using Argon2
        let argon2 = Argon2::default();
        let mut key_bytes = [0u8; 32];
        if let Err(e) = argon2.hash_password_into(password.as_bytes(), &salt, &mut key_bytes) {
            return Err(anyhow!("Failed to derive key: {:?}", e));
        }
        let key = Key::<Aes256Gcm>::from_slice(&key_bytes);
        
        // Generate nonce
        let mut nonce_bytes = [0u8; NONCE_SIZE];
        thread_rng().fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);
        
        // Encrypt data
        let cipher = Aes256Gcm::new(key);
        let ciphertext = cipher.encrypt(nonce, plaintext.as_ref())
            .map_err(|e| anyhow!("Encryption failed: {}", e))?;
        
        // Create encrypted wallet structure
        let encrypted = EncryptedWallet {
            salt: salt.to_vec(),
            nonce: nonce_bytes.to_vec(),
            ciphertext,
            version: 1,
        };
        
        // Save to file
        let file_path = self.storage_path.join(format!("{}.wallet", wallet_name));
        let file_content = serde_json::to_vec_pretty(&encrypted)?;
        std::fs::write(file_path, file_content)?;
        
        // Clear sensitive data
        key_bytes.zeroize();
        
        Ok(())
    }
    
    /// Load and decrypt wallet data
    pub fn load_wallet(&self, wallet_name: &str, password: &str) -> Result<WalletData> {
        let file_path = self.storage_path.join(format!("{}.wallet", wallet_name));
        
        if !file_path.exists() {
            return Err(anyhow!("Wallet file not found"));
        }
        
        // Read encrypted wallet
        let file_content = std::fs::read(&file_path)?;
        let encrypted: EncryptedWallet = serde_json::from_slice(&file_content)?;
        
        // Derive key from password
        let argon2 = Argon2::default();
        let mut key_bytes = [0u8; 32];
        if let Err(e) = argon2.hash_password_into(password.as_bytes(), &encrypted.salt, &mut key_bytes) {
            return Err(anyhow!("Failed to derive key: {:?}", e));
        }
        let key = Key::<Aes256Gcm>::from_slice(&key_bytes);
        
        // Decrypt data
        let cipher = Aes256Gcm::new(key);
        let nonce = Nonce::from_slice(&encrypted.nonce);
        let plaintext = cipher.decrypt(nonce, encrypted.ciphertext.as_ref())
            .map_err(|_| anyhow!("Invalid password or corrupted data"))?;
        
        // Deserialize wallet data
        let wallet_data: WalletData = serde_json::from_slice(&plaintext)?;
        
        // Clear sensitive data
        key_bytes.zeroize();
        
        Ok(wallet_data)
    }
    
    /// List available wallets
    pub fn list_wallets(&self) -> Result<Vec<String>> {
        let mut wallets = Vec::new();
        
        for entry in std::fs::read_dir(&self.storage_path)? {
            let entry = entry?;
            let path = entry.path();
            
            if let Some(extension) = path.extension() {
                if extension == "wallet" {
                    if let Some(stem) = path.file_stem() {
                        if let Some(name) = stem.to_str() {
                            wallets.push(name.to_string());
                        }
                    }
                }
            }
        }
        
        Ok(wallets)
    }
    
    /// Delete a wallet
    pub fn delete_wallet(&self, wallet_name: &str) -> Result<()> {
        let file_path = self.storage_path.join(format!("{}.wallet", wallet_name));
        
        if file_path.exists() {
            std::fs::remove_file(file_path)?;
        }
        
        Ok(())
    }
    
    /// Check if wallet exists
    pub fn wallet_exists(&self, wallet_name: &str) -> bool {
        let file_path = self.storage_path.join(format!("{}.wallet", wallet_name));
        file_path.exists()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    
    #[test]
    fn test_encrypt_decrypt() {
        let temp_dir = TempDir::new().unwrap();
        let storage = WalletStorage::new(temp_dir.path()).unwrap();
        
        let wallet_data = WalletData {
            mnemonic: "test mnemonic phrase here".to_string(),
            accounts: vec![
                AccountData {
                    name: "Account 1".to_string(),
                    derivation_path: "m/44'/501'/0'/0'".to_string(),
                    pubkey: "TestPubkey123".to_string(),
                }
            ],
            created_at: chrono::Utc::now(),
            modified_at: chrono::Utc::now(),
        };
        
        let password = "test_password_123";
        let wallet_name = "test_wallet";
        
        // Save wallet
        storage.save_wallet(wallet_name, &wallet_data, password).unwrap();
        
        // Load wallet
        let loaded = storage.load_wallet(wallet_name, password).unwrap();
        
        assert_eq!(loaded.mnemonic, wallet_data.mnemonic);
        assert_eq!(loaded.accounts.len(), wallet_data.accounts.len());
        assert_eq!(loaded.accounts[0].name, wallet_data.accounts[0].name);
    }
    
    #[test]
    fn test_wrong_password() {
        let temp_dir = TempDir::new().unwrap();
        let storage = WalletStorage::new(temp_dir.path()).unwrap();
        
        let wallet_data = WalletData {
            mnemonic: "test mnemonic".to_string(),
            accounts: vec![],
            created_at: chrono::Utc::now(),
            modified_at: chrono::Utc::now(),
        };
        
        storage.save_wallet("test", &wallet_data, "correct_password").unwrap();
        
        // Try to load with wrong password
        let result = storage.load_wallet("test", "wrong_password");
        assert!(result.is_err());
    }
}