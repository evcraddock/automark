use crate::types::{Bookmark, BookmarkResult, BookmarkError};
use async_trait::async_trait;

#[async_trait]
pub trait BookmarkRepository: Send + Sync {
    async fn create(&mut self, bookmark: Bookmark) -> BookmarkResult<Bookmark>;
    async fn find_all(&self) -> BookmarkResult<Vec<Bookmark>>;
    async fn delete(&mut self, id: &str) -> BookmarkResult<()>;
}

#[cfg(test)]
pub struct MockBookmarkRepository {
    bookmarks: std::collections::HashMap<String, Bookmark>,
}

#[cfg(test)]
impl MockBookmarkRepository {
    pub fn new() -> Self {
        Self {
            bookmarks: std::collections::HashMap::new(),
        }
    }
}

#[cfg(test)]
#[async_trait]
impl BookmarkRepository for MockBookmarkRepository {
    async fn create(&mut self, bookmark: Bookmark) -> BookmarkResult<Bookmark> {
        let id = bookmark.id.clone();
        self.bookmarks.insert(id, bookmark.clone());
        Ok(bookmark)
    }

    async fn find_all(&self) -> BookmarkResult<Vec<Bookmark>> {
        Ok(self.bookmarks.values().cloned().collect())
    }

    async fn delete(&mut self, id: &str) -> BookmarkResult<()> {
        match self.bookmarks.remove(id) {
            Some(_) => Ok(()),
            None => Err(BookmarkError::NotFound(id.to_string())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Bookmark;

    #[tokio::test]
    async fn test_create_bookmark() {
        let mut repo = MockBookmarkRepository::new();
        let bookmark = Bookmark::new("https://example.com", "Example").unwrap();
        let original_id = bookmark.id.clone();
        
        let result = repo.create(bookmark).await.unwrap();
        
        assert_eq!(result.id, original_id);
        assert_eq!(result.url, "https://example.com");
        assert_eq!(result.title, "Example");
    }

    #[tokio::test]
    async fn test_find_all_empty() {
        let repo = MockBookmarkRepository::new();
        
        let bookmarks = repo.find_all().await.unwrap();
        
        assert!(bookmarks.is_empty());
    }

    #[tokio::test]
    async fn test_find_all_with_bookmarks() {
        let mut repo = MockBookmarkRepository::new();
        let bookmark1 = Bookmark::new("https://example.com", "Example 1").unwrap();
        let bookmark2 = Bookmark::new("https://test.com", "Test").unwrap();
        
        repo.create(bookmark1.clone()).await.unwrap();
        repo.create(bookmark2.clone()).await.unwrap();
        
        let bookmarks = repo.find_all().await.unwrap();
        
        assert_eq!(bookmarks.len(), 2);
        assert!(bookmarks.contains(&bookmark1));
        assert!(bookmarks.contains(&bookmark2));
    }

    #[tokio::test]
    async fn test_delete_existing_bookmark() {
        let mut repo = MockBookmarkRepository::new();
        let bookmark = Bookmark::new("https://example.com", "Example").unwrap();
        let bookmark_id = bookmark.id.clone();
        
        repo.create(bookmark).await.unwrap();
        
        // Verify it exists
        let bookmarks = repo.find_all().await.unwrap();
        assert_eq!(bookmarks.len(), 1);
        
        // Delete it
        let result = repo.delete(&bookmark_id).await;
        assert!(result.is_ok());
        
        // Verify it's gone
        let bookmarks = repo.find_all().await.unwrap();
        assert!(bookmarks.is_empty());
    }

    #[tokio::test]
    async fn test_delete_nonexistent_bookmark() {
        let mut repo = MockBookmarkRepository::new();
        
        let result = repo.delete("nonexistent-id").await;
        
        assert!(matches!(result, Err(BookmarkError::NotFound(_))));
        if let Err(BookmarkError::NotFound(id)) = result {
            assert_eq!(id, "nonexistent-id");
        }
    }

    #[tokio::test]
    async fn test_repository_state_persistence() {
        let mut repo = MockBookmarkRepository::new();
        
        // Add bookmark
        let bookmark = Bookmark::new("https://example.com", "Example").unwrap();
        let bookmark_id = bookmark.id.clone();
        repo.create(bookmark).await.unwrap();
        
        // Verify it persists across operations
        let bookmarks = repo.find_all().await.unwrap();
        assert_eq!(bookmarks.len(), 1);
        
        // Add another
        let bookmark2 = Bookmark::new("https://test.com", "Test").unwrap();
        repo.create(bookmark2).await.unwrap();
        
        // Verify both exist
        let bookmarks = repo.find_all().await.unwrap();
        assert_eq!(bookmarks.len(), 2);
        
        // Delete first one
        repo.delete(&bookmark_id).await.unwrap();
        
        // Verify only one remains
        let bookmarks = repo.find_all().await.unwrap();
        assert_eq!(bookmarks.len(), 1);
        assert_eq!(bookmarks[0].url, "https://test.com");
    }
}