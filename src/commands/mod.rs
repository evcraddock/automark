use clap::{Parser, Subcommand, Args};
use crate::traits::BookmarkRepository;
use crate::types::BookmarkResult;
use async_trait::async_trait;
use serde::{Serialize, Deserialize};

pub mod add;
pub mod list;
pub mod delete;
pub mod search;
pub mod sync;

pub use add::handle_add_command;
pub use list::handle_list_command;
pub use delete::handle_delete_command;
pub use search::handle_search_command;
pub use sync::handle_sync_command;

/// Output format for CLI responses
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputFormat {
    /// Human-readable output
    Human,
    /// JSON output
    Json,
}


/// Standard JSON response wrapper
#[derive(Serialize, Deserialize, Debug)]
pub struct JsonResponse<T> {
    /// Whether the operation was successful
    pub success: bool,
    /// The data payload (only present on success)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
    /// Error information (only present on failure)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonError>,
    /// API schema version
    pub version: &'static str,
}

/// JSON error format
#[derive(Serialize, Deserialize, Debug)]
pub struct JsonError {
    /// Error code for programmatic handling
    pub code: &'static str,
    /// Human-readable error message
    pub message: String,
    /// Additional context details
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
}

impl<T> JsonResponse<T> {
    /// Create a successful response
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
            version: "1.0",
        }
    }
    
    /// Create an error response
    pub fn error(code: &'static str, message: String) -> JsonResponse<()> {
        JsonResponse {
            success: false,
            data: None,
            error: Some(JsonError {
                code,
                message,
                details: None,
            }),
            version: "1.0",
        }
    }
}

/// Output formatting utilities
pub mod output {
    use super::*;
    
    /// Print response in the specified format
    pub fn print_response<T: Serialize>(format: OutputFormat, data: T) -> BookmarkResult<()> {
        match format {
            OutputFormat::Json => {
                let response = JsonResponse::success(data);
                println!("{}", serde_json::to_string_pretty(&response)
                    .map_err(|e| crate::types::BookmarkError::InvalidUrl(format!("JSON serialization error: {}", e)))?);
            }
            OutputFormat::Human => {
                // Human output is handled by each command individually
                // This function is primarily for JSON output
            }
        }
        Ok(())
    }
    
    /// Print error in the specified format
    pub fn print_error(format: OutputFormat, error: &crate::types::BookmarkError) {
        match format {
            OutputFormat::Json => {
                let (code, message) = error_to_json_fields(error);
                let response = JsonResponse::<()>::error(code, message);
                if let Ok(json) = serde_json::to_string_pretty(&response) {
                    println!("{}", json);
                } else {
                    eprintln!("{{\"success\": false, \"error\": {{\"code\": \"SERIALIZATION_ERROR\", \"message\": \"Failed to serialize error response\"}}}}");
                }
            }
            OutputFormat::Human => {
                eprintln!("Error: {}", error);
            }
        }
    }
    
    pub fn error_to_json_fields(error: &crate::types::BookmarkError) -> (&'static str, String) {
        match error {
            crate::types::BookmarkError::InvalidUrl(_) => ("INVALID_URL", error.to_string()),
            crate::types::BookmarkError::EmptyTitle => ("EMPTY_TITLE", error.to_string()),
            crate::types::BookmarkError::NotFound(_) => ("NOT_FOUND", error.to_string()),
            crate::types::BookmarkError::InvalidId(_) => ("INVALID_ID", error.to_string()),
            crate::types::BookmarkError::MetadataExtraction(_) => ("METADATA_EXTRACTION_ERROR", error.to_string()),
            crate::types::BookmarkError::SyncError(_) => ("SYNC_ERROR", error.to_string()),
        }
    }
}

#[derive(Parser)]
#[command(name = "automark")]
#[command(about = "A local-first CLI bookmarking application")]
#[command(version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
    /// Output format
    #[arg(short = 'o', long = "output", value_enum, default_value = "human", global = true)]
    pub output: OutputFormatArg,
}

#[derive(clap::ValueEnum, Clone, Debug)]
pub enum OutputFormatArg {
    /// Human-readable output
    Human,
    /// JSON output
    Json,
}

impl From<OutputFormatArg> for OutputFormat {
    fn from(arg: OutputFormatArg) -> Self {
        match arg {
            OutputFormatArg::Human => Self::Human,
            OutputFormatArg::Json => Self::Json,
        }
    }
}

