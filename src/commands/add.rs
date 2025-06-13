use crate::commands::{AddArgs, CommandHandler};
use crate::traits::BookmarkRepository;
use crate::types::{Bookmark, BookmarkResult};

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
        // Create bookmark from args
        let bookmark = Bookmark::new(&self.args.url, &self.args.title)?;
        
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
    let command = AddCommand::new(args);
    command.execute(repository).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::repository::MockBookmarkRepository;
    use crate::types::BookmarkError;

    #[tokio::test]
    async fn test_add_valid_bookmark() {
        let mut repo = MockBookmarkRepository::new();
        let args = AddArgs {
            url: "https://example.com".to_string(),
            title: "Example Site".to_string(),
        };
        
        let result = handle_add_command(args, &mut repo).await;
        assert!(result.is_ok());
        
        // Verify bookmark was created in repository
        let bookmarks = repo.find_all().await.unwrap();
        assert_eq!(bookmarks.len(), 1);
        assert_eq!(bookmarks[0].url, "https://example.com");
        assert_eq!(bookmarks[0].title, "Example Site");
    }

    #[tokio::test]
    async fn test_add_invalid_url() {
        let mut repo = MockBookmarkRepository::new();
        let args = AddArgs {
            url: "not-a-url".to_string(),
            title: "Invalid URL".to_string(),
        };
        
        let result = handle_add_command(args, &mut repo).await;
        assert!(matches!(result, Err(BookmarkError::InvalidUrl(_))));
        
        // Verify no bookmark was created
        let bookmarks = repo.find_all().await.unwrap();
        assert!(bookmarks.is_empty());
    }

    #[tokio::test]
    async fn test_add_empty_title() {
        let mut repo = MockBookmarkRepository::new();
        let args = AddArgs {
            url: "https://example.com".to_string(),
            title: "".to_string(),
        };
        
        let result = handle_add_command(args, &mut repo).await;
        assert!(matches!(result, Err(BookmarkError::EmptyTitle)));
        
        // Verify no bookmark was created
        let bookmarks = repo.find_all().await.unwrap();
        assert!(bookmarks.is_empty());
    }

    #[tokio::test]
    async fn test_add_whitespace_only_title() {
        let mut repo = MockBookmarkRepository::new();
        let args = AddArgs {
            url: "https://example.com".to_string(),
            title: "   ".to_string(),
        };
        
        let result = handle_add_command(args, &mut repo).await;
        assert!(matches!(result, Err(BookmarkError::EmptyTitle)));
        
        // Verify no bookmark was created
        let bookmarks = repo.find_all().await.unwrap();
        assert!(bookmarks.is_empty());
    }

    #[tokio::test]
    async fn test_add_command_creation() {
        let args = AddArgs {
            url: "https://example.com".to_string(),
            title: "Test".to_string(),
        };
        
        let command = AddCommand::new(args);
        assert_eq!(command.args.url, "https://example.com");
        assert_eq!(command.args.title, "Test");
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
                title: title.to_string(),
            };
            
            let result = handle_add_command(args, &mut repo).await;
            assert!(result.is_ok(), "Failed to add bookmark for URL: {}", url);
        }
        
        // Verify all bookmarks were created
        let bookmarks = repo.find_all().await.unwrap();
        assert_eq!(bookmarks.len(), 4);
    }

    #[tokio::test]
    async fn test_title_trimming() {
        let mut repo = MockBookmarkRepository::new();
        let args = AddArgs {
            url: "https://example.com".to_string(),
            title: "  Trimmed Title  ".to_string(),
        };
        
        let result = handle_add_command(args, &mut repo).await;
        assert!(result.is_ok());
        
        let bookmarks = repo.find_all().await.unwrap();
        assert_eq!(bookmarks[0].title, "Trimmed Title");
    }
}