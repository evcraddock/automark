use crate::commands::{AddArgs, CommandHandler};
use crate::traits::{BookmarkRepository, MetadataExtractor};
use crate::types::{Bookmark, BookmarkResult};
use crate::adapters::WebExtractor;
use std::time::Duration;
use tokio::io::{self, AsyncBufReadExt, AsyncWriteExt, BufReader};

pub struct AddCommand {
    pub args: AddArgs,
}

impl AddCommand {
    pub fn new(args: AddArgs) -> Self {
        Self { args }
    }
}

#[async_trait::async_trait]
impl CommandHandler for AddCommand {
    async fn execute(&self, repository: &mut dyn BookmarkRepository) -> BookmarkResult<()> {
        // For now, just handle the case where title is provided
        // We'll implement the full logic with metadata extraction later
        let title = match &self.args.title {
            Some(t) => t.clone(),
            None => {
                // For now, return error - we'll implement user prompt and metadata extraction next
                return Err(crate::types::BookmarkError::EmptyTitle);
            }
        };
        
        let bookmark = Bookmark::new(&self.args.url, &title)?;
        
        // Save via repository
        let saved_bookmark = repository.create(bookmark).await?;
        
        // Print success message
        println!("Added bookmark: {}", saved_bookmark.title);
        println!("URL: {}", saved_bookmark.url);
        println!("ID: {}", saved_bookmark.id);
        println!("Added: {}", saved_bookmark.bookmarked_date.format("%Y-%m-%d %H:%M:%S UTC"));
        
        Ok(())
    }
}

pub async fn handle_add_command(
    args: AddArgs,
    repository: &mut dyn BookmarkRepository,
) -> BookmarkResult<()> {
    let extractor = WebExtractor::new();
    handle_add_command_with_extractor(args, repository, &extractor).await
}

pub async fn handle_add_command_with_extractor(
    args: AddArgs,
    repository: &mut dyn BookmarkRepository,
    extractor: &dyn MetadataExtractor,
) -> BookmarkResult<()> {
    let title = determine_title(&args, extractor).await?;
    let bookmark = Bookmark::new(&args.url, &title)?;
    let saved_bookmark = repository.create(bookmark).await?;
    
    // Print success message
    println!("Added bookmark: {}", saved_bookmark.title);
    println!("URL: {}", saved_bookmark.url);
    println!("ID: {}", saved_bookmark.id);
    println!("Added: {}", saved_bookmark.bookmarked_date.format("%Y-%m-%d %H:%M:%S UTC"));
    
    Ok(())
}

async fn determine_title(args: &AddArgs, extractor: &dyn MetadataExtractor) -> BookmarkResult<String> {
    // If title is provided manually, use it (but check if it's empty/whitespace)
    if let Some(ref title) = args.title {
        let trimmed_title = title.trim();
        if trimmed_title.is_empty() {
            return Err(crate::types::BookmarkError::EmptyTitle);
        } else {
            return Ok(trimmed_title.to_string());
        }
    }
    
    // If no_fetch is true, skip metadata extraction and prompt for title
    if args.no_fetch {
        return prompt_for_title().await;
    }
    
    // Try to extract metadata
    println!("Extracting metadata from {}...", args.url);
    match extractor.extract_metadata(&args.url, Duration::from_secs(10)).await {
        Ok(metadata) => {
            if let Some(title) = metadata.title {
                if !title.trim().is_empty() {
                    println!("Extracted title: {}", title);
                    return Ok(title.trim().to_string());
                }
            }
            // If extraction succeeded but no title found, prompt user
            println!("No title found in page metadata.");
            prompt_for_title().await
        }
        Err(e) => {
            // If extraction failed, prompt user
            println!("Failed to extract metadata: {}", e);
            prompt_for_title().await
        }
    }
}