#[derive(Subcommand)]
pub enum Commands {
    /// Add a new bookmark
    Add(AddArgs),
    /// List all bookmarks
    List,
    /// Delete a bookmark by ID
    Delete(DeleteArgs),
    /// Search bookmarks with advanced filtering
    Search(search::SearchArgs),
    /// Sync bookmarks with a remote server
    Sync(sync::SyncArgs),
}

#[derive(Args)]
pub struct AddArgs {
    /// URL to bookmark
    pub url: String,
    /// Title for the bookmark (optional, will be extracted from page if not provided)
    pub title: Option<String>,
    /// Skip metadata extraction and prompt for title if not provided
    #[arg(long)]
    pub no_fetch: bool,
}

#[derive(Args)]
pub struct DeleteArgs {
    /// ID of bookmark to delete (can be partial ID)
    pub id: String,
}

#[async_trait]
pub trait CommandHandler {
    async fn execute(&self, repository: &mut dyn BookmarkRepository, format: OutputFormat) -> BookmarkResult<()>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::error::ErrorKind;

    #[test]
    fn test_add_command_parsing() {
        let cli = Cli::try_parse_from(&["automark", "add", "https://example.com", "Example Title"]);
        assert!(cli.is_ok());
        
        if let Ok(Cli { command: Commands::Add(args), .. }) = cli {
            assert_eq!(args.url, "https://example.com");
            assert_eq!(args.title, Some("Example Title".to_string()));
            assert_eq!(args.no_fetch, false);
        } else {
            panic!("Expected Add command");
        }
    }

    #[test]
    fn test_list_command_parsing() {
        let cli = Cli::try_parse_from(&["automark", "list"]);
        assert!(cli.is_ok());
        
        if let Ok(Cli { command: Commands::List, .. }) = cli {
            // Success
        } else {
            panic!("Expected List command");
        }
    }

    #[test]
    fn test_delete_command_parsing() {
        let cli = Cli::try_parse_from(&["automark", "delete", "abc123"]);
        assert!(cli.is_ok());
        
        if let Ok(Cli { command: Commands::Delete(args), .. }) = cli {
            assert_eq!(args.id, "abc123");
        } else {
            panic!("Expected Delete command");
        }
    }

    #[test]
    fn test_missing_arguments() {
        // Missing URL for add command
        let cli = Cli::try_parse_from(&["automark", "add"]);
        assert!(cli.is_err());
        
        // Missing ID for delete command
        let cli = Cli::try_parse_from(&["automark", "delete"]);
        assert!(cli.is_err());
    }

    #[test]
    fn test_help_output() {
        let cli = Cli::try_parse_from(&["automark", "--help"]);
        match cli {
            Err(err) => {
                assert_eq!(err.kind(), ErrorKind::DisplayHelp);
                let help_output = err.to_string();
                assert!(help_output.contains("A local-first CLI bookmarking application"));
                assert!(help_output.contains("add"));
                assert!(help_output.contains("list"));
                assert!(help_output.contains("delete"));
            }
            _ => panic!("Expected help error"),
        }
    }

    #[test]
    fn test_version_output() {
        let cli = Cli::try_parse_from(&["automark", "--version"]);
        match cli {
            Err(err) => {
                assert_eq!(err.kind(), ErrorKind::DisplayVersion);
            }
            _ => panic!("Expected version error"),
        }
    }

    #[test]
    fn test_invalid_command() {
        let cli = Cli::try_parse_from(&["automark", "invalid"]);
        assert!(cli.is_err());
    }

    #[test]
    fn test_add_with_spaces_in_title() {
        let cli = Cli::try_parse_from(&["automark", "add", "https://example.com", "Multi Word Title"]);
        assert!(cli.is_ok());
        
        if let Ok(Cli { command: Commands::Add(args), .. }) = cli {
            assert_eq!(args.title, Some("Multi Word Title".to_string()));
        }
    }

    #[test]
    fn test_add_without_title() {
        let cli = Cli::try_parse_from(&["automark", "add", "https://example.com"]);
        assert!(cli.is_ok());
        
        if let Ok(Cli { command: Commands::Add(args), .. }) = cli {
            assert_eq!(args.url, "https://example.com");
            assert_eq!(args.title, None);
            assert_eq!(args.no_fetch, false);
        } else {
            panic!("Expected Add command");
        }
    }

