#![allow(dead_code)]
use crate::types::{Bookmark, BookmarkResult, BookmarkFilters};
#[cfg(test)]
use crate::types::BookmarkError;
use async_trait::async_trait;

/// Repository trait for managing bookmarks with CRDT support
/// 
/// This trait provides comprehensive CRUD operations, search, and filtering capabilities
/// for bookmarks. All operations are designed to work with CRDT (Conflict-free Replicated
/// Data Types) semantics for distributed synchronization.
#[async_trait]
pub trait BookmarkRepository: Send + Sync {
    /// Create a new bookmark
    /// 
    /// # Arguments
    /// * `bookmark` - The bookmark to create
    /// 
    /// # Returns
    /// The created bookmark with any auto-generated fields populated
    /// 
    /// # CRDT Behavior
    /// Creates a new document entry in the CRDT with a unique ID
    async fn create(&mut self, bookmark: Bookmark) -> BookmarkResult<Bookmark>;
    
    /// Find all bookmarks, optionally filtered
    /// 
    /// # Arguments  
    /// * `filters` - Optional filters to apply (None returns all bookmarks)
    /// 
    /// # Returns
    /// Vector of bookmarks matching the filter criteria
    /// 
    /// # CRDT Behavior
    /// Reads current state without modifying the CRDT document
    async fn find_all(&self, filters: Option<BookmarkFilters>) -> BookmarkResult<Vec<Bookmark>>;
    
    /// Find a bookmark by its ID
    /// 
    /// # Arguments
    /// * `id` - The bookmark ID to search for
    /// 
    /// # Returns
    /// The bookmark if found, NotFound error if not found
    /// 
    /// # CRDT Behavior
    /// Reads current state without modifying the CRDT document
    async fn find_by_id(&self, id: &str) -> BookmarkResult<Bookmark>;
    
    /// Update an existing bookmark
    /// 
    /// # Arguments
    /// * `bookmark` - The bookmark with updated fields
    /// 
    /// # Returns
    /// The updated bookmark
    /// 
    /// # CRDT Behavior
    /// Merges changes at the field level, preserving concurrent modifications.
    /// Collections (tags, notes) use set union semantics.
    /// Scalar fields use last-writer-wins semantics based on timestamps.
    async fn update(&mut self, bookmark: Bookmark) -> BookmarkResult<Bookmark>;
    
    /// Delete a bookmark by ID
    /// 
    /// # Arguments
    /// * `id` - The ID of the bookmark to delete
    /// 
    /// # CRDT Behavior
    /// Uses tombstone markers to ensure deletion propagates across replicas
    async fn delete(&mut self, id: &str) -> BookmarkResult<()>;
    
    /// Search bookmarks by text content
    /// 
    /// Searches across title, URL, author, and note content.
    /// Search is case-insensitive.
    /// 
    /// # Arguments
    /// * `query` - The text to search for
    /// 
    /// # Returns
    /// Vector of bookmarks containing the search text
    async fn search_by_text(&self, query: &str) -> BookmarkResult<Vec<Bookmark>>;
    
    /// Find bookmarks containing all specified tags
    /// 
    /// Uses AND logic - bookmark must contain ALL specified tags.
    /// Tag matching is case-insensitive.
    /// 
    /// # Arguments
    /// * `tags` - Vector of tags that must all be present
    /// 
    /// # Returns
    /// Vector of bookmarks containing all specified tags
    async fn find_by_tags(&self, tags: &[String]) -> BookmarkResult<Vec<Bookmark>>;
    
    /// Add a note to an existing bookmark
    /// 
    /// # Arguments
    /// * `bookmark_id` - ID of the bookmark to add note to
    /// * `content` - Content of the note
    /// 
    /// # Returns
    /// The note ID of the created note
    /// 
    /// # CRDT Behavior
    /// Adds to the notes collection using CRDT list semantics
    async fn add_note(&mut self, bookmark_id: &str, content: &str) -> BookmarkResult<String>;
    
    /// Remove a note from a bookmark
    /// 
    /// # Arguments
    /// * `bookmark_id` - ID of the bookmark to remove note from
    /// * `note_id` - ID of the note to remove
    /// 
    /// # CRDT Behavior
    /// Marks note as deleted using tombstone in CRDT list
    async fn remove_note(&mut self, bookmark_id: &str, note_id: &str) -> BookmarkResult<()>;
    
