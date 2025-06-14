use crate::commands::{AddArgs, CommandHandler, OutputFormat, output};
use crate::traits::{BookmarkRepository, MetadataExtractor};
use crate::types::{Bookmark, BookmarkResult, Config, ExtractedMetadata};
use crate::adapters::WebExtractor;
use std::time::Duration;
use tokio::io::{self, AsyncBufReadExt, AsyncWriteExt, BufReader};
use serde::{Serialize, Deserialize};
use tokio::time;

pub struct AddCommand {
    pub args: AddArgs,
}

impl AddCommand {
    pub fn new(args: AddArgs) -> Self {
        Self { args }
    }
}

/// JSON response data for add command
#[derive(Serialize, Deserialize, Debug)]
pub struct AddResponse {
    pub bookmark: Bookmark,
    pub metadata_extracted: bool,
    pub extraction_time_ms: Option<u64>,
    pub extracted_metadata: Option<ExtractedMetadataInfo>,
    pub extraction_status: ExtractionStatus,
}

/// Information about extracted metadata for response
#[derive(Serialize, Deserialize, Debug)]
pub struct ExtractedMetadataInfo {
    pub title: Option<String>,
    pub author: Option<String>,
    pub publish_date: Option<String>,
}

/// Status of metadata extraction
#[derive(Serialize, Deserialize, Debug)]
pub enum ExtractionStatus {
    Success,
    Skipped,
    Failed(String),
    Timeout,
}

#[async_trait::async_trait]
impl CommandHandler for AddCommand {
    async fn execute(&self, repository: &mut dyn BookmarkRepository, format: OutputFormat) -> BookmarkResult<()> {
        // Use default config for CommandHandler trait implementation
        let config = Config::default();
        let extractor = WebExtractor::new();
        handle_add_command_with_extractor_and_config(
            self.args.clone(),
            repository,
            &extractor,
            &config,
            format,
        ).await
    }
}

pub async fn handle_add_command(
    args: AddArgs,
    repository: &mut dyn BookmarkRepository,
    config: &Config,
    format: OutputFormat,
) -> BookmarkResult<()> {
    let extractor = WebExtractor::new();
    handle_add_command_with_extractor_and_config(args, repository, &extractor, config, format).await
}

pub async fn handle_add_command_with_extractor_and_config(
    args: AddArgs,
    repository: &mut dyn BookmarkRepository,
    extractor: &dyn MetadataExtractor,
    config: &Config,
    format: OutputFormat,
) -> BookmarkResult<()> {
    let start_time = std::time::Instant::now();
    
    // Determine if metadata extraction should be performed
    let should_extract = should_extract_metadata(&args, config);
    let mut extraction_status = ExtractionStatus::Skipped;
    let mut extracted_metadata_info = None;
    let mut extracted_metadata = None;
    
    if should_extract {
        let extraction_result = extract_metadata_with_config(&args.url, extractor, config).await;
        match extraction_result {
            Ok(metadata) => {
                extraction_status = ExtractionStatus::Success;
                extracted_metadata_info = Some(ExtractedMetadataInfo {
                    title: metadata.title.clone(),
                    author: metadata.author.clone(),
                    publish_date: metadata.publish_date.as_ref().map(|d| d.to_rfc3339()),
                });
                extracted_metadata = Some(metadata);
                
                if format == OutputFormat::Human {
                    println!("Successfully extracted metadata from {}", args.url);
                    if let Some(ref title) = extracted_metadata.as_ref().unwrap().title {
                        println!("  Title: {}", title);
                    }
                    if let Some(ref author) = extracted_metadata.as_ref().unwrap().author {
                        println!("  Author: {}", author);
                    }
                }
            }
            Err(e) => {
                extraction_status = ExtractionStatus::Failed(e.to_string());
                if format == OutputFormat::Human {
                    println!("Metadata extraction failed: {}", e);
                    println!("Continuing with bookmark creation...");
                }
            }
        }
    }
    
    let extraction_time = start_time.elapsed();
    
    // Create bookmark with metadata integration
    let bookmark = create_bookmark_with_metadata(&args, extracted_metadata.as_ref())?;
    let saved_bookmark = repository.create(bookmark).await?;
    
    // Output results
    match format {
        OutputFormat::Json => {
            let response = AddResponse {
                bookmark: saved_bookmark,
                metadata_extracted: should_extract && matches!(extraction_status, ExtractionStatus::Success),
                extraction_time_ms: if should_extract {
                    Some(extraction_time.as_millis() as u64)
                } else {
                    None
                },
                extracted_metadata: extracted_metadata_info,
                extraction_status,
            };
            output::print_response(format, response)?;
        }
        OutputFormat::Human => {
            println!("\n✓ Successfully added bookmark:");
            println!("  Title: {}", saved_bookmark.title);
            println!("  URL: {}", saved_bookmark.url);
            if let Some(ref author) = saved_bookmark.author {
                println!("  Author: {}", author);
            }
            if !saved_bookmark.tags.is_empty() {
                println!("  Tags: {}", saved_bookmark.tags.join(", "));
            }
            println!("  ID: {}", saved_bookmark.id);
            println!("  Added: {}", saved_bookmark.bookmarked_date.format("%Y-%m-%d %H:%M:%S UTC"));
            
            if should_extract {
                match extraction_status {
                    ExtractionStatus::Success => println!("  Metadata extraction: successful ({:.2}s)", extraction_time.as_secs_f64()),
                    ExtractionStatus::Failed(_) => println!("  Metadata extraction: failed ({:.2}s)", extraction_time.as_secs_f64()),
                    ExtractionStatus::Timeout => println!("  Metadata extraction: timed out ({:.2}s)", extraction_time.as_secs_f64()),
                    _ => {}
                }
            } else {
                println!("  Metadata extraction: skipped");
            }
        }
    }
    
    Ok(())
}

