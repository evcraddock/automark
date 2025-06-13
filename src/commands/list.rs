use crate::commands::CommandHandler;
use crate::traits::BookmarkRepository;
use crate::types::{Bookmark, BookmarkResult};

pub struct ListCommand;

impl ListCommand {
    pub fn new() -> Self {
        Self
    }
    
    fn format_bookmark(&self, bookmark: &Bookmark) -> String {
        let date = bookmark.bookmarked_date.format("%Y-%m-%d %H:%M:%S UTC");
        format!(
            "{}\n  URL: {}\n  Added: {}",
            bookmark.title,
            bookmark.url,
            date
        )
    }
    
    fn format_bookmark_list(&self, bookmarks: &[Bookmark]) -> String {
        if bookmarks.is_empty() {
            "No bookmarks found. Use 'automark add <URL> <TITLE>' to add your first bookmark.".to_string()
        } else {
            let mut output = format!("Found {} bookmark(s):\n\n", bookmarks.len());
            for (index, bookmark) in bookmarks.iter().enumerate() {
                let partial_id = if bookmark.id.len() >= 8 {
                    &bookmark.id[..8]
                } else {
                    &bookmark.id
                };
                output.push_str(&format!("{}. [{}] {}", 
                    index + 1, 
                    partial_id,
                    self.format_bookmark(bookmark)
                ));
                output.push('\n');
                if index < bookmarks.len() - 1 {
                    output.push('\n');
                }
            }
            output
        }
    }
}

#[async_trait::async_trait]
impl CommandHandler for ListCommand {
    async fn execute(&self, repository: &mut dyn BookmarkRepository) -> BookmarkResult<()> {
        let bookmarks = repository.find_all(None).await?;
        let output = self.format_bookmark_list(&bookmarks);
        print!("{}", output);
        Ok(())
    }
}

pub async fn handle_list_command(
    repository: &mut dyn BookmarkRepository,
) -> BookmarkResult<()> {
    let command = ListCommand::new();
    command.execute(repository).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::repository::MockBookmarkRepository;
    use crate::types::Bookmark;

    #[tokio::test]
    async fn test_list_empty_repository() {
        let mut repo = MockBookmarkRepository::new();
        
        let result = handle_list_command(&mut repo).await;
        assert!(result.is_ok());
        
        // The actual output is printed, but we can test the formatting method directly
        let command = ListCommand::new();
        let output = command.format_bookmark_list(&[]);
        assert!(output.contains("No bookmarks found"));
        assert!(output.contains("automark add"));
    }

    #[tokio::test]
    async fn test_list_single_bookmark() {
        let mut repo = MockBookmarkRepository::new();
        let bookmark = Bookmark::new("https://example.com", "Example Site").unwrap();
        repo.create(bookmark.clone()).await.unwrap();
        
        let result = handle_list_command(&mut repo).await;
        assert!(result.is_ok());
        
        // Test formatting directly
        let command = ListCommand::new();
        let output = command.format_bookmark_list(&[bookmark]);
        assert!(output.contains("Found 1 bookmark(s):"));
        assert!(output.contains("1. ["));
        assert!(output.contains("] Example Site"));
        assert!(output.contains("https://example.com"));
        assert!(output.contains("Added:"));
    }

    #[tokio::test]
    async fn test_list_multiple_bookmarks() {
        let mut repo = MockBookmarkRepository::new();
        
        let bookmark1 = Bookmark::new("https://example.com", "Example Site").unwrap();
        let bookmark2 = Bookmark::new("https://test.com", "Test Site").unwrap();
        let bookmark3 = Bookmark::new("https://rust-lang.org", "Rust Programming").unwrap();
        
        repo.create(bookmark1.clone()).await.unwrap();
        repo.create(bookmark2.clone()).await.unwrap();
        repo.create(bookmark3.clone()).await.unwrap();
        
        let result = handle_list_command(&mut repo).await;
        assert!(result.is_ok());
        
        // Test formatting directly
        let command = ListCommand::new();
        let bookmarks = vec![bookmark1, bookmark2, bookmark3];
        let output = command.format_bookmark_list(&bookmarks);
        
        assert!(output.contains("Found 3 bookmark(s):"));
        assert!(output.contains("1. ["));
        assert!(output.contains("2. ["));
        assert!(output.contains("3. ["));
        assert!(output.contains("Example Site"));
        assert!(output.contains("Test Site"));
        assert!(output.contains("Rust Programming"));
    }

    #[tokio::test]
    async fn test_bookmark_formatting() {
        let bookmark = Bookmark::new("https://example.com", "Test Bookmark").unwrap();
        let command = ListCommand::new();
        let output = command.format_bookmark(&bookmark);
        
        assert!(output.contains("Test Bookmark"));
        assert!(output.contains("https://example.com"));
        assert!(output.contains("Added:"));
        assert!(output.contains("UTC"));
        
        // Check structure
        assert!(output.starts_with("Test Bookmark"));
        assert!(output.contains("\n  URL: https://example.com"));
        assert!(output.contains("\n  Added:"));
    }

    #[tokio::test]
    async fn test_partial_id_display() {
        // Create bookmark with known long ID
        let mut bookmark = Bookmark::new("https://example.com", "Test").unwrap();
        bookmark.id = "abcdef1234567890".to_string(); // 16 chars
        
        let command = ListCommand::new();
        let output = command.format_bookmark_list(&[bookmark]);
        
        // Should show first 8 characters
        assert!(output.contains("[abcdef12]"));
        assert!(!output.contains("34567890")); // Should not show the rest
    }

    #[tokio::test]
    async fn test_short_id_display() {
        // Create bookmark with short ID
        let mut bookmark = Bookmark::new("https://example.com", "Test").unwrap();
        bookmark.id = "abc".to_string(); // 3 chars
        
        let command = ListCommand::new();
        let output = command.format_bookmark_list(&[bookmark]);
        
        // Should show full ID when less than 8 characters
        assert!(output.contains("[abc]"));
    }

    #[tokio::test]
    async fn test_date_formatting_consistency() {
        let bookmark = Bookmark::new("https://example.com", "Test").unwrap();
        let command = ListCommand::new();
        let output = command.format_bookmark(&bookmark);
        
        // Check date format pattern (YYYY-MM-DD HH:MM:SS UTC)
        let date_pattern = regex::Regex::new(r"\d{4}-\d{2}-\d{2} \d{2}:\d{2}:\d{2} UTC").unwrap();
        assert!(date_pattern.is_match(&output));
    }

    #[tokio::test]
    async fn test_list_command_creation() {
        let command = ListCommand::new();
        // Just verify it can be created - it's a unit struct
        let _command = command;
    }

    #[tokio::test]
    async fn test_output_structure() {
        let bookmark1 = Bookmark::new("https://example.com", "First").unwrap();
        let bookmark2 = Bookmark::new("https://test.com", "Second").unwrap();
        
        let command = ListCommand::new();
        let output = command.format_bookmark_list(&[bookmark1, bookmark2]);
        
        // Test structure
        let lines: Vec<&str> = output.lines().collect();
        assert!(lines[0].starts_with("Found 2 bookmark(s):"));
        assert_eq!(lines[1], ""); // Empty line after header
        assert!(lines[2].starts_with("1. [")); // First bookmark
        assert!(lines[3].starts_with("  URL:")); // First bookmark URL
        assert!(lines[4].starts_with("  Added:")); // First bookmark date
        assert_eq!(lines[5], ""); // Empty line between bookmarks
        assert!(lines[6].starts_with("2. [")); // Second bookmark
    }
}