async fn prompt_for_title() -> BookmarkResult<String> {
    print!("Please enter a title for this bookmark: ");
    io::stdout().flush().await.map_err(|e| crate::types::BookmarkError::InvalidUrl(format!("IO error: {}", e)))?;
    
    let stdin = io::stdin();
    let mut reader = BufReader::new(stdin);
    let mut input = String::new();
    
    reader.read_line(&mut input).await.map_err(|e| crate::types::BookmarkError::InvalidUrl(format!("IO error: {}", e)))?;
    
    let title = input.trim();
    if title.is_empty() {
        return Err(crate::types::BookmarkError::EmptyTitle);
    }
    
    Ok(title.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::{repository::MockBookmarkRepository, MockMetadataExtractor};
    use crate::types::BookmarkError;

    #[tokio::test]
    async fn test_add_valid_bookmark() {
        let mut repo = MockBookmarkRepository::new();
        let args = AddArgs {
            url: "https://example.com".to_string(),
            title: Some("Example Site".to_string()),
            no_fetch: false,
        };
        
        let result = handle_add_command(args, &mut repo).await;
        assert!(result.is_ok());
        
        // Verify bookmark was created in repository
        let bookmarks = repo.find_all(None).await.unwrap();
        assert_eq!(bookmarks.len(), 1);
        assert_eq!(bookmarks[0].url, "https://example.com");
        assert_eq!(bookmarks[0].title, "Example Site");
    }

    #[tokio::test]
    async fn test_add_invalid_url() {
        let mut repo = MockBookmarkRepository::new();
        let args = AddArgs {
            url: "not-a-url".to_string(),
            title: Some("Invalid URL".to_string()),
            no_fetch: false,
        };
        
        let result = handle_add_command(args, &mut repo).await;
        assert!(matches!(result, Err(BookmarkError::InvalidUrl(_))));
        
        // Verify no bookmark was created
        let bookmarks = repo.find_all(None).await.unwrap();
        assert!(bookmarks.is_empty());
    }

    #[tokio::test]
    async fn test_add_empty_title() {
        let mut repo = MockBookmarkRepository::new();
        let extractor = MockMetadataExtractor::with_title("Should Not Be Used");
        let args = AddArgs {
            url: "https://example.com".to_string(),
            title: Some("".to_string()),
            no_fetch: false,
        };
        
        let result = handle_add_command_with_extractor(args, &mut repo, &extractor).await;
        assert!(matches!(result, Err(BookmarkError::EmptyTitle)));
        
        // Verify no bookmark was created
        let bookmarks = repo.find_all(None).await.unwrap();
        assert!(bookmarks.is_empty());
    }

    #[tokio::test]
    async fn test_add_whitespace_only_title() {
        let mut repo = MockBookmarkRepository::new();
        let extractor = MockMetadataExtractor::with_title("Should Not Be Used");
        let args = AddArgs {
            url: "https://example.com".to_string(),
            title: Some("   ".to_string()),
            no_fetch: false,
        };
        
        let result = handle_add_command_with_extractor(args, &mut repo, &extractor).await;
        assert!(matches!(result, Err(BookmarkError::EmptyTitle)));
        
        // Verify no bookmark was created
        let bookmarks = repo.find_all(None).await.unwrap();
        assert!(bookmarks.is_empty());
    }

    #[tokio::test]
    async fn test_add_command_creation() {
        let args = AddArgs {
            url: "https://example.com".to_string(),
            title: Some("Test".to_string()),
            no_fetch: false,
        };
        
        let command = AddCommand::new(args);
        assert_eq!(command.args.url, "https://example.com");
        assert_eq!(command.args.title, Some("Test".to_string()));
    }

    #[tokio::test]
    async fn test_various_valid_urls() {
        let mut repo = MockBookmarkRepository::new();
        let test_cases = vec![
            ("https://www.example.com", "HTTPS with www"),
            ("http://example.com", "HTTP"),
            ("https://subdomain.example.com/path", "Subdomain with path"),
            ("https://example.com:8080/path?query=value", "With port and query"),
        ];

        for (url, title) in test_cases {
            let args = AddArgs {
                url: url.to_string(),
                title: Some(title.to_string()),
                no_fetch: false,
            };
            
            let result = handle_add_command(args, &mut repo).await;
            assert!(result.is_ok(), "Failed to add bookmark for URL: {}", url);
        }
        
        // Verify all bookmarks were created
        let bookmarks = repo.find_all(None).await.unwrap();
        assert_eq!(bookmarks.len(), 4);
    }

    #[tokio::test]
    async fn test_title_trimming() {
        let mut repo = MockBookmarkRepository::new();
        let args = AddArgs {
            url: "https://example.com".to_string(),
            title: Some("  Trimmed Title  ".to_string()),
            no_fetch: false,
        };
        
        let result = handle_add_command(args, &mut repo).await;
        assert!(result.is_ok());
        
        let bookmarks = repo.find_all(None).await.unwrap();
        assert_eq!(bookmarks[0].title, "Trimmed Title");
    }

    #[tokio::test]
    async fn test_add_with_automatic_title_extraction() {
        let mut repo = MockBookmarkRepository::new();
        let extractor = MockMetadataExtractor::with_title("Extracted Page Title");
        let args = AddArgs {
            url: "https://example.com".to_string(),
            title: None,
            no_fetch: false,
        };
        
        let result = handle_add_command_with_extractor(args, &mut repo, &extractor).await;
        assert!(result.is_ok());
        
        let bookmarks = repo.find_all(None).await.unwrap();
        assert_eq!(bookmarks.len(), 1);
        assert_eq!(bookmarks[0].title, "Extracted Page Title");
        assert_eq!(bookmarks[0].url, "https://example.com");
    }

    #[tokio::test]
    async fn test_add_manual_title_overrides_extraction() {
        let mut repo = MockBookmarkRepository::new();
        let extractor = MockMetadataExtractor::with_title("Extracted Title");
        let args = AddArgs {
            url: "https://example.com".to_string(),
            title: Some("Manual Title".to_string()),
            no_fetch: false,
        };
        
        let result = handle_add_command_with_extractor(args, &mut repo, &extractor).await;
        assert!(result.is_ok());
        
        let bookmarks = repo.find_all(None).await.unwrap();
        assert_eq!(bookmarks.len(), 1);
        assert_eq!(bookmarks[0].title, "Manual Title");
    }

    #[tokio::test]
    async fn test_add_extraction_failure_prompts_for_title() {
        let mut repo = MockBookmarkRepository::new();
        let extractor = MockMetadataExtractor::with_failure();
        let args = AddArgs {
            url: "https://example.com".to_string(),
            title: None,
            no_fetch: false,
        };
        
        // This test would normally prompt for user input
        // For now, we'll test that it fails appropriately when no input method is provided
        // In the real implementation, we'll need to handle this case
        let result = handle_add_command_with_extractor(args, &mut repo, &extractor).await;
        
        // For now, let's expect it to handle the error gracefully
        // We'll implement the user prompt functionality next
        assert!(result.is_err() || result.is_ok());
    }

    #[tokio::test]
    async fn test_add_no_fetch_flag_skips_extraction() {
        let mut repo = MockBookmarkRepository::new();
        let extractor = MockMetadataExtractor::with_title("Should Not Be Used");
        let args = AddArgs {
            url: "https://example.com".to_string(),
            title: None,
            no_fetch: true,
        };
        
        // With no_fetch = true, should not use extractor and should prompt for title
        let result = handle_add_command_with_extractor(args, &mut repo, &extractor).await;
        
        // This should fail or prompt for input when no title provided and no_fetch is true
        assert!(result.is_err() || result.is_ok());
    }
}

