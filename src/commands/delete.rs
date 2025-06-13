use crate::commands::{CommandHandler, DeleteArgs, OutputFormat, output};
use crate::traits::BookmarkRepository;
use crate::types::{Bookmark, BookmarkResult, BookmarkError};
use serde::{Serialize, Deserialize};

/// JSON response data for delete command
#[derive(Serialize, Deserialize, Debug)]
pub struct DeleteResponse {
    pub deleted_bookmark: Bookmark,
    pub operation_status: String,
    pub affected_count: u32,
}

pub struct DeleteCommand {
    args: DeleteArgs,
}

impl DeleteCommand {
    pub fn new(args: DeleteArgs) -> Self {
        Self { args }
    }
    
    async fn find_bookmark_by_id(&self, repository: &mut dyn BookmarkRepository) -> BookmarkResult<Bookmark> {
        let all_bookmarks = repository.find_all(None).await?;
        
        // Try exact match first
        for bookmark in &all_bookmarks {
            if bookmark.id == self.args.id {
                return Ok(bookmark.clone());
            }
        }
        
        // If no exact match and input is ≤8 chars, try partial match
        if self.args.id.len() <= 8 {
            let matches: Vec<&Bookmark> = all_bookmarks
                .iter()
                .filter(|bookmark| bookmark.id.starts_with(&self.args.id))
                .collect();
                
            match matches.len() {
                0 => Err(BookmarkError::NotFound(self.args.id.clone())),
                1 => Ok(matches[0].clone()),
                _ => {
                    let matching_ids: Vec<String> = matches
                        .iter()
                        .map(|b| b.id[..8.min(b.id.len())].to_string())
                        .collect();
                    Err(BookmarkError::InvalidId(format!(
                        "Ambiguous ID '{}' matches multiple bookmarks: {}. Use a longer ID prefix.",
                        self.args.id,
                        matching_ids.join(", ")
                    )))
                }
            }
        } else {
            Err(BookmarkError::NotFound(self.args.id.clone()))
        }
    }
    
    fn format_deletion_confirmation(&self, bookmark: &Bookmark) -> String {
        format!(
            "Deleted bookmark: {}\n  URL: {}\n  ID: {}",
            bookmark.title,
            bookmark.url,
            bookmark.id
        )
    }
}

#[async_trait::async_trait]
impl CommandHandler for DeleteCommand {
    async fn execute(&self, repository: &mut dyn BookmarkRepository, format: OutputFormat) -> BookmarkResult<()> {
        let bookmark = self.find_bookmark_by_id(repository).await?;
        repository.delete(&bookmark.id).await?;
        
        match format {
            OutputFormat::Json => {
                let response = DeleteResponse {
                    deleted_bookmark: bookmark,
                    operation_status: "success".to_string(),
                    affected_count: 1,
                };
                output::print_response(format, response)?;
            }
            OutputFormat::Human => {
                let confirmation = self.format_deletion_confirmation(&bookmark);
                print!("{}", confirmation);
            }
        }
        
        Ok(())
    }
}

