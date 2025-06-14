use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use thiserror::Error;

/// Configuration errors
#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("Configuration file error: {0}")]
    FileError(String),
    #[error("Invalid configuration: {0}")]
    ValidationError(String),
    #[error("Path error: {0}")]
    PathError(String),
}

pub type ConfigResult<T> = Result<T, ConfigError>;

/// Main application configuration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct Config {
    pub storage: StorageConfig,
    #[serde(default)]
    pub sync: SyncConfig,
}

/// Storage configuration settings
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StorageConfig {
    /// Directory where bookmark data is stored
    pub data_dir: String,
}


/// Sync configuration settings
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SyncConfig {
    /// Enable sync functionality
    pub enabled: bool,
    /// Default sync server URL
    pub server_url: String,
    /// Connection timeout in seconds
    pub timeout_secs: u64,
    /// Automatically sync after operations (add, delete, etc.)
    pub auto_sync: bool,
    /// Show sync progress in human output mode
    pub show_progress: bool,
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self {
            data_dir: "~/.local/share/automark".to_string(),
        }
    }
}

impl Default for SyncConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            server_url: "wss://sync.automerge.org".to_string(),
            timeout_secs: 30,
            auto_sync: false, // Disabled by default for user control
            show_progress: true,
        }
    }
}

impl Config {
    /// Create a new configuration with default values
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Get the expanded data directory path
    pub fn data_dir_path(&self) -> ConfigResult<PathBuf> {
        expand_path(&self.storage.data_dir)
    }
    
    /// Validate the configuration
    pub fn validate(&self) -> ConfigResult<()> {
        // Validate data directory path
        let data_path = self.data_dir_path()?;
        
        // Check if path is absolute after expansion
        if !data_path.is_absolute() {
            return Err(ConfigError::ValidationError(
                format!("Data directory must be an absolute path: {}", data_path.display())
            ));
        }
        
        Ok(())
    }
    
    /// Generate default configuration file content with comments
    pub fn default_toml_content() -> String {
        r#"# Automark Configuration File
# This file contains configuration settings for the automark bookmark manager.

[storage]
# Directory where bookmark data is stored
# Use ~ for home directory, which will be expanded automatically
data_dir = "~/.local/share/automark"

[sync]
# Enable or disable sync functionality
enabled = true

# Default sync server URL
# The Automerge community server is for development/prototyping only
server_url = "wss://sync.automerge.org"

# Connection timeout in seconds
timeout_secs = 30

# Automatically sync after operations (add, delete, etc.)
# Set to true for seamless collaboration
auto_sync = false

# Show sync progress messages in human output mode
show_progress = true
"#.to_string()
    }
}

