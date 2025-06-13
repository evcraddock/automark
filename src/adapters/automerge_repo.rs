use crate::traits::BookmarkRepository;
use crate::types::{Bookmark, BookmarkResult, BookmarkError, BookmarkFilters};
use async_trait::async_trait;
use automerge::{AutoCommit, ObjType, ReadDoc, ROOT};
use automerge::transaction::Transactable;
use std::path::PathBuf;
use std::fs;
use chrono::{DateTime, Utc};

pub struct AutomergeBookmarkRepository {
    doc: AutoCommit,
    bookmarks_list: automerge::ObjId,
    file_path: PathBuf,
}

impl AutomergeBookmarkRepository {
    pub fn new(file_path: PathBuf) -> BookmarkResult<Self> {
        // Create parent directories if they don't exist
        if let Some(parent) = file_path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| BookmarkError::InvalidUrl(format!("Failed to create directory: {}", e)))?;
        }

        let (doc, bookmarks_list) = Self::load_from_file(&file_path)?;

        Ok(Self { doc, bookmarks_list, file_path })
    }

    fn load_from_file(path: &PathBuf) -> BookmarkResult<(AutoCommit, automerge::ObjId)> {
        let mut doc = if path.exists() {
            let bytes = fs::read(path)
                .map_err(|e| BookmarkError::InvalidUrl(format!("Failed to read file: {}", e)))?;
            
            if bytes.is_empty() {
                AutoCommit::new()
            } else {
                AutoCommit::load(&bytes)
                    .map_err(|e| BookmarkError::InvalidUrl(format!("Failed to load Automerge document: {}", e)))?
            }
        } else {
            AutoCommit::new()
        };

        // Get or create the bookmarks list
        let bookmarks_list = match doc.get(ROOT, "bookmarks")
            .map_err(|e| BookmarkError::InvalidUrl(format!("Failed to get bookmarks: {}", e)))? {
            Some((_, obj_id)) => obj_id,
            None => {
                doc.put_object(ROOT, "bookmarks", ObjType::List)
                    .map_err(|e| BookmarkError::InvalidUrl(format!("Failed to create bookmarks list: {}", e)))?
            }
        };

        Ok((doc, bookmarks_list))
    }

    fn save(&mut self) -> BookmarkResult<()> {
        let bytes = self.doc.save();
        
        // Use atomic write: write to temp file, then rename
        let temp_path = self.file_path.with_extension("tmp");
        
        fs::write(&temp_path, bytes)
            .map_err(|e| BookmarkError::InvalidUrl(format!("Failed to write temp file: {}", e)))?;
        
        fs::rename(&temp_path, &self.file_path)
            .map_err(|e| BookmarkError::InvalidUrl(format!("Failed to rename file: {}", e)))?;
        
        Ok(())
    }

    fn bookmark_from_automerge(&self, obj_id: &automerge::ObjId) -> BookmarkResult<Bookmark> {
        let id = self.doc.get(obj_id, "id")
            .map_err(|e| BookmarkError::InvalidUrl(format!("Failed to get bookmark id: {}", e)))?
            .and_then(|(value, _)| value.to_str().map(|s| s.to_string()))
            .ok_or_else(|| BookmarkError::InvalidUrl("Bookmark missing id".to_string()))?;

        let url = self.doc.get(obj_id, "url")
            .map_err(|e| BookmarkError::InvalidUrl(format!("Failed to get bookmark url: {}", e)))?
            .and_then(|(value, _)| value.to_str().map(|s| s.to_string()))
            .ok_or_else(|| BookmarkError::InvalidUrl("Bookmark missing url".to_string()))?;

        let title = self.doc.get(obj_id, "title")
            .map_err(|e| BookmarkError::InvalidUrl(format!("Failed to get bookmark title: {}", e)))?
            .and_then(|(value, _)| value.to_str().map(|s| s.to_string()))
            .ok_or_else(|| BookmarkError::InvalidUrl("Bookmark missing title".to_string()))?;

        let date_str = self.doc.get(obj_id, "bookmarked_date")
            .map_err(|e| BookmarkError::InvalidUrl(format!("Failed to get bookmark date: {}", e)))?
            .and_then(|(value, _)| value.to_str().map(|s| s.to_string()))
            .ok_or_else(|| BookmarkError::InvalidUrl("Bookmark missing date".to_string()))?;

        let bookmarked_date = DateTime::parse_from_rfc3339(&date_str)
            .map_err(|e| BookmarkError::InvalidUrl(format!("Failed to parse date: {}", e)))?
            .with_timezone(&Utc);

        Ok(Bookmark {
            id,
            url,
            title,
            bookmarked_date,
            author: None,
            tags: Vec::new(),
            publish_date: None,
            notes: Vec::new(),
            reading_status: crate::types::ReadingStatus::Unread,
            priority_rating: None,
        })
    }

    fn add_bookmark_to_automerge(&mut self, bookmark: &Bookmark) -> BookmarkResult<()> {
        // Find the next index in the list
        let list_len = self.doc.length(&self.bookmarks_list);
        
        // Create a new bookmark object
        let bookmark_obj = self.doc.insert_object(&self.bookmarks_list, list_len, ObjType::Map)
            .map_err(|e| BookmarkError::InvalidUrl(format!("Failed to insert bookmark object: {}", e)))?;

        // Set bookmark properties
        self.doc.put(&bookmark_obj, "id", bookmark.id.clone())
            .map_err(|e| BookmarkError::InvalidUrl(format!("Failed to set bookmark id: {}", e)))?;
        
        self.doc.put(&bookmark_obj, "url", bookmark.url.clone())
            .map_err(|e| BookmarkError::InvalidUrl(format!("Failed to set bookmark url: {}", e)))?;
        
        self.doc.put(&bookmark_obj, "title", bookmark.title.clone())
            .map_err(|e| BookmarkError::InvalidUrl(format!("Failed to set bookmark title: {}", e)))?;
        
        self.doc.put(&bookmark_obj, "bookmarked_date", bookmark.bookmarked_date.to_rfc3339())
            .map_err(|e| BookmarkError::InvalidUrl(format!("Failed to set bookmark date: {}", e)))?;

        Ok(())
    }

    fn find_bookmark_index(&self, id: &str) -> BookmarkResult<Option<usize>> {
        let list_len = self.doc.length(&self.bookmarks_list);
        
        for i in 0..list_len {
            if let Ok(Some((_, obj_id))) = self.doc.get(&self.bookmarks_list, i) {
                if let Ok(Some((value, _))) = self.doc.get(&obj_id, "id") {
                    if let Some(bookmark_id) = value.to_str() {
                        if bookmark_id == id {
                            return Ok(Some(i));
                        }
                    }
                }
            }
        }
        
        Ok(None)
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

#[async_trait]
impl BookmarkRepository for AutomergeBookmarkRepository {
    async fn create(&mut self, bookmark: Bookmark) -> BookmarkResult<Bookmark> {
        self.add_bookmark_to_automerge(&bookmark)?;
        self.save()?;
        Ok(bookmark)
    }

    async fn find_all(&self, filters: Option<BookmarkFilters>) -> BookmarkResult<Vec<Bookmark>> {
        let mut bookmarks = Vec::new();
        let list_len = self.doc.length(&self.bookmarks_list);
        
        for i in 0..list_len {
            if let Ok(Some((_, obj_id))) = self.doc.get(&self.bookmarks_list, i) {
                match self.bookmark_from_automerge(&obj_id) {
                    Ok(bookmark) => bookmarks.push(bookmark),
                    Err(_) => continue, // Skip corrupted bookmarks
                }
            }
        }
        
        // Apply filters if provided
        if let Some(filters) = filters {
            bookmarks = self.apply_filters(bookmarks, &filters);
        }
        
        Ok(bookmarks)
    }
    
    async fn find_by_id(&self, id: &str) -> BookmarkResult<Bookmark> {
        let list_len = self.doc.length(&self.bookmarks_list);
        
        for i in 0..list_len {
            if let Ok(Some((_, obj_id))) = self.doc.get(&self.bookmarks_list, i) {
                if let Ok(bookmark) = self.bookmark_from_automerge(&obj_id) {
                    if bookmark.id == id {
                        return Ok(bookmark);
                    }
                }
            }
        }
        
        Err(BookmarkError::NotFound(id.to_string()))
    }
    
    async fn update(&mut self, bookmark: Bookmark) -> BookmarkResult<Bookmark> {
        // For now, implement as delete + create
        // TODO: Implement proper CRDT field-level updates
        self.delete(&bookmark.id).await?;
        self.create(bookmark).await
    }
    
    async fn search_by_text(&self, query: &str) -> BookmarkResult<Vec<Bookmark>> {
        let all_bookmarks = self.find_all(None).await?;
        let query_lower = query.to_lowercase();
        
        let results = all_bookmarks
            .into_iter()
            .filter(|bookmark| {
                bookmark.title.to_lowercase().contains(&query_lower) ||
                bookmark.url.to_lowercase().contains(&query_lower) ||
                bookmark.author.as_ref().map_or(false, |author| author.to_lowercase().contains(&query_lower)) ||
                bookmark.notes.iter().any(|note| note.content.to_lowercase().contains(&query_lower))
            })
            .collect();
            
        Ok(results)
    }
    
    async fn find_by_tags(&self, tags: &[String]) -> BookmarkResult<Vec<Bookmark>> {
        let all_bookmarks = self.find_all(None).await?;
        let tags_lower: Vec<String> = tags.iter().map(|tag| tag.to_lowercase()).collect();
        
        let results = all_bookmarks
            .into_iter()
            .filter(|bookmark| {
                tags_lower.iter().all(|tag| {
                    bookmark.tags.iter().any(|bookmark_tag| bookmark_tag.to_lowercase() == *tag)
                })
            })
            .collect();
            
        Ok(results)
    }
    
    async fn add_note(&mut self, bookmark_id: &str, content: &str) -> BookmarkResult<String> {
        // Find and update the bookmark
        let mut bookmark = self.find_by_id(bookmark_id).await?;
        let note_id = bookmark.add_note(content);
        self.update(bookmark).await?;
        Ok(note_id)
    }
    
    async fn remove_note(&mut self, bookmark_id: &str, note_id: &str) -> BookmarkResult<()> {
        let mut bookmark = self.find_by_id(bookmark_id).await?;
        if bookmark.remove_note(note_id) {
            self.update(bookmark).await?;
            Ok(())
        } else {
            Err(BookmarkError::NotFound(format!("Note {} not found", note_id)))
        }
    }

    async fn delete(&mut self, id: &str) -> BookmarkResult<()> {
        let index = self.find_bookmark_index(id)?
            .ok_or_else(|| BookmarkError::NotFound(id.to_string()))?;
            
        self.doc.delete(&self.bookmarks_list, index)
            .map_err(|e| BookmarkError::InvalidUrl(format!("Failed to delete bookmark: {}", e)))?;
            
        self.save()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn create_test_repo() -> (AutomergeBookmarkRepository, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test_bookmarks.automerge");
        let repo = AutomergeBookmarkRepository::new(file_path).unwrap();
        (repo, temp_dir)
    }

    #[tokio::test]
    async fn test_create_repository_with_new_file() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("new_bookmarks.automerge");
        
        let repo = AutomergeBookmarkRepository::new(file_path.clone());
        assert!(repo.is_ok());
        
        // File should not exist yet (only created on first save)
        assert!(!file_path.exists());
    }

    #[tokio::test]
    async fn test_create_repository_with_nested_path() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("nested").join("dir").join("bookmarks.automerge");
        
        let repo = AutomergeBookmarkRepository::new(file_path.clone());
        assert!(repo.is_ok());
        
        // Parent directories should be created
        assert!(file_path.parent().unwrap().exists());
    }

    #[tokio::test]
    async fn test_add_bookmark_and_save() {
        let (mut repo, _temp_dir) = create_test_repo();
        let bookmark = Bookmark::new("https://example.com", "Example").unwrap();
        
        let result = repo.create(bookmark.clone()).await;
        assert!(result.is_ok());
        
        // File should exist after save
        assert!(repo.file_path.exists());
        
        // Verify the bookmark was added
        let bookmarks = repo.find_all(None).await.unwrap();
        assert_eq!(bookmarks.len(), 1);
        assert_eq!(bookmarks[0].url, "https://example.com");
        assert_eq!(bookmarks[0].title, "Example");
    }

    #[tokio::test]
    async fn test_persistence_across_instances() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("persistent_bookmarks.automerge");
        
        // Create bookmark in first instance
        {
            let mut repo = AutomergeBookmarkRepository::new(file_path.clone()).unwrap();
            let bookmark = Bookmark::new("https://example.com", "Example").unwrap();
            repo.create(bookmark).await.unwrap();
        }
        
        // Load in second instance and verify bookmark exists
        {
            let repo = AutomergeBookmarkRepository::new(file_path).unwrap();
            let bookmarks = repo.find_all(None).await.unwrap();
            assert_eq!(bookmarks.len(), 1);
            assert_eq!(bookmarks[0].url, "https://example.com");
            assert_eq!(bookmarks[0].title, "Example");
        }
    }

    #[tokio::test]
    async fn test_find_all_empty() {
        let (repo, _temp_dir) = create_test_repo();
        
        let bookmarks = repo.find_all(None).await.unwrap();
        assert!(bookmarks.is_empty());
    }

    #[tokio::test]
    async fn test_find_all_with_multiple_bookmarks() {
        let (mut repo, _temp_dir) = create_test_repo();
        
        let bookmark1 = Bookmark::new("https://example.com", "Example").unwrap();
        let bookmark2 = Bookmark::new("https://test.com", "Test").unwrap();
        
        repo.create(bookmark1.clone()).await.unwrap();
        repo.create(bookmark2.clone()).await.unwrap();
        
        let bookmarks = repo.find_all(None).await.unwrap();
        assert_eq!(bookmarks.len(), 2);
        
        let urls: Vec<_> = bookmarks.iter().map(|b| &b.url).collect();
        assert!(urls.contains(&&"https://example.com".to_string()));
        assert!(urls.contains(&&"https://test.com".to_string()));
    }

    #[tokio::test]
    async fn test_delete_existing_bookmark() {
        let (mut repo, _temp_dir) = create_test_repo();
        
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
        let (mut repo, _temp_dir) = create_test_repo();
        
        let result = repo.delete("nonexistent-id").await;
        assert!(matches!(result, Err(BookmarkError::NotFound(_))));
    }

    #[tokio::test]
    async fn test_delete_persistence() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("delete_test.automerge");
        
        let bookmark_id = {
            let mut repo = AutomergeBookmarkRepository::new(file_path.clone()).unwrap();
            let bookmark = Bookmark::new("https://example.com", "Example").unwrap();
            let id = bookmark.id.clone();
            repo.create(bookmark).await.unwrap();
            
            // Delete the bookmark
            repo.delete(&id).await.unwrap();
            id
        };
        
        // Verify deletion persisted
        {
            let repo = AutomergeBookmarkRepository::new(file_path).unwrap();
            let bookmarks = repo.find_all(None).await.unwrap();
            assert!(bookmarks.is_empty());
            
            // Try to delete again - should fail
            let mut repo = repo;
            let result = repo.delete(&bookmark_id).await;
            assert!(matches!(result, Err(BookmarkError::NotFound(_))));
        }
    }

    #[tokio::test]
    async fn test_automerge_document_structure() {
        let (mut repo, _temp_dir) = create_test_repo();
        
        // Add some bookmarks
        let bookmark1 = Bookmark::new("https://example.com", "Example").unwrap();
        let bookmark2 = Bookmark::new("https://test.com", "Test").unwrap();
        
        repo.create(bookmark1.clone()).await.unwrap();
        repo.create(bookmark2.clone()).await.unwrap();
        
        // Verify the document structure
        let doc_bytes = repo.doc.save();
        assert!(!doc_bytes.is_empty());
        
        // Verify we can load the document
        let loaded_doc = AutoCommit::load(&doc_bytes).unwrap();
        assert!(loaded_doc.get(ROOT, "bookmarks").unwrap().is_some());
        
        // Verify list length
        let (_, bookmarks_list) = loaded_doc.get(ROOT, "bookmarks").unwrap().unwrap();
        assert_eq!(loaded_doc.length(&bookmarks_list), 2);
    }

    #[tokio::test]
    async fn test_bookmark_data_integrity() {
        let (mut repo, _temp_dir) = create_test_repo();
        
        let original_bookmark = Bookmark::new("https://example.com", "Example Site").unwrap();
        let original_id = original_bookmark.id.clone();
        let original_date = original_bookmark.bookmarked_date;
        
        repo.create(original_bookmark).await.unwrap();
        
        let retrieved_bookmarks = repo.find_all(None).await.unwrap();
        assert_eq!(retrieved_bookmarks.len(), 1);
        
        let retrieved_bookmark = &retrieved_bookmarks[0];
        assert_eq!(retrieved_bookmark.id, original_id);
        assert_eq!(retrieved_bookmark.url, "https://example.com");
        assert_eq!(retrieved_bookmark.title, "Example Site");
        assert_eq!(retrieved_bookmark.bookmarked_date, original_date);
    }
}