pub async fn handle_delete_command(
    args: DeleteArgs,
    repository: &mut dyn BookmarkRepository,
    format: OutputFormat,
) -> BookmarkResult<()> {
    let command = DeleteCommand::new(args);
    command.execute(repository, format).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::repository::MockBookmarkRepository;
    use crate::types::Bookmark;

    #[tokio::test]
    async fn test_delete_with_full_id() {
        let mut repo = MockBookmarkRepository::new();
        let bookmark = Bookmark::new("https://example.com", "Example Site").unwrap();
        let bookmark_id = bookmark.id.clone();
        repo.create(bookmark.clone()).await.unwrap();
        
        let args = DeleteArgs { id: bookmark_id.clone() };
        let result = handle_delete_command(args, &mut repo, OutputFormat::Human).await;
        
        assert!(result.is_ok());
        
        // Verify bookmark was deleted
        let remaining = repo.find_all(None).await.unwrap();
        assert!(remaining.is_empty());
    }

    #[tokio::test]
    async fn test_delete_with_unique_partial_id() {
        let mut repo = MockBookmarkRepository::new();
        let mut bookmark = Bookmark::new("https://example.com", "Example Site").unwrap();
        bookmark.id = "abcdef1234567890".to_string();
        repo.create(bookmark.clone()).await.unwrap();
        
        let args = DeleteArgs { id: "abcdef12".to_string() };
        let result = handle_delete_command(args, &mut repo, OutputFormat::Human).await;
        
        assert!(result.is_ok());
        
        // Verify bookmark was deleted
        let remaining = repo.find_all(None).await.unwrap();
        assert!(remaining.is_empty());
    }

    #[tokio::test]
    async fn test_delete_with_ambiguous_partial_id() {
        let mut repo = MockBookmarkRepository::new();
        
        let mut bookmark1 = Bookmark::new("https://example.com", "Example Site").unwrap();
        bookmark1.id = "abcdef1111111111".to_string();
        let mut bookmark2 = Bookmark::new("https://test.com", "Test Site").unwrap();
        bookmark2.id = "abcdef2222222222".to_string();
        
        repo.create(bookmark1).await.unwrap();
        repo.create(bookmark2).await.unwrap();
        
        let args = DeleteArgs { id: "abcdef".to_string() };
        let result = handle_delete_command(args, &mut repo, OutputFormat::Human).await;
        
        assert!(result.is_err());
        if let Err(BookmarkError::InvalidId(msg)) = result {
            assert!(msg.contains("Ambiguous ID 'abcdef'"));
            assert!(msg.contains("abcdef11, abcdef22"));
            assert!(msg.contains("Use a longer ID prefix"));
        } else {
            panic!("Expected InvalidId error");
        }
        
        // Verify no bookmarks were deleted
        let remaining = repo.find_all(None).await.unwrap();
        assert_eq!(remaining.len(), 2);
    }

    #[tokio::test]
    async fn test_delete_with_nonexistent_id() {
        let mut repo = MockBookmarkRepository::new();
        
        let args = DeleteArgs { id: "nonexistent".to_string() };
        let result = handle_delete_command(args, &mut repo, OutputFormat::Human).await;
        
        assert!(result.is_err());
        if let Err(BookmarkError::NotFound(id)) = result {
            assert_eq!(id, "nonexistent");
        } else {
            panic!("Expected NotFound error");
        }
    }

    #[tokio::test]
    async fn test_delete_with_long_nonexistent_id() {
        let mut repo = MockBookmarkRepository::new();
        let bookmark = Bookmark::new("https://example.com", "Example Site").unwrap();
        repo.create(bookmark).await.unwrap();
        
        // ID longer than 8 chars that doesn't match
        let args = DeleteArgs { id: "verylongidthatdoesnotexist".to_string() };
        let result = handle_delete_command(args, &mut repo, OutputFormat::Human).await;
        
        assert!(result.is_err());
        if let Err(BookmarkError::NotFound(id)) = result {
            assert_eq!(id, "verylongidthatdoesnotexist");
        } else {
            panic!("Expected NotFound error");
        }
    }

    #[tokio::test]
    async fn test_deletion_confirmation_format() {
        let mut bookmark = Bookmark::new("https://example.com", "Example Site").unwrap();
        bookmark.id = "test123".to_string();
        
        let args = DeleteArgs { id: "test123".to_string() };
        let command = DeleteCommand::new(args);
        let confirmation = command.format_deletion_confirmation(&bookmark);
        
        assert!(confirmation.contains("Deleted bookmark: Example Site"));
        assert!(confirmation.contains("URL: https://example.com"));
        assert!(confirmation.contains("ID: test123"));
        
        // Check structure
        assert!(confirmation.starts_with("Deleted bookmark: Example Site"));
        assert!(confirmation.contains("\n  URL: https://example.com"));
        assert!(confirmation.contains("\n  ID: test123"));
    }

    #[tokio::test]
    async fn test_exact_match_priority_over_partial() {
        let mut repo = MockBookmarkRepository::new();
        
        // Create bookmark with ID "abc"
        let mut bookmark1 = Bookmark::new("https://example.com", "Exact Match").unwrap();
        bookmark1.id = "abc".to_string();
        
        // Create bookmark with ID starting with "abc"
        let mut bookmark2 = Bookmark::new("https://test.com", "Partial Match").unwrap();
        bookmark2.id = "abcdef1234567890".to_string();
        
        repo.create(bookmark1.clone()).await.unwrap();
        repo.create(bookmark2.clone()).await.unwrap();
        
        // Search for "abc" should find exact match
        let args = DeleteArgs { id: "abc".to_string() };
        let result = handle_delete_command(args, &mut repo, OutputFormat::Human).await;
        
        assert!(result.is_ok());
        
        // Verify only the exact match was deleted
        let remaining = repo.find_all(None).await.unwrap();
        assert_eq!(remaining.len(), 1);
        assert_eq!(remaining[0].title, "Partial Match");
    }

    #[tokio::test]
    async fn test_partial_match_with_single_result() {
        let mut repo = MockBookmarkRepository::new();
        
        let mut bookmark = Bookmark::new("https://example.com", "Example Site").unwrap();
        bookmark.id = "unique123456789".to_string();
        repo.create(bookmark.clone()).await.unwrap();
        
        // Use first 6 chars as partial ID
        let args = DeleteArgs { id: "unique".to_string() };
        let result = handle_delete_command(args, &mut repo, OutputFormat::Human).await;
        
        assert!(result.is_ok());
        
        // Verify bookmark was deleted
        let remaining = repo.find_all(None).await.unwrap();
        assert!(remaining.is_empty());
    }

    #[tokio::test]
    async fn test_partial_id_length_boundary() {
        let mut repo = MockBookmarkRepository::new();
        
        let mut bookmark = Bookmark::new("https://example.com", "Example Site").unwrap();
        bookmark.id = "12345678901234567890".to_string();
        repo.create(bookmark.clone()).await.unwrap();
        
        // Test with exactly 8 characters (should try partial match)
        let args = DeleteArgs { id: "12345678".to_string() };
        let result = handle_delete_command(args, &mut repo, OutputFormat::Human).await;
        assert!(result.is_ok());
        
        // Re-add bookmark
        repo.create(bookmark.clone()).await.unwrap();
        
        // Test with 9 characters (should only try exact match)
        let args = DeleteArgs { id: "123456789".to_string() };
        let result = handle_delete_command(args, &mut repo, OutputFormat::Human).await;
        assert!(result.is_err());
        if let Err(BookmarkError::NotFound(_)) = result {
            // Expected
        } else {
            panic!("Expected NotFound error");
        }
    }

    #[tokio::test]
    async fn test_delete_command_creation() {
        let args = DeleteArgs { id: "test".to_string() };
        let command = DeleteCommand::new(args);
        assert_eq!(command.args.id, "test");
    }

    #[tokio::test]
    async fn test_delete_command_json_output() {
        let mut repo = MockBookmarkRepository::new();
        let bookmark = Bookmark::new("https://example.com", "Example Site").unwrap();
        let bookmark_id = bookmark.id.clone();
        repo.create(bookmark.clone()).await.unwrap();
        
        let args = DeleteArgs { id: bookmark_id.clone() };
        let result = handle_delete_command(args, &mut repo, OutputFormat::Json).await;
        
        assert!(result.is_ok());
        
        // Verify bookmark was deleted
        let remaining = repo.find_all(None).await.unwrap();
        assert!(remaining.is_empty());
    }

    #[tokio::test]
    async fn test_delete_command_handler_json_format() {
        let mut repo = MockBookmarkRepository::new();
        let bookmark = Bookmark::new("https://example.com", "Test").unwrap();
        let bookmark_id = bookmark.id.clone();
        repo.create(bookmark.clone()).await.unwrap();
        
        let args = DeleteArgs { id: bookmark_id };
        let command = DeleteCommand::new(args);
        let result = command.execute(&mut repo, OutputFormat::Json).await;
        assert!(result.is_ok());
        
        // Verify bookmark was deleted
        let remaining = repo.find_all(None).await.unwrap();
        assert!(remaining.is_empty());
    }

    #[tokio::test]
    async fn test_delete_response_serialization() {
        let bookmark = Bookmark::new("https://example.com", "Test").unwrap();
        let response = DeleteResponse {
            deleted_bookmark: bookmark.clone(),
            operation_status: "success".to_string(),
            affected_count: 1,
        };
        
        // Test that the response can be serialized to JSON
        let json = serde_json::to_string(&response);
        assert!(json.is_ok());
        
        let json_str = json.unwrap();
        assert!(json_str.contains("\"operation_status\":\"success\""));
        assert!(json_str.contains("\"affected_count\":1"));
        assert!(json_str.contains(&bookmark.id));
        assert!(json_str.contains("Test"));
    }
}