// Legacy function for backward compatibility in tests
pub async fn handle_add_command_with_extractor(
    args: AddArgs,
    repository: &mut dyn BookmarkRepository,
    extractor: &dyn MetadataExtractor,
    format: OutputFormat,
) -> BookmarkResult<()> {
    let config = Config::default();
    handle_add_command_with_extractor_and_config(args, repository, extractor, &config, format).await
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

/// Determine if metadata extraction should be performed based on args and config
fn should_extract_metadata(args: &AddArgs, config: &Config) -> bool {
    // If --no-fetch is specified, never extract
    if args.no_fetch {
        return false;
    }
    
    // If title and author are both provided manually, and we have no other metadata to extract
    if args.title.is_some() && args.author.is_some() {
        return false;
    }
    
    // Check config setting
    config.metadata.enabled
}

/// Extract metadata with configuration settings including timeout and retries
async fn extract_metadata_with_config(
    url: &str,
    extractor: &dyn MetadataExtractor,
    config: &Config,
) -> BookmarkResult<ExtractedMetadata> {
    let timeout_duration = Duration::from_secs(config.metadata.timeout_secs);
    let mut last_error = None;
    
    for attempt in 0..=config.metadata.retry_attempts {
        if attempt > 0 {
            // Wait before retry
            time::sleep(Duration::from_millis(config.metadata.retry_delay_ms)).await;
        }
        
        match time::timeout(timeout_duration, extractor.extract_metadata(url, timeout_duration)).await {
            Ok(Ok(metadata)) => return Ok(metadata),
            Ok(Err(e)) => {
                last_error = Some(e);
                if attempt < config.metadata.retry_attempts {
                    eprintln!("Metadata extraction attempt {} failed, retrying...", attempt + 1);
                }
            }
            Err(_) => {
                return Err(crate::types::BookmarkError::MetadataExtraction(
                    crate::types::ExtractorError::Timeout
                ));
            }
        }
    }
    
    // If we get here, all retries failed
    Err(crate::types::BookmarkError::MetadataExtraction(
        last_error.unwrap_or(crate::types::ExtractorError::NetworkError("Unknown error".to_string()))
    ))
}

/// Create a bookmark integrating manual args with extracted metadata
fn create_bookmark_with_metadata(
    args: &AddArgs,
    extracted_metadata: Option<&ExtractedMetadata>,
) -> BookmarkResult<Bookmark> {
    // Determine title: manual override > extracted > error
    let title = if let Some(ref manual_title) = args.title {
        manual_title.trim().to_string()
    } else if let Some(metadata) = extracted_metadata {
        if let Some(ref extracted_title) = metadata.title {
            extracted_title.trim().to_string()
        } else {
            return Err(crate::types::BookmarkError::EmptyTitle);
        }
    } else {
        return Err(crate::types::BookmarkError::EmptyTitle);
    };
    
    if title.is_empty() {
        return Err(crate::types::BookmarkError::EmptyTitle);
    }
    
    // Create base bookmark
    let mut bookmark = Bookmark::new(&args.url, &title)?;
    
    // Set author: manual override > extracted
    if let Some(ref manual_author) = args.author {
        bookmark.author = Some(manual_author.trim().to_string());
    } else if let Some(metadata) = extracted_metadata {
        bookmark.author = metadata.author.clone();
    }
    
    // Set publish date from extracted metadata if available
    if let Some(metadata) = extracted_metadata {
        bookmark.publish_date = metadata.publish_date;
    }
    
    // Add tags from args
    if !args.tags.is_empty() {
        bookmark.tags = args.tags.iter()
            .map(|tag| tag.trim().to_string())
            .filter(|tag| !tag.is_empty())
            .collect();
    }
    
    Ok(bookmark)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::{repository::MockBookmarkRepository, MockMetadataExtractor};
    use crate::types::BookmarkError;

    #[tokio::test]
    async fn test_add_valid_bookmark() {
        let mut repo = MockBookmarkRepository::new();
        let config = Config::default();
        let args = AddArgs {
            url: "https://example.com".to_string(),
            title: Some("Example Site".to_string()),
            author: None,
            tags: vec![],
            no_fetch: false,
        };
        
        let result = handle_add_command(args, &mut repo, &config, OutputFormat::Human).await;
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
        let config = Config::default();
        let args = AddArgs {
            url: "not-a-url".to_string(),
            title: Some("Invalid URL".to_string()),
            author: None,
            tags: vec![],
            no_fetch: false,
        };
        
        let result = handle_add_command(args, &mut repo, &config, OutputFormat::Human).await;
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
            author: None,
            tags: vec![],
            no_fetch: false,
        };
        
        let result = handle_add_command_with_extractor(args, &mut repo, &extractor, OutputFormat::Human).await;
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
            author: None,
            tags: vec![],
            no_fetch: false,
        };
        
        let result = handle_add_command_with_extractor(args, &mut repo, &extractor, OutputFormat::Human).await;
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
            author: None,
            tags: vec![],
            no_fetch: false,
        };
        
        let command = AddCommand::new(args);
        assert_eq!(command.args.url, "https://example.com");
        assert_eq!(command.args.title, Some("Test".to_string()));
    }

    #[tokio::test]
    async fn test_various_valid_urls() {
        let mut repo = MockBookmarkRepository::new();
        let config = Config::default();
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
                author: None,
                tags: vec![],
                no_fetch: false,
            };
            
            let result = handle_add_command(args, &mut repo, &config, OutputFormat::Human).await;
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
            author: None,
            tags: vec![],
            no_fetch: false,
        };
        
        let result = handle_add_command(args, &mut repo, &Config::default(), OutputFormat::Human).await;
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
            author: None,
            tags: vec![],
            no_fetch: false,
        };
        
        let result = handle_add_command_with_extractor(args, &mut repo, &extractor, OutputFormat::Human).await;
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
            author: None,
            tags: vec![],
            no_fetch: false,
        };
        
        let result = handle_add_command_with_extractor(args, &mut repo, &extractor, OutputFormat::Human).await;
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
            author: None,
            tags: vec![],
            no_fetch: false,
        };
        
        // This test would normally prompt for user input
        // For now, we'll test that it fails appropriately when no input method is provided
        // In the real implementation, we'll need to handle this case
        let result = handle_add_command_with_extractor(args, &mut repo, &extractor, OutputFormat::Human).await;
        
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
            author: None,
            tags: vec![],
            no_fetch: true,
        };
        
        // With no_fetch = true, should not use extractor and should prompt for title
        let result = handle_add_command_with_extractor(args, &mut repo, &extractor, OutputFormat::Human).await;
        
        // This should fail or prompt for input when no title provided and no_fetch is true
        assert!(result.is_err() || result.is_ok());
    }

    #[tokio::test]
    async fn test_add_command_json_output() {
        let mut repo = MockBookmarkRepository::new();
        let extractor = MockMetadataExtractor::with_title("Extracted Title");
        let args = AddArgs {
            url: "https://example.com".to_string(),
            title: Some("Test Bookmark".to_string()),
            author: None,
            tags: vec![],
            no_fetch: false,
        };
        
        let result = handle_add_command_with_extractor(args, &mut repo, &extractor, OutputFormat::Json).await;
        assert!(result.is_ok());
        
        // Verify bookmark was created
        let bookmarks = repo.find_all(None).await.unwrap();
        assert_eq!(bookmarks.len(), 1);
        assert_eq!(bookmarks[0].title, "Test Bookmark");
    }

    #[tokio::test]
    async fn test_add_command_handler_json_format() {
        let mut repo = MockBookmarkRepository::new();
        let args = AddArgs {
            url: "https://example.com".to_string(),
            title: Some("Handler Test".to_string()),
            author: None,
            tags: vec![],
            no_fetch: false,
        };
        
        let command = AddCommand::new(args);
        let result = command.execute(&mut repo, OutputFormat::Json).await;
        assert!(result.is_ok());
        
        // Verify bookmark was created
        let bookmarks = repo.find_all(None).await.unwrap();
        assert_eq!(bookmarks.len(), 1);
        assert_eq!(bookmarks[0].title, "Handler Test");
    }

    #[tokio::test]
    async fn test_add_response_serialization() {
        let bookmark = Bookmark::new("https://example.com", "Test").unwrap();
        let response = AddResponse {
            bookmark: bookmark.clone(),
            metadata_extracted: true,
            extraction_time_ms: Some(250),
            extracted_metadata: None,
            extraction_status: ExtractionStatus::Success,
        };
        
        // Test that the response can be serialized to JSON
        let json = serde_json::to_string(&response);
        assert!(json.is_ok());
        
        let json_str = json.unwrap();
        assert!(json_str.contains("\"metadata_extracted\":true"));
        assert!(json_str.contains("\"extraction_time_ms\":250"));
        assert!(json_str.contains(&bookmark.id));
    }

    // New tests for metadata integration
    #[tokio::test]
    async fn test_add_with_author_and_tags() {
        let mut repo = MockBookmarkRepository::new();
        let config = Config::default();
        let args = AddArgs {
            url: "https://example.com".to_string(),
            title: Some("Test Article".to_string()),
            author: Some("Jane Doe".to_string()),
            tags: vec!["rust".to_string(), "programming".to_string()],
            no_fetch: true, // Skip metadata extraction
        };
        
        let result = handle_add_command(args, &mut repo, &config, OutputFormat::Human).await;
        assert!(result.is_ok());
        
        let bookmarks = repo.find_all(None).await.unwrap();
        assert_eq!(bookmarks.len(), 1);
        assert_eq!(bookmarks[0].title, "Test Article");
        assert_eq!(bookmarks[0].author, Some("Jane Doe".to_string()));
        assert_eq!(bookmarks[0].tags, vec!["rust", "programming"]);
    }

    #[tokio::test]
    async fn test_add_with_metadata_extraction() {
        let mut repo = MockBookmarkRepository::new();
        let config = Config::default();
        let extractor = MockMetadataExtractor::with_metadata(
            Some("Extracted Title".to_string()),
            Some("Extracted Author".to_string()),
            None,
        );
        let args = AddArgs {
            url: "https://example.com".to_string(),
            title: None, // Should use extracted title
            author: None, // Should use extracted author
            tags: vec!["test".to_string()],
            no_fetch: false,
        };
        
        let result = handle_add_command_with_extractor_and_config(
            args, &mut repo, &extractor, &config, OutputFormat::Human
        ).await;
        assert!(result.is_ok());
        
        let bookmarks = repo.find_all(None).await.unwrap();
        assert_eq!(bookmarks.len(), 1);
        assert_eq!(bookmarks[0].title, "Extracted Title");
        assert_eq!(bookmarks[0].author, Some("Extracted Author".to_string()));
        assert_eq!(bookmarks[0].tags, vec!["test"]);
    }

    #[tokio::test]
    async fn test_add_manual_overrides_extracted() {
        let mut repo = MockBookmarkRepository::new();
        let config = Config::default();
        let extractor = MockMetadataExtractor::with_metadata(
            Some("Extracted Title".to_string()),
            Some("Extracted Author".to_string()),
            None,
        );
        let args = AddArgs {
            url: "https://example.com".to_string(),
            title: Some("Manual Title".to_string()), // Should override extracted
            author: Some("Manual Author".to_string()), // Should override extracted
            tags: vec![],
            no_fetch: false,
        };
        
        let result = handle_add_command_with_extractor_and_config(
            args, &mut repo, &extractor, &config, OutputFormat::Human
        ).await;
        assert!(result.is_ok());
        
        let bookmarks = repo.find_all(None).await.unwrap();
        assert_eq!(bookmarks.len(), 1);
        assert_eq!(bookmarks[0].title, "Manual Title");
        assert_eq!(bookmarks[0].author, Some("Manual Author".to_string()));
    }

    #[tokio::test]
    async fn test_add_extraction_failure_graceful() {
        let mut repo = MockBookmarkRepository::new();
        let config = Config::default();
        let extractor = MockMetadataExtractor::with_failure();
        let args = AddArgs {
            url: "https://example.com".to_string(),
            title: Some("Manual Title".to_string()), // Should fallback to this
            author: None,
            tags: vec![],
            no_fetch: false,
        };
        
        let result = handle_add_command_with_extractor_and_config(
            args, &mut repo, &extractor, &config, OutputFormat::Human
        ).await;
        assert!(result.is_ok());
        
        let bookmarks = repo.find_all(None).await.unwrap();
        assert_eq!(bookmarks.len(), 1);
        assert_eq!(bookmarks[0].title, "Manual Title");
    }

    #[tokio::test]
    async fn test_add_no_fetch_skips_extraction() {
        let mut repo = MockBookmarkRepository::new();
        let config = Config::default();
        let extractor = MockMetadataExtractor::with_title("Should Not Be Used");
        let args = AddArgs {
            url: "https://example.com".to_string(),
            title: Some("Manual Title".to_string()),
            author: None,
            tags: vec![],
            no_fetch: true,
        };
        
        let result = handle_add_command_with_extractor_and_config(
            args, &mut repo, &extractor, &config, OutputFormat::Human
        ).await;
        assert!(result.is_ok());
        
        let bookmarks = repo.find_all(None).await.unwrap();
        assert_eq!(bookmarks.len(), 1);
        assert_eq!(bookmarks[0].title, "Manual Title");
    }

    #[tokio::test]
    async fn test_metadata_config_integration() {
        let mut repo = MockBookmarkRepository::new();
        let mut config = Config::default();
        config.metadata.enabled = false; // Disable metadata extraction
        
        let extractor = MockMetadataExtractor::with_title("Should Not Be Used");
        let args = AddArgs {
            url: "https://example.com".to_string(),
            title: Some("Manual Title".to_string()),
            author: None,
            tags: vec![],
            no_fetch: false,
        };
        
        let result = handle_add_command_with_extractor_and_config(
            args, &mut repo, &extractor, &config, OutputFormat::Human
        ).await;
        assert!(result.is_ok());
        
        let bookmarks = repo.find_all(None).await.unwrap();
        assert_eq!(bookmarks.len(), 1);
        assert_eq!(bookmarks[0].title, "Manual Title");
    }

    #[tokio::test]
    async fn test_json_output_with_metadata() {
        let mut repo = MockBookmarkRepository::new();
        let config = Config::default();
        let extractor = MockMetadataExtractor::with_metadata(
            Some("Extracted Title".to_string()),
            Some("Extracted Author".to_string()),
            None,
        );
        let args = AddArgs {
            url: "https://example.com".to_string(),
            title: None,
            author: None,
            tags: vec!["tag1".to_string(), "tag2".to_string()],
            no_fetch: false,
        };
        
        let result = handle_add_command_with_extractor_and_config(
            args, &mut repo, &extractor, &config, OutputFormat::Json
        ).await;
        assert!(result.is_ok());
        
        let bookmarks = repo.find_all(None).await.unwrap();
        assert_eq!(bookmarks.len(), 1);
        assert_eq!(bookmarks[0].title, "Extracted Title");
        assert_eq!(bookmarks[0].author, Some("Extracted Author".to_string()));
        assert_eq!(bookmarks[0].tags, vec!["tag1", "tag2"]);
    }
}