    #[test]
    fn test_add_with_no_fetch_flag() {
        let cli = Cli::try_parse_from(&["automark", "add", "https://example.com", "--no-fetch"]);
        assert!(cli.is_ok());
        
        if let Ok(Cli { command: Commands::Add(args), .. }) = cli {
            assert_eq!(args.url, "https://example.com");
            assert_eq!(args.title, None);
            assert_eq!(args.no_fetch, true);
        } else {
            panic!("Expected Add command");
        }
    }

    #[test]
    fn test_add_with_title_and_no_fetch_flag() {
        let cli = Cli::try_parse_from(&["automark", "add", "https://example.com", "Title", "--no-fetch"]);
        assert!(cli.is_ok());
        
        if let Ok(Cli { command: Commands::Add(args), .. }) = cli {
            assert_eq!(args.url, "https://example.com");
            assert_eq!(args.title, Some("Title".to_string()));
            assert_eq!(args.no_fetch, true);
        } else {
            panic!("Expected Add command");
        }
    }

    #[test]
    fn test_output_format_parsing() {
        // Test default output format (human)
        let cli = Cli::try_parse_from(&["automark", "list"]);
        assert!(cli.is_ok());
        if let Ok(cli) = cli {
            assert!(matches!(cli.output, OutputFormatArg::Human));
        }

        // Test with short flag
        let cli = Cli::try_parse_from(&["automark", "-o", "json", "list"]);
        assert!(cli.is_ok());
        if let Ok(cli) = cli {
            assert!(matches!(cli.output, OutputFormatArg::Json));
        }

        // Test with long flag
        let cli = Cli::try_parse_from(&["automark", "--output", "json", "list"]);
        assert!(cli.is_ok());
        if let Ok(cli) = cli {
            assert!(matches!(cli.output, OutputFormatArg::Json));
        }

        // Test explicit human format
        let cli = Cli::try_parse_from(&["automark", "--output", "human", "list"]);
        assert!(cli.is_ok());
        if let Ok(cli) = cli {
            assert!(matches!(cli.output, OutputFormatArg::Human));
        }

        // Test output flag with add command
        let cli = Cli::try_parse_from(&["automark", "-o", "json", "add", "https://example.com", "Test"]);
        assert!(cli.is_ok());
        if let Ok(cli) = cli {
            assert!(matches!(cli.output, OutputFormatArg::Json));
            if let Commands::Add(args) = cli.command {
                assert_eq!(args.url, "https://example.com");
                assert_eq!(args.title, Some("Test".to_string()));
            }
        }
    }

    #[test]
    fn test_output_format_from_arg() {
        assert_eq!(OutputFormat::from(OutputFormatArg::Human), OutputFormat::Human);
        assert_eq!(OutputFormat::from(OutputFormatArg::Json), OutputFormat::Json);
    }

    #[test]
    fn test_json_response_success() {
        let data = "test data";
        let response = JsonResponse::success(data);
        
        assert_eq!(response.success, true);
        assert_eq!(response.data, Some("test data"));
        assert!(response.error.is_none());
        assert_eq!(response.version, "1.0");
    }

    #[test]
    fn test_json_response_error() {
        let response = JsonResponse::<()>::error("TEST_ERROR", "Test error message".to_string());
        
        assert_eq!(response.success, false);
        assert!(response.data.is_none());
        assert!(response.error.is_some());
        
        if let Some(error) = response.error {
            assert_eq!(error.code, "TEST_ERROR");
            assert_eq!(error.message, "Test error message");
            assert!(error.details.is_none());
        }
        assert_eq!(response.version, "1.0");
    }

    #[test]
    fn test_error_to_json_mapping() {
        use crate::types::BookmarkError;
        use super::output::error_to_json_fields;
        
        let invalid_url = BookmarkError::InvalidUrl("bad-url".to_string());
        let (code, _) = error_to_json_fields(&invalid_url);
        assert_eq!(code, "INVALID_URL");
        
        let not_found = BookmarkError::NotFound("123".to_string());
        let (code, _) = error_to_json_fields(&not_found);
        assert_eq!(code, "NOT_FOUND");
        
        let empty_title = BookmarkError::EmptyTitle;
        let (code, _) = error_to_json_fields(&empty_title);
        assert_eq!(code, "EMPTY_TITLE");
        
        let invalid_id = BookmarkError::InvalidId("ambiguous".to_string());
        let (code, _) = error_to_json_fields(&invalid_id);
        assert_eq!(code, "INVALID_ID");
    }
}