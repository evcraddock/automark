use clap::{Parser, Subcommand, Args};
use crate::traits::BookmarkRepository;
use crate::types::BookmarkResult;
use async_trait::async_trait;

pub mod add;
pub mod list;
pub mod delete;

pub use add::handle_add_command;
pub use list::handle_list_command;
pub use delete::handle_delete_command;

#[derive(Parser)]
#[command(name = "automark")]
#[command(about = "A local-first CLI bookmarking application")]
#[command(version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Add a new bookmark
    Add(AddArgs),
    /// List all bookmarks
    List,
    /// Delete a bookmark by ID
    Delete(DeleteArgs),
}

#[derive(Args)]
pub struct AddArgs {
    /// URL to bookmark
    pub url: String,
    /// Title for the bookmark
    pub title: String,
}

#[derive(Args)]
pub struct DeleteArgs {
    /// ID of bookmark to delete (can be partial ID)
    pub id: String,
}

#[async_trait]
pub trait CommandHandler {
    async fn execute(&self, repository: &mut dyn BookmarkRepository) -> BookmarkResult<()>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::error::ErrorKind;

    #[test]
    fn test_add_command_parsing() {
        let cli = Cli::try_parse_from(&["automark", "add", "https://example.com", "Example Title"]);
        assert!(cli.is_ok());
        
        if let Ok(Cli { command: Commands::Add(args) }) = cli {
            assert_eq!(args.url, "https://example.com");
            assert_eq!(args.title, "Example Title");
        } else {
            panic!("Expected Add command");
        }
    }

    #[test]
    fn test_list_command_parsing() {
        let cli = Cli::try_parse_from(&["automark", "list"]);
        assert!(cli.is_ok());
        
        if let Ok(Cli { command: Commands::List }) = cli {
            // Success
        } else {
            panic!("Expected List command");
        }
    }

    #[test]
    fn test_delete_command_parsing() {
        let cli = Cli::try_parse_from(&["automark", "delete", "abc123"]);
        assert!(cli.is_ok());
        
        if let Ok(Cli { command: Commands::Delete(args) }) = cli {
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
        
        if let Ok(Cli { command: Commands::Add(args) }) = cli {
            assert_eq!(args.title, "Multi Word Title");
        }
    }
}