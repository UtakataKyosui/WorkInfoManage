use aes_gcm::{
    aead::{Aead, AeadCore, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
use anyhow::Result;
use base64::{engine::general_purpose, Engine as _};
use rand::RngCore;
use std::fs;
use std::path::Path;

const KEY_FILE_PATH: &str = "secret.key";

pub struct Crypto {
    cipher: Aes256Gcm,
}

impl Crypto {
    pub fn new() -> Result<Self> {
        let key = Self::load_or_generate_key()?;
        let cipher = Aes256Gcm::new(&key);
        Ok(Self { cipher })
    }

    fn load_or_generate_key() -> Result<Key<Aes256Gcm>> {
        let dir = crate::logic::utils::get_app_data_dir();
        if !dir.exists() {
            fs::create_dir_all(&dir)?;
        }
        let path = dir.join(KEY_FILE_PATH);

        if path.exists() {
            let key_bytes = fs::read(path)?;
            Ok(*Key::<Aes256Gcm>::from_slice(&key_bytes))
        } else {
            let key = Aes256Gcm::generate_key(OsRng);
            fs::write(path, key)?;
            Ok(key)
        }
    }

    pub fn encrypt(&self, plaintext: &str) -> Result<String> {
        let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
        let ciphertext = self
            .cipher
            .encrypt(&nonce, plaintext.as_bytes())
            .map_err(|e| anyhow::anyhow!("Encryption failed: {}", e))?;

        // Combine nonce + ciphertext
        let mut combined = nonce.to_vec();
        combined.extend_from_slice(&ciphertext);

        Ok(general_purpose::STANDARD.encode(combined))
    }

    pub fn decrypt(&self, encrypted_data: &str) -> Result<String> {
        let decoded = general_purpose::STANDARD.decode(encrypted_data)?;

        if decoded.len() < 12 {
            return Err(anyhow::anyhow!("Invalid encrypted data length"));
        }

        let (nonce_bytes, ciphertext_bytes) = decoded.split_at(12);
        let nonce = Nonce::from_slice(nonce_bytes);

        let plaintext_bytes = self
            .cipher
            .decrypt(nonce, ciphertext_bytes)
            .map_err(|e| anyhow::anyhow!("Decryption failed: {}", e))?;

        Ok(String::from_utf8(plaintext_bytes)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;

    #[test]
    #[serial]
    fn test_encrypt_decrypt() {
        // Use a different key file for testing to avoid messaging with real one?
        // Actually, the code uses hardcoded KEY_FILE_PATH joined with get_app_data_dir().
        // We should probably mock get_app_data_dir or just clean up carefully.

        let dir = crate::logic::utils::get_app_data_dir();
        // Ensure dir exists (it should be created by new())

        let key_path = dir.join(KEY_FILE_PATH);
        let backup_path = dir.join("secret.key.bak");

        if key_path.exists() {
            fs::rename(&key_path, &backup_path).unwrap();
        }

        let crypto = Crypto::new().unwrap();
        let plaintext = "Hello, World!";
        let encrypted = crypto.encrypt(plaintext).unwrap();
        let decrypted = crypto.decrypt(&encrypted).unwrap();

        assert_eq!(plaintext, decrypted);
        assert_ne!(plaintext, encrypted);

        // Restore key
        if backup_path.exists() {
            fs::rename(&backup_path, &key_path).unwrap();
        } else if key_path.exists() {
            // If we created a key but didn't have a backup (fresh install simulation), remove it?
            // Or just leave it as a test artifact? Better to remove if we created it.
            // But we can't easily know if we created it without checking before.
            // For now, let's just remove it if we didn't back up anything.
            fs::remove_file(&key_path).unwrap();
        }
    }
}
