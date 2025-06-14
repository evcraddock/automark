use crate::traits::BookmarkRepository;
use crate::types::{BookmarkResult, Config};
use super::{OutputFormat, sync::{handle_sync_command, SyncArgs}};

/// Automatically sync if enabled in configuration
pub async fn auto_sync_if_enabled(
    repository: &mut dyn BookmarkRepository,
    config: &Config,
    format: OutputFormat,
) -> BookmarkResult<()> {
    // Only auto-sync if it's enabled and sync is enabled
    if !config.sync.enabled || !config.sync.auto_sync {
        return Ok(());
    }
    
    // Create default sync args for auto-sync
    let sync_args = SyncArgs {
        server: None, // Use config default
        document_id: None, // Use default document
        dry_run: false, // Don't dry run for auto-sync
        timeout: None, // Use config timeout
    };
    
    // Perform sync but suppress output unless there's an error
    let silent_format = match format {
        OutputFormat::Human => {
            if config.sync.show_progress {
                format
            } else {
                // TODO: Add a "silent" format that only shows errors
                format
            }
        }
        OutputFormat::Json => format, // Keep JSON output as-is
    };
    
    match handle_sync_command(&sync_args, repository, config, silent_format).await {
        Ok(()) => {
            // Successful auto-sync
            if format == OutputFormat::Human && config.sync.show_progress {
                println!("📡 Auto-sync completed");
            }
            Ok(())
        }
        Err(e) => {
            // Auto-sync failed - show warning but don't fail the operation
            if format == OutputFormat::Human {
                eprintln!("⚠️  Auto-sync failed: {}", e);
                eprintln!("   Your changes are saved locally. Run 'automark sync' to retry.");
            }
            // Don't propagate the error - the main operation should still succeed
            Ok(())
        }
    }
}

/// Quietly sync without showing output (for background sync)
#[allow(dead_code)]
pub async fn quiet_sync(
    repository: &mut dyn BookmarkRepository,
    config: &Config,
) -> BookmarkResult<bool> {
    if !config.sync.enabled {
        return Ok(false);
    }
    
    let sync_args = SyncArgs {
        server: None,
        document_id: None,
        dry_run: false,
        timeout: None,
    };
    
    // Use JSON format to suppress human output
    match handle_sync_command(&sync_args, repository, config, OutputFormat::Json).await {
        Ok(()) => Ok(true),
        Err(_) => Ok(false), // Failed but don't propagate error
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::repository::MockBookmarkRepository;
    use crate::types::Config;
    
    #[tokio::test]
    async fn test_auto_sync_disabled() {
        let mut repo = MockBookmarkRepository::new();
        let mut config = Config::default();
        config.sync.auto_sync = false;
        
        let result = auto_sync_if_enabled(&mut repo, &config, OutputFormat::Human).await;
        assert!(result.is_ok());
    }
    
    #[tokio::test]
    async fn test_auto_sync_sync_disabled() {
        let mut repo = MockBookmarkRepository::new();
        let mut config = Config::default();
        config.sync.enabled = false;
        config.sync.auto_sync = true;
        
        let result = auto_sync_if_enabled(&mut repo, &config, OutputFormat::Human).await;
        assert!(result.is_ok());
    }
    
    #[tokio::test]
    async fn test_quiet_sync_disabled() {
        let mut repo = MockBookmarkRepository::new();
        let mut config = Config::default();
        config.sync.enabled = false;
        
        let result = quiet_sync(&mut repo, &config).await;
        assert!(result.is_ok());
        assert!(!result.unwrap()); // Should return false when disabled
    }
}