    /// Generate sync message for a peer
    /// 
    /// # Arguments
    /// * `peer_id` - The ID of the peer to sync with
    /// 
    /// # Returns
    /// The sync message as bytes
    async fn generate_sync_message(&mut self, peer_id: &str) -> BookmarkResult<Vec<u8>>;
    
    /// Apply sync message from a peer
    /// 
    /// # Arguments
    /// * `peer_id` - The ID of the peer the message is from
    /// * `message` - The sync message as bytes
    /// 
    /// # Returns
    /// Whether any changes were applied
    async fn apply_sync_message(&mut self, peer_id: &str, message: Vec<u8>) -> BookmarkResult<bool>;
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
    
    fn apply_filters(&self, mut bookmarks: Vec<Bookmark>, filters: &BookmarkFilters) -> Vec<Bookmark> {
        // Apply text query filter
        if let Some(ref query) = filters.text_query {
            let query_lower = query.to_lowercase();
            bookmarks.retain(|bookmark| {
                bookmark.title.to_lowercase().contains(&query_lower) ||
                bookmark.url.to_lowercase().contains(&query_lower) ||
                bookmark.author.as_ref().map_or(false, |author| author.to_lowercase().contains(&query_lower)) ||
                bookmark.notes.iter().any(|note| note.content.to_lowercase().contains(&query_lower))
            });
        }
        
        // Apply tags filter (AND logic - must contain ALL tags)
        if let Some(ref filter_tags) = filters.tags {
            let tags_lower: Vec<String> = filter_tags.iter().map(|tag| tag.to_lowercase()).collect();
            bookmarks.retain(|bookmark| {
                tags_lower.iter().all(|tag| {
                    bookmark.tags.iter().any(|bookmark_tag| bookmark_tag.to_lowercase() == *tag)
                })
            });
        }
        
        // Apply reading status filter
        if let Some(ref status) = filters.reading_status {
            bookmarks.retain(|bookmark| bookmark.reading_status == *status);
        }
        
        // Apply priority range filter
        if let Some((min_priority, max_priority)) = filters.priority_range {
            bookmarks.retain(|bookmark| {
                if let Some(priority) = bookmark.priority_rating {
                    priority >= min_priority && priority <= max_priority
                } else {
                    false // If no priority set, exclude from priority range filter
                }
            });
        }
        
        bookmarks
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

    async fn find_all(&self, filters: Option<BookmarkFilters>) -> BookmarkResult<Vec<Bookmark>> {
        let mut bookmarks: Vec<Bookmark> = self.bookmarks.values().cloned().collect();
        
        if let Some(filters) = filters {
            bookmarks = self.apply_filters(bookmarks, &filters);
        }
        
        Ok(bookmarks)
    }
    
    async fn find_by_id(&self, id: &str) -> BookmarkResult<Bookmark> {
        self.bookmarks
            .get(id)
            .cloned()
            .ok_or_else(|| BookmarkError::NotFound(id.to_string()))
    }
    
    async fn update(&mut self, bookmark: Bookmark) -> BookmarkResult<Bookmark> {
        let id = bookmark.id.clone();
        if self.bookmarks.contains_key(&id) {
            self.bookmarks.insert(id, bookmark.clone());
            Ok(bookmark)
        } else {
            Err(BookmarkError::NotFound(id))
        }
    }

    async fn delete(&mut self, id: &str) -> BookmarkResult<()> {
        match self.bookmarks.remove(id) {
            Some(_) => Ok(()),
            None => Err(BookmarkError::NotFound(id.to_string())),
        }
    }
    
    async fn search_by_text(&self, query: &str) -> BookmarkResult<Vec<Bookmark>> {
        let query_lower = query.to_lowercase();
        let results = self.bookmarks
            .values()
            .filter(|bookmark| {
                bookmark.title.to_lowercase().contains(&query_lower) ||
                bookmark.url.to_lowercase().contains(&query_lower) ||
                bookmark.author.as_ref().map_or(false, |author| author.to_lowercase().contains(&query_lower)) ||
                bookmark.notes.iter().any(|note| note.content.to_lowercase().contains(&query_lower))
            })
            .cloned()
            .collect();
            
        Ok(results)
    }
    
    async fn find_by_tags(&self, tags: &[String]) -> BookmarkResult<Vec<Bookmark>> {
        let tags_lower: Vec<String> = tags.iter().map(|tag| tag.to_lowercase()).collect();
        let results = self.bookmarks
            .values()
            .filter(|bookmark| {
                tags_lower.iter().all(|tag| {
                    bookmark.tags.iter().any(|bookmark_tag| bookmark_tag.to_lowercase() == *tag)
                })
            })
            .cloned()
            .collect();
            
        Ok(results)
    }
    
    async fn add_note(&mut self, bookmark_id: &str, content: &str) -> BookmarkResult<String> {
        if let Some(bookmark) = self.bookmarks.get_mut(bookmark_id) {
            let note_id = bookmark.add_note(content);
            Ok(note_id)
        } else {
            Err(BookmarkError::NotFound(bookmark_id.to_string()))
        }
    }
    
    async fn remove_note(&mut self, bookmark_id: &str, note_id: &str) -> BookmarkResult<()> {
        if let Some(bookmark) = self.bookmarks.get_mut(bookmark_id) {
            if bookmark.remove_note(note_id) {
                Ok(())
            } else {
                Err(BookmarkError::NotFound(format!("Note {} not found", note_id)))
            }
        } else {
            Err(BookmarkError::NotFound(bookmark_id.to_string()))
        }
    }
    
    async fn generate_sync_message(&mut self, _peer_id: &str) -> BookmarkResult<Vec<u8>> {
        // Mock implementation - return empty message
        Ok(vec![])
    }
    
    async fn apply_sync_message(&mut self, _peer_id: &str, _message: Vec<u8>) -> BookmarkResult<bool> {
        // Mock implementation - no changes applied
        Ok(false)
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
        
        let bookmarks = repo.find_all(None).await.unwrap();
        
        assert!(bookmarks.is_empty());
    }

    #[tokio::test]
    async fn test_find_all_with_bookmarks() {
        let mut repo = MockBookmarkRepository::new();
        let bookmark1 = Bookmark::new("https://example.com", "Example 1").unwrap();
        let bookmark2 = Bookmark::new("https://test.com", "Test").unwrap();
        
        repo.create(bookmark1.clone()).await.unwrap();
        repo.create(bookmark2.clone()).await.unwrap();
        
        let bookmarks = repo.find_all(None).await.unwrap();
        
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
        let bookmarks = repo.find_all(None).await.unwrap();
        assert_eq!(bookmarks.len(), 1);
        
        // Delete it
        let result = repo.delete(&bookmark_id).await;
        assert!(result.is_ok());
        
        // Verify it's gone
        let bookmarks = repo.find_all(None).await.unwrap();
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
        let bookmarks = repo.find_all(None).await.unwrap();
        assert_eq!(bookmarks.len(), 1);
        
        // Add another
        let bookmark2 = Bookmark::new("https://test.com", "Test").unwrap();
        repo.create(bookmark2).await.unwrap();
        
        // Verify both exist
        let bookmarks = repo.find_all(None).await.unwrap();
        assert_eq!(bookmarks.len(), 2);
        
        // Delete first one
        repo.delete(&bookmark_id).await.unwrap();
        
        // Verify only one remains
        let bookmarks = repo.find_all(None).await.unwrap();
        assert_eq!(bookmarks.len(), 1);
        assert_eq!(bookmarks[0].url, "https://test.com");
    }

    // New comprehensive tests for enhanced functionality
    
    #[tokio::test]
    async fn test_find_by_id_existing() {
        let mut repo = MockBookmarkRepository::new();
        let bookmark = Bookmark::new("https://example.com", "Example").unwrap();
        let bookmark_id = bookmark.id.clone();
        
        repo.create(bookmark.clone()).await.unwrap();
        
        let found = repo.find_by_id(&bookmark_id).await.unwrap();
        assert_eq!(found, bookmark);
    }
    
    #[tokio::test]
    async fn test_find_by_id_nonexistent() {
        let repo = MockBookmarkRepository::new();
        
        let result = repo.find_by_id("nonexistent").await;
        assert!(matches!(result, Err(BookmarkError::NotFound(_))));
    }
    
    #[tokio::test]
    async fn test_update_existing_bookmark() {
        let mut repo = MockBookmarkRepository::new();
        let mut bookmark = Bookmark::new("https://example.com", "Original Title").unwrap();
        let bookmark_id = bookmark.id.clone();
        
        repo.create(bookmark.clone()).await.unwrap();
        
        // Update the bookmark
        bookmark.title = "Updated Title".to_string();
        let updated = repo.update(bookmark.clone()).await.unwrap();
        
        assert_eq!(updated.title, "Updated Title");
        
        // Verify it's persisted
        let found = repo.find_by_id(&bookmark_id).await.unwrap();
        assert_eq!(found.title, "Updated Title");
    }
    
    #[tokio::test]
    async fn test_update_nonexistent_bookmark() {
        let mut repo = MockBookmarkRepository::new();
        let bookmark = Bookmark::new("https://example.com", "Test").unwrap();
        
        let result = repo.update(bookmark).await;
        assert!(matches!(result, Err(BookmarkError::NotFound(_))));
    }
    
    #[tokio::test]
    async fn test_search_by_text_in_title() {
        let mut repo = MockBookmarkRepository::new();
        let bookmark1 = Bookmark::new("https://example.com", "Rust Programming").unwrap();
        let bookmark2 = Bookmark::new("https://test.com", "Python Guide").unwrap();
        
        repo.create(bookmark1.clone()).await.unwrap();
        repo.create(bookmark2).await.unwrap();
        
        let results = repo.search_by_text("rust").await.unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].title, "Rust Programming");
    }
    
    #[tokio::test]
    async fn test_search_by_text_in_url() {
        let mut repo = MockBookmarkRepository::new();
        let bookmark1 = Bookmark::new("https://rust-lang.org", "Programming").unwrap();
        let bookmark2 = Bookmark::new("https://python.org", "Programming").unwrap();
        
        repo.create(bookmark1.clone()).await.unwrap();
        repo.create(bookmark2).await.unwrap();
        
        let results = repo.search_by_text("rust-lang").await.unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].url, "https://rust-lang.org");
    }
    
    #[tokio::test]
    async fn test_search_by_text_case_insensitive() {
        let mut repo = MockBookmarkRepository::new();
        let bookmark = Bookmark::new("https://example.com", "RUST Programming").unwrap();
        
        repo.create(bookmark.clone()).await.unwrap();
        
        let results = repo.search_by_text("rust").await.unwrap();
        assert_eq!(results.len(), 1);
        
        let results = repo.search_by_text("PROGRAMMING").await.unwrap();
        assert_eq!(results.len(), 1);
    }
    
    #[tokio::test]
    async fn test_search_by_text_in_notes() {
        let mut repo = MockBookmarkRepository::new();
        let mut bookmark = Bookmark::new("https://example.com", "Test").unwrap();
        bookmark.add_note("This is about Rust programming");
        
        let created = repo.create(bookmark).await.unwrap();
        
        let results = repo.search_by_text("rust programming").await.unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, created.id);
    }
    
    #[tokio::test]
    async fn test_find_by_tags_single_tag() {
        let mut repo = MockBookmarkRepository::new();
        let bookmark1 = Bookmark::new("https://example.com", "Rust").unwrap()
            .with_tags(vec!["programming".to_string(), "rust".to_string()]);
        let bookmark2 = Bookmark::new("https://test.com", "Python").unwrap()
            .with_tags(vec!["programming".to_string(), "python".to_string()]);
        
        repo.create(bookmark1.clone()).await.unwrap();
        repo.create(bookmark2).await.unwrap();
        
        let results = repo.find_by_tags(&["rust".to_string()]).await.unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].title, "Rust");
    }
    
    #[tokio::test]
    async fn test_find_by_tags_multiple_tags_and_logic() {
        let mut repo = MockBookmarkRepository::new();
        let bookmark1 = Bookmark::new("https://example.com", "Rust Web").unwrap()
            .with_tags(vec!["programming".to_string(), "rust".to_string(), "web".to_string()]);
        let bookmark2 = Bookmark::new("https://test.com", "Rust CLI").unwrap()
            .with_tags(vec!["programming".to_string(), "rust".to_string()]);
        
        repo.create(bookmark1.clone()).await.unwrap();
        repo.create(bookmark2).await.unwrap();
        
        // Should find only the bookmark with both rust AND web tags
        let results = repo.find_by_tags(&["rust".to_string(), "web".to_string()]).await.unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].title, "Rust Web");
    }
    
    #[tokio::test]
    async fn test_find_by_tags_case_insensitive() {
        let mut repo = MockBookmarkRepository::new();
        let bookmark = Bookmark::new("https://example.com", "Test").unwrap()
            .with_tags(vec!["PROGRAMMING".to_string(), "Rust".to_string()]);
        
        repo.create(bookmark.clone()).await.unwrap();
        
        let results = repo.find_by_tags(&["programming".to_string()]).await.unwrap();
        assert_eq!(results.len(), 1);
        
        let results = repo.find_by_tags(&["rust".to_string()]).await.unwrap();
        assert_eq!(results.len(), 1);
    }
    
    #[tokio::test]
    async fn test_add_note_to_existing_bookmark() {
        let mut repo = MockBookmarkRepository::new();
        let bookmark = Bookmark::new("https://example.com", "Test").unwrap();
        let bookmark_id = bookmark.id.clone();
        
        repo.create(bookmark).await.unwrap();
        
        let note_id = repo.add_note(&bookmark_id, "Test note content").await.unwrap();
        assert!(!note_id.is_empty());
        
        // Verify note was added
        let found = repo.find_by_id(&bookmark_id).await.unwrap();
        assert_eq!(found.notes.len(), 1);
        assert_eq!(found.notes[0].content, "Test note content");
        assert_eq!(found.notes[0].id, note_id);
    }
    
    #[tokio::test]
    async fn test_add_note_to_nonexistent_bookmark() {
        let mut repo = MockBookmarkRepository::new();
        
        let result = repo.add_note("nonexistent", "Test note").await;
        assert!(matches!(result, Err(BookmarkError::NotFound(_))));
    }
    
    #[tokio::test]
    async fn test_remove_note_from_bookmark() {
        let mut repo = MockBookmarkRepository::new();
        let bookmark = Bookmark::new("https://example.com", "Test").unwrap();
        let bookmark_id = bookmark.id.clone();
        
        repo.create(bookmark).await.unwrap();
        
        // Add a note
        let note_id = repo.add_note(&bookmark_id, "Test note").await.unwrap();
        
        // Verify it exists
        let found = repo.find_by_id(&bookmark_id).await.unwrap();
        assert_eq!(found.notes.len(), 1);
        
        // Remove the note
        let result = repo.remove_note(&bookmark_id, &note_id).await;
        assert!(result.is_ok());
        
        // Verify it's gone
        let found = repo.find_by_id(&bookmark_id).await.unwrap();
        assert_eq!(found.notes.len(), 0);
    }
    
    #[tokio::test]
    async fn test_remove_note_nonexistent_note() {
        let mut repo = MockBookmarkRepository::new();
        let bookmark = Bookmark::new("https://example.com", "Test").unwrap();
        let bookmark_id = bookmark.id.clone();
        
        repo.create(bookmark).await.unwrap();
        
        let result = repo.remove_note(&bookmark_id, "nonexistent").await;
        assert!(matches!(result, Err(BookmarkError::NotFound(_))));
    }
    
    #[tokio::test]
    async fn test_remove_note_nonexistent_bookmark() {
        let mut repo = MockBookmarkRepository::new();
        
        let result = repo.remove_note("nonexistent", "note_id").await;
        assert!(matches!(result, Err(BookmarkError::NotFound(_))));
    }
    
    #[tokio::test]
    async fn test_find_all_with_text_filter() {
        use crate::types::BookmarkFilters;
        
        let mut repo = MockBookmarkRepository::new();
        let bookmark1 = Bookmark::new("https://example.com", "Rust Programming").unwrap();
        let bookmark2 = Bookmark::new("https://test.com", "Python Guide").unwrap();
        
        repo.create(bookmark1).await.unwrap();
        repo.create(bookmark2).await.unwrap();
        
        let filters = BookmarkFilters {
            text_query: Some("rust".to_string()),
            ..Default::default()
        };
        
        let results = repo.find_all(Some(filters)).await.unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].title, "Rust Programming");
    }
    
    #[tokio::test]
    async fn test_find_all_with_tags_filter() {
        use crate::types::BookmarkFilters;
        
        let mut repo = MockBookmarkRepository::new();
        let bookmark1 = Bookmark::new("https://example.com", "Rust").unwrap()
            .with_tags(vec!["programming".to_string(), "rust".to_string()]);
        let bookmark2 = Bookmark::new("https://test.com", "Python").unwrap()
            .with_tags(vec!["programming".to_string(), "python".to_string()]);
        
        repo.create(bookmark1).await.unwrap();
        repo.create(bookmark2).await.unwrap();
        
        let filters = BookmarkFilters {
            tags: Some(vec!["rust".to_string()]),
            ..Default::default()
        };
        
        let results = repo.find_all(Some(filters)).await.unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].title, "Rust");
    }
    
    #[tokio::test]
    async fn test_find_all_with_reading_status_filter() {
        use crate::types::{BookmarkFilters, ReadingStatus};
        
        let mut repo = MockBookmarkRepository::new();
        let mut bookmark1 = Bookmark::new("https://example.com", "Read Article").unwrap();
        bookmark1.reading_status = ReadingStatus::Completed;
        let bookmark2 = Bookmark::new("https://test.com", "Unread Article").unwrap();
        
        repo.create(bookmark1).await.unwrap();
        repo.create(bookmark2).await.unwrap();
        
        let filters = BookmarkFilters {
            reading_status: Some(ReadingStatus::Completed),
            ..Default::default()
        };
        
        let results = repo.find_all(Some(filters)).await.unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].title, "Read Article");
    }
    
    #[tokio::test]
    async fn test_find_all_with_priority_range_filter() {
        use crate::types::BookmarkFilters;
        
        let mut repo = MockBookmarkRepository::new();
        let bookmark1 = Bookmark::new("https://example.com", "High Priority").unwrap()
            .with_priority(5).unwrap();
        let bookmark2 = Bookmark::new("https://test.com", "Low Priority").unwrap()
            .with_priority(2).unwrap();
        
        repo.create(bookmark1).await.unwrap();
        repo.create(bookmark2).await.unwrap();
        
        let filters = BookmarkFilters {
            priority_range: Some((4, 5)),
            ..Default::default()
        };
        
        let results = repo.find_all(Some(filters)).await.unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].title, "High Priority");
    }
    
    #[tokio::test]
    async fn test_find_all_with_multiple_filters() {
        use crate::types::{BookmarkFilters, ReadingStatus};
        
        let mut repo = MockBookmarkRepository::new();
        let mut bookmark1 = Bookmark::new("https://example.com", "Rust Programming").unwrap()
            .with_tags(vec!["programming".to_string(), "rust".to_string()])
            .with_priority(5).unwrap();
        bookmark1.reading_status = ReadingStatus::Completed;
        
        let mut bookmark2 = Bookmark::new("https://test.com", "Rust Guide").unwrap()
            .with_tags(vec!["programming".to_string(), "rust".to_string()])
            .with_priority(3).unwrap();
        bookmark2.reading_status = ReadingStatus::Unread;
        
        repo.create(bookmark1).await.unwrap();
        repo.create(bookmark2).await.unwrap();
        
        let filters = BookmarkFilters {
            text_query: Some("rust".to_string()),
            tags: Some(vec!["programming".to_string()]),
            reading_status: Some(ReadingStatus::Completed),
            priority_range: Some((4, 5)),
            bookmarked_since: None,
            bookmarked_until: None,
            published_since: None,
            published_until: None,
            sort_by: None,
            sort_order: None,
        };
        
        let results = repo.find_all(Some(filters)).await.unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].title, "Rust Programming");
    }
    
    #[tokio::test]
    async fn test_find_all_empty_filters_returns_all() {
        let mut repo = MockBookmarkRepository::new();
        let bookmark1 = Bookmark::new("https://example.com", "Test 1").unwrap();
        let bookmark2 = Bookmark::new("https://test.com", "Test 2").unwrap();
        
        repo.create(bookmark1).await.unwrap();
        repo.create(bookmark2).await.unwrap();
        
        let filters = BookmarkFilters::default();
        let results = repo.find_all(Some(filters)).await.unwrap();
        assert_eq!(results.len(), 2);
    }
}