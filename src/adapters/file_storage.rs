use crate::types::{Config, ConfigError, ConfigResult};
use std::fs;
use std::path::{Path, PathBuf};

pub struct FileStorageManager;

impl FileStorageManager {
    /// Load configuration from file system
    pub fn load_config() -> ConfigResult<Config> {
        let config_path = Self::get_config_file_path()?;
        
        if config_path.exists() {
            Self::load_config_from_file(&config_path)
        } else {
            Self::create_default_config(&config_path)
        }
    }
    
    /// Get the configuration file path
    pub fn get_config_file_path() -> ConfigResult<PathBuf> {
        let config_dir = dirs::config_dir()
            .ok_or_else(|| ConfigError::PathError("Could not determine config directory".to_string()))?;
        
        let automark_config_dir = config_dir.join("automark");
        Ok(automark_config_dir.join("config.toml"))
    }
    
    /// Load configuration from a specific file
    fn load_config_from_file(path: &Path) -> ConfigResult<Config> {
        let content = fs::read_to_string(path)
            .map_err(|e| ConfigError::FileError(format!("Failed to read config file: {}", e)))?;
        
        let config: Config = toml::from_str(&content)
            .map_err(|e| ConfigError::FileError(format!("Failed to parse config file: {}", e)))?;
        
        // Validate the loaded configuration
        config.validate()?;
        
        Ok(config)
    }
    