/// Expand ~ in paths to the actual home directory
pub fn expand_path(path: &str) -> ConfigResult<PathBuf> {
    if path.starts_with('~') {
        let home_dir = dirs::home_dir()
            .ok_or_else(|| ConfigError::PathError("Could not determine home directory".to_string()))?;
        
        if path == "~" {
            Ok(home_dir)
        } else if let Some(relative_path) = path.strip_prefix("~/") {
            // Remove "~/"
            Ok(home_dir.join(relative_path))
        } else {
            // Handle cases like ~username (not supported)
            Err(ConfigError::PathError(
                format!("Unsupported path format: {}. Only ~ and ~/ are supported.", path)
            ))
        }
    } else {
        Ok(PathBuf::from(path))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_default() {
        let config = Config::default();
        assert_eq!(config.storage.data_dir, "~/.local/share/automark");
    }

    #[test]
    fn test_config_new() {
        let config = Config::new();
        assert_eq!(config, Config::default());
    }

    #[test]
    fn test_storage_config_default() {
        let storage = StorageConfig::default();
        assert_eq!(storage.data_dir, "~/.local/share/automark");
    }

    #[test]
    fn test_config_serialization() {
        let config = Config::default();
        let toml_str = toml::to_string(&config).unwrap();
        
        assert!(toml_str.contains("[storage]"));
        assert!(toml_str.contains("data_dir"));
        assert!(toml_str.contains("~/.local/share/automark"));
        
        // Test deserialization
        let parsed_config: Config = toml::from_str(&toml_str).unwrap();
        assert_eq!(parsed_config, config);
    }

    #[test]
    fn test_expand_path_home_only() {
        let result = expand_path("~");
        assert!(result.is_ok());
        let path = result.unwrap();
        assert!(path.is_absolute());
        assert_eq!(path, dirs::home_dir().unwrap());
    }

    #[test]
    fn test_expand_path_home_relative() {
        let result = expand_path("~/.local/share/automark");
        assert!(result.is_ok());
        let path = result.unwrap();
        assert!(path.is_absolute());
        assert!(path.ends_with(".local/share/automark"));
    }

    #[test]
    fn test_expand_path_absolute() {
        let result = expand_path("/absolute/path");
        assert!(result.is_ok());
        let path = result.unwrap();
        assert_eq!(path, PathBuf::from("/absolute/path"));
    }

    #[test]
    fn test_expand_path_relative() {
        let result = expand_path("relative/path");
        assert!(result.is_ok());
        let path = result.unwrap();
        assert_eq!(path, PathBuf::from("relative/path"));
    }

    #[test]
    fn test_expand_path_unsupported_format() {
        let result = expand_path("~username/path");
        assert!(result.is_err());
        match result {
            Err(ConfigError::PathError(msg)) => {
                assert!(msg.contains("Unsupported path format"));
                assert!(msg.contains("~username/path"));
            }
            _ => panic!("Expected PathError"),
        }
    }

    #[test]
    fn test_config_data_dir_path() {
        let config = Config::default();
        let result = config.data_dir_path();
        assert!(result.is_ok());
        let path = result.unwrap();
        assert!(path.is_absolute());
        assert!(path.ends_with(".local/share/automark"));
    }

    #[test]
    fn test_config_validate_default() {
        let config = Config::default();
        let result = config.validate();
        assert!(result.is_ok());
    }

    #[test]
    fn test_config_validate_relative_path() {
        let mut config = Config::default();
        config.storage.data_dir = "relative/path".to_string();
        
        let result = config.validate();
        assert!(result.is_err());
        match result {
            Err(ConfigError::ValidationError(msg)) => {
                assert!(msg.contains("must be an absolute path"));
            }
            _ => panic!("Expected ValidationError"),
        }
    }

    #[test]
    fn test_config_validate_absolute_path() {
        let mut config = Config::default();
        config.storage.data_dir = "/absolute/path".to_string();
        
        let result = config.validate();
        assert!(result.is_ok());
    }

    #[test]
    fn test_default_toml_content() {
        let content = Config::default_toml_content();
        
        assert!(content.contains("# Automark Configuration File"));
        assert!(content.contains("[storage]"));
        assert!(content.contains("data_dir = \"~/.local/share/automark\""));
        assert!(content.contains("# Directory where bookmark data is stored"));
        
        // Verify it can be parsed as valid TOML
        let parsed: Config = toml::from_str(&content).unwrap();
        assert_eq!(parsed, Config::default());
    }

    #[test]
    fn test_config_equality() {
        let config1 = Config::default();
        let config2 = Config::new();
        assert_eq!(config1, config2);
        
        let mut config3 = Config::default();
        config3.storage.data_dir = "/different/path".to_string();
        assert_ne!(config1, config3);
    }

    #[test]
    fn test_storage_config_equality() {
        let storage1 = StorageConfig::default();
        let storage2 = StorageConfig {
            data_dir: "~/.local/share/automark".to_string(),
        };
        assert_eq!(storage1, storage2);
        
        let storage3 = StorageConfig {
            data_dir: "/different/path".to_string(),
        };
        assert_ne!(storage1, storage3);
    }

    #[test]
    fn test_config_errors_display() {
        let file_error = ConfigError::FileError("test error".to_string());
        assert_eq!(file_error.to_string(), "Configuration file error: test error");
        
        let validation_error = ConfigError::ValidationError("invalid setting".to_string());
        assert_eq!(validation_error.to_string(), "Invalid configuration: invalid setting");
        
        let path_error = ConfigError::PathError("bad path".to_string());
        assert_eq!(path_error.to_string(), "Path error: bad path");
    }
}