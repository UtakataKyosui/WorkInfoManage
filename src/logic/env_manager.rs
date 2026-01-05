use crate::security::crypto::Crypto;
use anyhow::{Context, Result};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

const ENV_EXAMPLE_PATH: &str = ".env.example";
const ENV_ENC_PATH: &str = ".env.enc";

#[derive(Clone, Debug)]
pub struct EnvVar {
    pub key: String,
    pub value: String,
    pub is_secret: bool,
}

pub struct EnvManager {
    crypto: Crypto,
    pub variables: Vec<EnvVar>,
    pub selection: usize,
    target_keys: HashSet<String>,
    example_path: PathBuf,
    enc_path: PathBuf,
}

impl EnvManager {
    pub fn new() -> Result<Self> {
        let app_dir = crate::logic::utils::get_app_data_dir();
        if !app_dir.exists() {
            fs::create_dir_all(&app_dir)?;
        }
        Self::new_with_paths(PathBuf::from(ENV_EXAMPLE_PATH), app_dir.join(ENV_ENC_PATH))
    }

    pub fn new_with_paths<P: AsRef<Path>>(example_path: P, enc_path: P) -> Result<Self> {
        let crypto = Crypto::new().context("Failed to initialize crypto")?;
        let mut manager = Self {
            crypto,
            variables: Vec::new(),
            selection: 0,
            target_keys: HashSet::new(),
            example_path: example_path.as_ref().to_path_buf(),
            enc_path: enc_path.as_ref().to_path_buf(),
        };
        manager.load_target_keys()?;
        manager.load_values()?;
        Ok(manager)
    }

    fn load_target_keys(&mut self) -> Result<()> {
        if !self.example_path.exists() {
            return Ok(());
        }

        let content = fs::read_to_string(&self.example_path)?;
        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            if let Some((key, _)) = line.split_once('=') {
                self.target_keys.insert(key.trim().to_string());
            }
        }
        Ok(())
    }

    fn load_values(&mut self) -> Result<()> {
        // First, populate variables from target_keys with empty or default values
        let mut vars = HashMap::new();
        for key in &self.target_keys {
            vars.insert(key.clone(), String::new());
        }

        // Try load encrypted file
        if self.enc_path.exists() {
            let content = fs::read_to_string(&self.enc_path)?;
            let decrypted_content = self
                .crypto
                .decrypt(&content)
                .context("Failed to decrypt .env.enc")
                .unwrap_or_else(|_| String::new()); // Fallback to empty if decrypt fails (wrong key?)

            for line in decrypted_content.lines() {
                if let Some((key, value)) = line.split_once('=') {
                    if self.target_keys.contains(key) {
                        vars.insert(key.to_string(), value.to_string());
                    }
                }
            }
        }

        // Convert to Vec<EnvVar> and sort
        self.variables = vars
            .into_iter()
            .map(|(key, value)| EnvVar {
                key,
                value,
                is_secret: true, // Treat all as secret for now, or match on name
            })
            .collect();
        self.variables.sort_by(|a, b| a.key.cmp(&b.key));

        Ok(())
    }

    pub fn save(&self) -> Result<()> {
        let mut lines = Vec::new();
        for var in &self.variables {
            if !var.value.is_empty() {
                lines.push(format!("{}={}", var.key, var.value));
            }
        }
        let content = lines.join("\n");
        let encrypted = self.crypto.encrypt(&content)?;
        fs::write(&self.enc_path, encrypted)?;
        Ok(())
    }

    pub fn update_value(&mut self, key: &str, value: String) {
        if let Some(var) = self.variables.iter_mut().find(|v| v.key == key) {
            var.value = value;
        }
    }

    pub fn get_value(&self, key: &str) -> Option<&String> {
        self.variables
            .iter()
            .find(|v| v.key == key)
            .map(|v| &v.value)
    }

    // Export decrypted variables as a map (for app usage)
    pub fn get_env_map(&self) -> HashMap<String, String> {
        self.variables
            .iter()
            .filter(|v| !v.value.is_empty())
            .map(|v| (v.key.clone(), v.value.clone()))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    #[serial]
    fn test_load_target_keys() {
        // Create temp files
        let example_file = NamedTempFile::new().unwrap();
        let enc_file = NamedTempFile::new().unwrap();
        let example_path = example_file.path().to_path_buf();
        let enc_path = enc_file.path().to_path_buf();

        // Write to example file
        let mut file = fs::File::create(&example_path).unwrap();
        writeln!(file, "TEST_KEY_1=value").unwrap();
        writeln!(file, "TEST_KEY_2=").unwrap();
        writeln!(file, "# Comment").unwrap();

        let manager = EnvManager::new_with_paths(&example_path, &enc_path).unwrap();

        // Check if keys are loaded
        let keys: HashSet<&String> = manager.variables.iter().map(|v| &v.key).collect();
        assert!(keys.contains(&"TEST_KEY_1".to_string()));
        assert!(keys.contains(&"TEST_KEY_2".to_string()));
        assert_eq!(manager.variables.len(), 2);
    }

    #[test]
    #[serial]
    fn test_save_and_load_encrypted() {
        let example_file = NamedTempFile::new().unwrap();
        let enc_file = NamedTempFile::new().unwrap();
        let example_path = example_file.path().to_path_buf();
        let enc_path = enc_file.path().to_path_buf();

        // Setup example
        {
            let mut file = fs::File::create(&example_path).unwrap();
            writeln!(file, "SECRET_KEY=ignore_me").unwrap();
        }

        // Initialize and Save
        {
            let mut manager = EnvManager::new_with_paths(&example_path, &enc_path).unwrap();
            if let Some(var) = manager.variables.iter_mut().find(|v| v.key == "SECRET_KEY") {
                var.value = "my_secret_value".to_string();
            }
            manager.save().unwrap();
        }

        // Reload and Verify
        {
            let manager = EnvManager::new_with_paths(&example_path, &enc_path).unwrap();
            let var = manager
                .variables
                .iter()
                .find(|v| v.key == "SECRET_KEY")
                .unwrap();
            assert_eq!(var.value, "my_secret_value");
        }

        // Verify file is actually encrypted (not plain text)
        let content = fs::read_to_string(&enc_path).unwrap();
        assert!(!content.contains("my_secret_value"));
    }
}
