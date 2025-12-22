use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use anyhow::{Context, Result};

// Fixed JSON storage path
const JSON_STORAGE_PATH: &str = "~/task-manage/data.json";

#[derive(Debug, Clone, PartialEq)]
pub enum StorageConfig {
    Database { url: String },
    Json { path: PathBuf },
}

// Custom serialization/deserialization
impl Serialize for StorageConfig {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        match self {
            StorageConfig::Database { url } => {
                let mut state = serializer.serialize_struct("StorageConfig", 2)?;
                state.serialize_field("type", "database")?;
                state.serialize_field("url", url)?;
                state.end()
            }
            StorageConfig::Json { path } => {
                let mut state = serializer.serialize_struct("StorageConfig", 2)?;
                state.serialize_field("type", "json")?;
                state.serialize_field("path", &path.to_string_lossy())?;
                state.end()
            }
        }
    }
}

impl<'de> Deserialize<'de> for StorageConfig {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(tag = "type")]
        enum Helper {
            #[serde(rename = "json")]
            Json { path: Option<String> },
            #[serde(rename = "database")]
            Database { url: String },
        }

        match Helper::deserialize(deserializer)? {
            Helper::Json { path } => {
                let path = path.map(PathBuf::from).unwrap_or_else(|| PathBuf::from(JSON_STORAGE_PATH));
                Ok(StorageConfig::Json { path })
            }
            Helper::Database { url } => Ok(StorageConfig::Database { url }),
        }
    }
}

impl StorageConfig {
    /// Get the JSON storage config with default path
    pub fn json() -> Self {
        let expanded = shellexpand::full(JSON_STORAGE_PATH)
            .expect("Expanding default JSON path should not fail");
        StorageConfig::Json {
            path: PathBuf::from(expanded.as_ref()),
        }
    }

    /// Expand environment variables and tilde in paths
    pub fn expand_paths(&mut self) -> Result<()> {
        match self {
            StorageConfig::Database { url } => {
                *url = shellexpand::full(url)
                    .context("Failed to expand database URL")?
                    .into_owned();
            }
            StorageConfig::Json { path } => {
                let path_str = path.to_string_lossy().to_string();
                let expanded = shellexpand::full(&path_str)
                    .context("Failed to expand JSON path")?;
                *path = PathBuf::from(expanded.as_ref());
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Config {
    pub storage: StorageConfig,
}

impl Config {
    /// Load configuration from default location (config.toml)
    pub fn load() -> Result<Self> {
        Self::from_file("config.toml")
    }

    /// Load configuration from specified file
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = std::fs::read_to_string(path.as_ref())
            .with_context(|| format!("Failed to read config file: {}", path.as_ref().display()))?;

        Self::parse(&content)
    }

    /// Parse configuration from TOML string with environment variable expansion
    pub fn parse(content: &str) -> Result<Self> {
        // Expand environment variables in the content
        let expanded = Self::expand_env_vars(content)?;

        let mut config: Config = toml::from_str(&expanded)
            .context("Failed to parse TOML configuration")?;

        // Expand paths for all storage types
        config.storage.expand_paths()?;

        Ok(config)
    }

    /// Expand environment variables in format ${VAR_NAME}
    fn expand_env_vars(content: &str) -> Result<String> {
        shellexpand::full(content)
            .map(|s| s.into_owned())
            .context("Failed to expand environment variables")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_json_storage_config() {
        let toml_str = r#"
            [storage]
            type = "json"
        "#;

        let config = Config::parse(toml_str).unwrap();
        assert!(matches!(config.storage, StorageConfig::Json { .. }));

        // Verify path is set to fixed location
        if let StorageConfig::Json { path } = config.storage {
            assert!(path.to_string_lossy().contains("task-manage/data.json"));
        }
    }

    #[test]
    fn test_parse_database_storage_config() {
        let toml_str = r#"
            [storage]
            type = "database"
            url = "postgresql://localhost/test"
        "#;

        let config: Config = toml::from_str(toml_str).expect("Failed to parse TOML");

        match config.storage {
            StorageConfig::Database { url } => {
                assert_eq!(url, "postgresql://localhost/test");
            }
            _ => panic!("Expected Database storage config"),
        }

        let config = Config::parse(toml_str).unwrap();
        assert!(matches!(config.storage, StorageConfig::Database { .. }));
    }

    #[test]
    #[serial_test::serial]
    fn test_env_var_expansion() {
        // SAFETY: This test uses serial_test::serial to ensure no concurrent execution
        unsafe {
            std::env::set_var("TEST_DB_URL", "postgresql://test:5432/db");
        }

        let toml_str = r#"
            [storage]
            type = "database"
            url = "${TEST_DB_URL}"
        "#;

        let config = Config::parse(toml_str).unwrap();
        if let StorageConfig::Database { url } = config.storage {
            assert_eq!(url, "postgresql://test:5432/db");
        } else {
            panic!("Expected Database storage");
        }

        // SAFETY: This test uses serial_test::serial to ensure no concurrent execution
        unsafe {
            std::env::remove_var("TEST_DB_URL");
        }
    }

    #[test]
    #[serial_test::serial]
    fn test_env_var_in_path() {
        // SAFETY: This test uses serial_test::serial to ensure no concurrent execution
        unsafe {
            std::env::set_var("DB_HOST", "myhost");
        }

        let toml_str = r#"
            [storage]
            type = "database"
            url = "postgresql://${DB_HOST}:5432/db"
        "#;

        let config = Config::parse(toml_str).unwrap();
        if let StorageConfig::Database { url } = config.storage {
            assert_eq!(url, "postgresql://myhost:5432/db");
        } else {
            panic!("Expected Database storage");
        }

        // SAFETY: This test uses serial_test::serial to ensure no concurrent execution
        unsafe {
            std::env::remove_var("DB_HOST");
        }
    }

    #[test]
    fn test_missing_env_var() {
        let toml_str = r#"
            [storage]
            type = "database"
            url = "${NONEXISTENT_VAR}"
        "#;

        // Should fail because environment variable doesn't exist
        assert!(Config::parse(toml_str).is_err());
    }

    #[test]
    fn test_invalid_storage_type() {
        let toml_str = r#"
            [storage]
            type = "invalid"
        "#;

        assert!(Config::parse(toml_str).is_err());
    }

    #[test]
    fn test_missing_required_field() {
        let toml_str = r#"
            [storage]
            type = "database"
        "#;

        // Should fail because 'url' is required for database type
        assert!(Config::parse(toml_str).is_err());
    }
}
