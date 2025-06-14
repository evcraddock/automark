# Task 7: Implement Configuration Management System

**GitHub Issue**: [#7](https://github.com/evcraddock/automark/issues/7)

## Objective
Create a robust configuration system using TOML files with automatic initialization and validation.

## Requirements

1. **Create configuration types** in `src/types/config.rs`:
   - Config struct with storage settings
   - StorageConfig struct with data_dir field
   - Future-extensible structure for additional settings
   - Implement serde derives for TOML serialization

2. **Implement configuration loading** in `src/adapters/file_storage.rs`:
   - Load from `~/.config/automark/config.toml`
   - Create default config file if missing
   - Expand `~` to actual home directory path
   - Validate configuration values on load

3. **Add configuration validation**:
   - Ensure data_dir is valid and accessible
   - Create data directory if it doesn't exist
   - Validate path permissions for read/write access
   - Handle configuration errors gracefully

4. **Implement default configuration**:
   - Default data_dir: `~/.local/share/automark`
   - Default config file creation with comments
   - Extensible structure for future settings
   - Cross-platform path handling

5. **Add configuration management dependencies**:
   - Add `config` crate for TOML parsing
   - Add `dirs` crate for directory discovery
   - Ensure cross-platform compatibility
   - Handle configuration file encoding properly

6. **Update main.rs integration**:
   - Load configuration before repository initialization
   - Pass configuration to repository constructor
   - Handle configuration errors at startup
   - Provide helpful error messages for config issues

7. **Create configuration utilities**:
   - Helper functions for path expansion
   - Directory creation with proper permissions
   - Configuration validation functions
   - Error conversion for configuration failures

8. **Write comprehensive tests** using TDD approach:
   - Test configuration loading from existing files
   - Test default configuration creation
   - Test path expansion and validation
   - Test error handling for invalid configurations
   - Use temporary directories for testing

## Success Criteria
- Configuration loads reliably from TOML files
- Default configuration creates properly
- Path handling works cross-platform
- Error messages are helpful and actionable
- Integration with repository works smoothly