    /// Create default configuration file
    fn create_default_config(config_path: &Path) -> ConfigResult<Config> {
        let config = Config::default();
        
        // Create parent directories if they don't exist
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| ConfigError::FileError(format!("Failed to create config directory: {}", e)))?;
        }
        
        // Write default configuration file with comments
        let content = Config::default_toml_content();
        fs::write(config_path, content)
            .map_err(|e| ConfigError::FileError(format!("Failed to write default config file: {}", e)))?;
        
        Ok(config)
    }
    
    /// Ensure data directory exists with proper permissions
    pub fn ensure_data_directory(config: &Config) -> ConfigResult<PathBuf> {
        let data_path = config.data_dir_path()?;
        
        if !data_path.exists() {
            fs::create_dir_all(&data_path)
                .map_err(|e| ConfigError::FileError(format!("Failed to create data directory: {}", e)))?;
        }
        
        // Verify the directory is accessible
        Self::verify_directory_access(&data_path)?;
        
        Ok(data_path)
    }
    
    /// Verify directory has read/write permissions
    fn verify_directory_access(path: &Path) -> ConfigResult<()> {
        // Check if directory exists and is actually a directory
        if !path.exists() {
            return Err(ConfigError::ValidationError(
                format!("Data directory does not exist: {}", path.display())
            ));
        }
        
        if !path.is_dir() {
            return Err(ConfigError::ValidationError(
                format!("Data path is not a directory: {}", path.display())
            ));
        }
        
        // Test write access by creating a temporary file
        let test_file = path.join(".automark_test");
        match fs::write(&test_file, "test") {
            Ok(_) => {
                // Clean up test file
                let _ = fs::remove_file(&test_file);
                Ok(())
            }
            Err(e) => Err(ConfigError::ValidationError(
                format!("Data directory is not writable: {}", e)
            )),
        }
    }
    
    /// Get the full path to the bookmark data file
    pub fn get_bookmark_file_path(config: &Config) -> ConfigResult<PathBuf> {
        let data_dir = config.data_dir_path()?;
        Ok(data_dir.join("bookmarks.automerge"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::{TempDir, NamedTempFile};
    use std::os::unix::fs::PermissionsExt;

    #[test]
    fn test_get_config_file_path() {
        let result = FileStorageManager::get_config_file_path();
        assert!(result.is_ok());
        
        let path = result.unwrap();
        assert!(path.ends_with("automark/config.toml"));
        assert!(path.is_absolute());
    }

    #[test]
    fn test_load_config_from_file_valid() {
        let temp_file = NamedTempFile::new().unwrap();
        let content = r#"
[storage]
data_dir = "/tmp/test"
"#;
        fs::write(temp_file.path(), content).unwrap();
        
        let result = FileStorageManager::load_config_from_file(temp_file.path());
        assert!(result.is_ok());
        
        let config = result.unwrap();
        assert_eq!(config.storage.data_dir, "/tmp/test");
    }

    #[test]
    fn test_load_config_from_file_invalid_toml() {
        let temp_file = NamedTempFile::new().unwrap();
        let content = "invalid toml content [[[";
        fs::write(temp_file.path(), content).unwrap();
        
        let result = FileStorageManager::load_config_from_file(temp_file.path());
        assert!(result.is_err());
        
        match result {
            Err(ConfigError::FileError(msg)) => {
                assert!(msg.contains("Failed to parse config file"));
            }
            _ => panic!("Expected FileError"),
        }
    }

    #[test]
    fn test_load_config_from_file_nonexistent() {
        let result = FileStorageManager::load_config_from_file(Path::new("/nonexistent/file"));
        assert!(result.is_err());
        
        match result {
            Err(ConfigError::FileError(msg)) => {
                assert!(msg.contains("Failed to read config file"));
            }
            _ => panic!("Expected FileError"),
        }
    }

    #[test]
    fn test_load_config_from_file_invalid_config() {
        let temp_file = NamedTempFile::new().unwrap();
        let content = r#"
[storage]
data_dir = "relative/path"
"#;
        fs::write(temp_file.path(), content).unwrap();
        
        let result = FileStorageManager::load_config_from_file(temp_file.path());
        assert!(result.is_err());
        
        match result {
            Err(ConfigError::ValidationError(msg)) => {
                assert!(msg.contains("must be an absolute path"));
            }
            _ => panic!("Expected ValidationError"),
        }
    }

    #[test]
    fn test_create_default_config() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.toml");
        
        let result = FileStorageManager::create_default_config(&config_path);
        assert!(result.is_ok());
        
        let config = result.unwrap();
        assert_eq!(config, Config::default());
        
        // Verify file was created
        assert!(config_path.exists());
        
        // Verify file content
        let content = fs::read_to_string(&config_path).unwrap();
        assert!(content.contains("# Automark Configuration File"));
        assert!(content.contains("[storage]"));
        assert!(content.contains("data_dir = \"~/.local/share/automark\""));
    }

    #[test]
    fn test_create_default_config_with_nested_dirs() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("nested").join("dirs").join("config.toml");
        
        let result = FileStorageManager::create_default_config(&config_path);
        assert!(result.is_ok());
        
        // Verify directories were created
        assert!(config_path.parent().unwrap().exists());
        assert!(config_path.exists());
    }

    #[test]
    fn test_ensure_data_directory_creates_missing() {
        let temp_dir = TempDir::new().unwrap();
        let mut config = Config::default();
        config.storage.data_dir = temp_dir.path().join("data").to_string_lossy().to_string();
        
        let result = FileStorageManager::ensure_data_directory(&config);
        assert!(result.is_ok());
        
        let data_path = result.unwrap();
        assert!(data_path.exists());
        assert!(data_path.is_dir());
    }

    #[test]
    fn test_ensure_data_directory_existing() {
        let temp_dir = TempDir::new().unwrap();
        let data_dir = temp_dir.path().join("existing_data");
        fs::create_dir(&data_dir).unwrap();
        
        let mut config = Config::default();
        config.storage.data_dir = data_dir.to_string_lossy().to_string();
        
        let result = FileStorageManager::ensure_data_directory(&config);
        assert!(result.is_ok());
        
        let data_path = result.unwrap();
        assert_eq!(data_path, data_dir);
    }

    #[test]
    fn test_verify_directory_access_valid() {
        let temp_dir = TempDir::new().unwrap();
        
        let result = FileStorageManager::verify_directory_access(temp_dir.path());
        assert!(result.is_ok());
    }

    #[test]
    fn test_verify_directory_access_nonexistent() {
        let result = FileStorageManager::verify_directory_access(Path::new("/nonexistent/directory"));
        assert!(result.is_err());
        
        match result {
            Err(ConfigError::ValidationError(msg)) => {
                assert!(msg.contains("does not exist"));
            }
            _ => panic!("Expected ValidationError"),
        }
    }

    #[test]
    fn test_verify_directory_access_file_not_dir() {
        let temp_file = NamedTempFile::new().unwrap();
        
        let result = FileStorageManager::verify_directory_access(temp_file.path());
        assert!(result.is_err());
        
        match result {
            Err(ConfigError::ValidationError(msg)) => {
                assert!(msg.contains("is not a directory"));
            }
            _ => panic!("Expected ValidationError"),
        }
    }

    #[cfg(unix)]
    #[test]
    fn test_verify_directory_access_readonly() {
        let temp_dir = TempDir::new().unwrap();
        
        // Make directory read-only
        let mut perms = temp_dir.path().metadata().unwrap().permissions();
        perms.set_mode(0o444);
        fs::set_permissions(temp_dir.path(), perms).unwrap();
        
        let result = FileStorageManager::verify_directory_access(temp_dir.path());
        assert!(result.is_err());
        
        match result {
            Err(ConfigError::ValidationError(msg)) => {
                assert!(msg.contains("not writable"));
            }
            _ => panic!("Expected ValidationError"),
        }
        
        // Restore permissions for cleanup
        let mut perms = temp_dir.path().metadata().unwrap().permissions();
        perms.set_mode(0o755);
        fs::set_permissions(temp_dir.path(), perms).unwrap();
    }

    #[test]
    fn test_get_bookmark_file_path() {
        let config = Config::default();
        
        let result = FileStorageManager::get_bookmark_file_path(&config);
        assert!(result.is_ok());
        
        let path = result.unwrap();
        assert!(path.ends_with("bookmarks.automerge"));
        assert!(path.is_absolute());
    }

    #[test]
    fn test_get_bookmark_file_path_custom_data_dir() {
        let temp_dir = TempDir::new().unwrap();
        let mut config = Config::default();
        config.storage.data_dir = temp_dir.path().to_string_lossy().to_string();
        
        let result = FileStorageManager::get_bookmark_file_path(&config);
        assert!(result.is_ok());
        
        let path = result.unwrap();
        assert_eq!(path, temp_dir.path().join("bookmarks.automerge"));
    }

    #[test]
    fn test_load_config_creates_default_when_missing() {
        // This test needs to mock the config directory
        // For now, we'll test the individual components
        
        // Test that default config creation works
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.toml");
        
        let config = FileStorageManager::create_default_config(&config_path).unwrap();
        assert_eq!(config, Config::default());
        
        // Test that loading the created config works
        let loaded_config = FileStorageManager::load_config_from_file(&config_path).unwrap();
        assert_eq!(loaded_config.storage.data_dir, "~/.local/share/automark");
    }
}