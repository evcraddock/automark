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
    bookmarks_map: automerge::ObjId,
    file_path: PathBuf,
}

impl AutomergeBookmarkRepository {
    pub fn new(file_path: PathBuf) -> BookmarkResult<Self> {
        // Create parent directories if they don't exist
        if let Some(parent) = file_path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| BookmarkError::InvalidUrl(format!("Failed to create directory: {}", e)))?;
        }

        let (doc, bookmarks_map) = Self::load_from_file(&file_path)?;

        Ok(Self { doc, bookmarks_map, file_path })
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

        // Get or create the bookmarks map
        let bookmarks_map = match doc.get(ROOT, "bookmarks")
            .map_err(|e| BookmarkError::InvalidUrl(format!("Failed to get bookmarks: {}", e)))? {
            Some((_, obj_id)) => obj_id,
            None => {
                doc.put_object(ROOT, "bookmarks", ObjType::Map)
                    .map_err(|e| BookmarkError::InvalidUrl(format!("Failed to create bookmarks map: {}", e)))?
            }
        };

        Ok((doc, bookmarks_map))
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
        // Extract basic fields
        let id = self.get_string_field(obj_id, "id")?;
        let url = self.get_string_field(obj_id, "url")?;
        let title = self.get_string_field(obj_id, "title")?;
        
        let date_str = self.get_string_field(obj_id, "bookmarked_date")?;
        let bookmarked_date = DateTime::parse_from_rfc3339(&date_str)
            .map_err(|e| BookmarkError::InvalidUrl(format!("Failed to parse date: {}", e)))?
            .with_timezone(&Utc);

        // Extract optional fields
        let author = self.get_optional_string_field(obj_id, "author");
        
        let publish_date = self.get_optional_string_field(obj_id, "publish_date")
            .and_then(|date_str| DateTime::parse_from_rfc3339(&date_str).ok())
            .map(|dt| dt.with_timezone(&Utc));

        let reading_status = self.get_optional_string_field(obj_id, "reading_status")
            .and_then(|status_str| match status_str.as_str() {
                "Unread" => Some(crate::types::ReadingStatus::Unread),
                "Reading" => Some(crate::types::ReadingStatus::Reading),
                "Completed" => Some(crate::types::ReadingStatus::Completed),
                _ => None,
            })
            .unwrap_or(crate::types::ReadingStatus::Unread);

        let priority_rating = self.get_optional_string_field(obj_id, "priority_rating")
            .and_then(|priority_str| priority_str.parse::<u8>().ok());

        // Extract tags from list
        let tags = self.get_tags_from_list(obj_id)?;
        
        // Extract notes from list
        let notes = self.get_notes_from_list(obj_id)?;

        Ok(Bookmark {
            id,
            url,
            title,
            bookmarked_date,
            author,
            tags,
            publish_date,
            notes,
            reading_status,
            priority_rating,
        })
    }

    fn get_string_field(&self, obj_id: &automerge::ObjId, field: &str) -> BookmarkResult<String> {
        self.doc.get(obj_id, field)
            .map_err(|e| BookmarkError::InvalidUrl(format!("Failed to get {}: {}", field, e)))?
            .and_then(|(value, _)| value.to_str().map(|s| s.to_string()))
            .ok_or_else(|| BookmarkError::InvalidUrl(format!("Bookmark missing {}", field)))
    }

    fn get_optional_string_field(&self, obj_id: &automerge::ObjId, field: &str) -> Option<String> {
        self.doc.get(obj_id, field)
            .ok()?
            .and_then(|(value, _)| value.to_str().map(|s| s.to_string()))
    }

    fn get_tags_from_list(&self, obj_id: &automerge::ObjId) -> BookmarkResult<Vec<String>> {
        let tags_list = match self.doc.get(obj_id, "tags")
            .map_err(|e| BookmarkError::InvalidUrl(format!("Failed to get tags: {}", e)))? {
            Some((_, list_id)) => list_id,
            None => return Ok(Vec::new()),
        };

        let mut tags = Vec::new();
        let list_len = self.doc.length(&tags_list);
        
        for i in 0..list_len {
            if let Ok(Some((value, _))) = self.doc.get(&tags_list, i) {
                if let Some(tag) = value.to_str() {
                    tags.push(tag.to_string());
                }
            }
        }

        Ok(tags)
    }

    fn get_notes_from_list(&self, obj_id: &automerge::ObjId) -> BookmarkResult<Vec<crate::types::Note>> {
        let notes_list = match self.doc.get(obj_id, "notes")
            .map_err(|e| BookmarkError::InvalidUrl(format!("Failed to get notes: {}", e)))? {
            Some((_, list_id)) => list_id,
            None => return Ok(Vec::new()),
        };

        let mut notes = Vec::new();
        let list_len = self.doc.length(&notes_list);
        
        for i in 0..list_len {
            if let Ok(Some((_, note_obj_id))) = self.doc.get(&notes_list, i) {
                if let Ok(note) = self.note_from_automerge(&note_obj_id) {
                    notes.push(note);
                }
            }
        }

        Ok(notes)
    }

    fn note_from_automerge(&self, obj_id: &automerge::ObjId) -> BookmarkResult<crate::types::Note> {
        let id = self.get_string_field(obj_id, "id")?;
        let content = self.get_string_field(obj_id, "content")?;
        let created_at_str = self.get_string_field(obj_id, "created_at")?;
        
        let created_at = DateTime::parse_from_rfc3339(&created_at_str)
            .map_err(|e| BookmarkError::InvalidUrl(format!("Failed to parse note date: {}", e)))?
            .with_timezone(&Utc);

        Ok(crate::types::Note {
            id,
            content,
            created_at,
        })
    }

    fn add_bookmark_to_automerge(&mut self, bookmark: &Bookmark) -> BookmarkResult<()> {
        // Create a new bookmark object in the map using the bookmark ID as key
        let bookmark_obj = self.doc.put_object(&self.bookmarks_map, &bookmark.id, ObjType::Map)
            .map_err(|e| BookmarkError::InvalidUrl(format!("Failed to create bookmark object: {}", e)))?;

        // Set basic bookmark properties
        self.doc.put(&bookmark_obj, "id", bookmark.id.clone())
            .map_err(|e| BookmarkError::InvalidUrl(format!("Failed to set bookmark id: {}", e)))?;
        
        self.doc.put(&bookmark_obj, "url", bookmark.url.clone())
            .map_err(|e| BookmarkError::InvalidUrl(format!("Failed to set bookmark url: {}", e)))?;
        
        self.doc.put(&bookmark_obj, "title", bookmark.title.clone())
            .map_err(|e| BookmarkError::InvalidUrl(format!("Failed to set bookmark title: {}", e)))?;
        
        self.doc.put(&bookmark_obj, "bookmarked_date", bookmark.bookmarked_date.to_rfc3339())
            .map_err(|e| BookmarkError::InvalidUrl(format!("Failed to set bookmark date: {}", e)))?;

        // Set optional fields
        if let Some(ref author) = bookmark.author {
            self.doc.put(&bookmark_obj, "author", author.clone())
                .map_err(|e| BookmarkError::InvalidUrl(format!("Failed to set author: {}", e)))?;
        }

        if let Some(ref publish_date) = bookmark.publish_date {
            self.doc.put(&bookmark_obj, "publish_date", publish_date.to_rfc3339())
                .map_err(|e| BookmarkError::InvalidUrl(format!("Failed to set publish_date: {}", e)))?;
        }

        // Set reading status
        let status_str = match bookmark.reading_status {
            crate::types::ReadingStatus::Unread => "Unread",
            crate::types::ReadingStatus::Reading => "Reading",
            crate::types::ReadingStatus::Completed => "Completed",
        };
        self.doc.put(&bookmark_obj, "reading_status", status_str)
            .map_err(|e| BookmarkError::InvalidUrl(format!("Failed to set reading_status: {}", e)))?;

        // Set priority rating
        if let Some(priority) = bookmark.priority_rating {
            self.doc.put(&bookmark_obj, "priority_rating", priority.to_string())
                .map_err(|e| BookmarkError::InvalidUrl(format!("Failed to set priority_rating: {}", e)))?;
        }

        // Add tags as a list
        if !bookmark.tags.is_empty() {
            let tags_list = self.doc.put_object(&bookmark_obj, "tags", ObjType::List)
                .map_err(|e| BookmarkError::InvalidUrl(format!("Failed to create tags list: {}", e)))?;
            
            for tag in &bookmark.tags {
                self.doc.insert(&tags_list, self.doc.length(&tags_list), tag.clone())
                    .map_err(|e| BookmarkError::InvalidUrl(format!("Failed to add tag: {}", e)))?;
            }
        }

        // Add notes as a list
        if !bookmark.notes.is_empty() {
            let notes_list = self.doc.put_object(&bookmark_obj, "notes", ObjType::List)
                .map_err(|e| BookmarkError::InvalidUrl(format!("Failed to create notes list: {}", e)))?;
            
            for note in &bookmark.notes {
                self.add_note_to_list(&notes_list, note)?;
            }
        }

        Ok(())
    }

    fn add_note_to_list(&mut self, notes_list: &automerge::ObjId, note: &crate::types::Note) -> BookmarkResult<()> {
        let note_obj = self.doc.insert_object(notes_list, self.doc.length(notes_list), ObjType::Map)
            .map_err(|e| BookmarkError::InvalidUrl(format!("Failed to create note object: {}", e)))?;

        self.doc.put(&note_obj, "id", note.id.clone())
            .map_err(|e| BookmarkError::InvalidUrl(format!("Failed to set note id: {}", e)))?;
        
        self.doc.put(&note_obj, "content", note.content.clone())
            .map_err(|e| BookmarkError::InvalidUrl(format!("Failed to set note content: {}", e)))?;
        
        self.doc.put(&note_obj, "created_at", note.created_at.to_rfc3339())
            .map_err(|e| BookmarkError::InvalidUrl(format!("Failed to set note created_at: {}", e)))?;

        Ok(())
    }

    fn bookmark_exists(&self, id: &str) -> bool {
        match self.doc.get(&self.bookmarks_map, id) {
            Ok(Some(_)) => true,
            _ => false,
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

#[async_trait]
impl BookmarkRepository for AutomergeBookmarkRepository {
    async fn create(&mut self, bookmark: Bookmark) -> BookmarkResult<Bookmark> {
        self.add_bookmark_to_automerge(&bookmark)?;
        self.save()?;
        Ok(bookmark)
    }

    async fn find_all(&self, filters: Option<BookmarkFilters>) -> BookmarkResult<Vec<Bookmark>> {
        let mut bookmarks = Vec::new();
        
        // Iterate through all bookmarks in the map
        let keys: Vec<String> = self.doc.keys(&self.bookmarks_map).collect();
        for bookmark_id in keys {
            if let Ok(Some((_, obj_id))) = self.doc.get(&self.bookmarks_map, &bookmark_id) {
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
        match self.doc.get(&self.bookmarks_map, id)
            .map_err(|e| BookmarkError::InvalidUrl(format!("Failed to get bookmark: {}", e)))? {
            Some((_, obj_id)) => self.bookmark_from_automerge(&obj_id),
            None => Err(BookmarkError::NotFound(id.to_string())),
        }
    }
    
    async fn update(&mut self, bookmark: Bookmark) -> BookmarkResult<Bookmark> {
        // Check if bookmark exists
        let obj_id = match self.doc.get(&self.bookmarks_map, &bookmark.id)
            .map_err(|e| BookmarkError::InvalidUrl(format!("Failed to get bookmark for update: {}", e)))? {
            Some((_, obj_id)) => obj_id,
            None => return Err(BookmarkError::NotFound(bookmark.id.clone())),
        };

        // Update fields with CRDT field-level semantics
        self.update_bookmark_fields(&obj_id, &bookmark)?;
        self.save()?;
        
        Ok(bookmark)
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
        // Get bookmark object directly from map
        let obj_id = match self.doc.get(&self.bookmarks_map, bookmark_id)
            .map_err(|e| BookmarkError::InvalidUrl(format!("Failed to get bookmark for note: {}", e)))? {
            Some((_, obj_id)) => obj_id,
            None => return Err(BookmarkError::NotFound(bookmark_id.to_string())),
        };

        // Create new note
        let note = crate::types::Note::new(content);
        let note_id = note.id.clone();

        // Get or create notes list
        let notes_list = match self.doc.get(&obj_id, "notes")
            .map_err(|e| BookmarkError::InvalidUrl(format!("Failed to get notes list: {}", e)))? {
            Some((_, list_id)) => list_id,
            None => {
                self.doc.put_object(&obj_id, "notes", ObjType::List)
                    .map_err(|e| BookmarkError::InvalidUrl(format!("Failed to create notes list: {}", e)))?
            }
        };

        // Add note to list with CRDT append semantics
        self.add_note_to_list(&notes_list, &note)?;
        self.save()?;
        
        Ok(note_id)
    }
    
    async fn remove_note(&mut self, bookmark_id: &str, note_id: &str) -> BookmarkResult<()> {
        // Get bookmark object directly from map
        let obj_id = match self.doc.get(&self.bookmarks_map, bookmark_id)
            .map_err(|e| BookmarkError::InvalidUrl(format!("Failed to get bookmark for note removal: {}", e)))? {
            Some((_, obj_id)) => obj_id,
            None => return Err(BookmarkError::NotFound(bookmark_id.to_string())),
        };

        // Get notes list
        let notes_list = match self.doc.get(&obj_id, "notes")
            .map_err(|e| BookmarkError::InvalidUrl(format!("Failed to get notes list for removal: {}", e)))? {
            Some((_, list_id)) => list_id,
            None => return Err(BookmarkError::NotFound(format!("Note {} not found", note_id))),
        };

        // Find and remove the note
        let list_len = self.doc.length(&notes_list);
        for i in 0..list_len {
            if let Ok(Some((_, note_obj))) = self.doc.get(&notes_list, i) {
                if let Ok(Some((value, _))) = self.doc.get(&note_obj, "id") {
                    if let Some(stored_note_id) = value.to_str() {
                        if stored_note_id == note_id {
                            // Remove note from list with CRDT delete semantics
                            self.doc.delete(&notes_list, i)
                                .map_err(|e| BookmarkError::InvalidUrl(format!("Failed to remove note: {}", e)))?;
                            self.save()?;
                            return Ok(());
                        }
                    }
                }
            }
        }

        Err(BookmarkError::NotFound(format!("Note {} not found", note_id)))
    }

    async fn delete(&mut self, id: &str) -> BookmarkResult<()> {
        // Check if bookmark exists first
        if !self.bookmark_exists(id) {
            return Err(BookmarkError::NotFound(id.to_string()));
        }
            
        self.doc.delete(&self.bookmarks_map, id)
            .map_err(|e| BookmarkError::InvalidUrl(format!("Failed to delete bookmark: {}", e)))?;
            
        self.save()?;
        Ok(())
    }
}

// Additional helper methods for CRDT operations
impl AutomergeBookmarkRepository {
    fn update_bookmark_fields(&mut self, obj_id: &automerge::ObjId, bookmark: &Bookmark) -> BookmarkResult<()> {
        // Update basic fields (last-writer-wins semantics)
        self.doc.put(obj_id, "url", bookmark.url.clone())
            .map_err(|e| BookmarkError::InvalidUrl(format!("Failed to update url: {}", e)))?;
        
        self.doc.put(obj_id, "title", bookmark.title.clone())
            .map_err(|e| BookmarkError::InvalidUrl(format!("Failed to update title: {}", e)))?;
        
        self.doc.put(obj_id, "bookmarked_date", bookmark.bookmarked_date.to_rfc3339())
            .map_err(|e| BookmarkError::InvalidUrl(format!("Failed to update date: {}", e)))?;

        // Update optional fields
        if let Some(ref author) = bookmark.author {
            self.doc.put(obj_id, "author", author.clone())
                .map_err(|e| BookmarkError::InvalidUrl(format!("Failed to update author: {}", e)))?;
        } else {
            // Remove author field if None
            let _ = self.doc.delete(obj_id, "author");
        }

        if let Some(ref publish_date) = bookmark.publish_date {
            self.doc.put(obj_id, "publish_date", publish_date.to_rfc3339())
                .map_err(|e| BookmarkError::InvalidUrl(format!("Failed to update publish_date: {}", e)))?;
        } else {
            let _ = self.doc.delete(obj_id, "publish_date");
        }

        // Update reading status
        let status_str = match bookmark.reading_status {
            crate::types::ReadingStatus::Unread => "Unread",
            crate::types::ReadingStatus::Reading => "Reading",
            crate::types::ReadingStatus::Completed => "Completed",
        };
        self.doc.put(obj_id, "reading_status", status_str)
            .map_err(|e| BookmarkError::InvalidUrl(format!("Failed to update reading_status: {}", e)))?;

        // Update priority rating
        if let Some(priority) = bookmark.priority_rating {
            self.doc.put(obj_id, "priority_rating", priority.to_string())
                .map_err(|e| BookmarkError::InvalidUrl(format!("Failed to update priority_rating: {}", e)))?;
        } else {
            let _ = self.doc.delete(obj_id, "priority_rating");
        }

        // Update tags with set union semantics
        self.update_tags_list(obj_id, &bookmark.tags)?;
        
        // Update notes with sequence semantics 
        self.update_notes_list(obj_id, &bookmark.notes)?;

        Ok(())
    }

    fn update_tags_list(&mut self, obj_id: &automerge::ObjId, new_tags: &[String]) -> BookmarkResult<()> {
        // Clear existing tags and recreate (simple approach for now)
        // TODO: Implement proper CRDT set union semantics
        let _ = self.doc.delete(obj_id, "tags");
        
        if !new_tags.is_empty() {
            let tags_list = self.doc.put_object(obj_id, "tags", ObjType::List)
                .map_err(|e| BookmarkError::InvalidUrl(format!("Failed to create tags list: {}", e)))?;
            
            for tag in new_tags {
                self.doc.insert(&tags_list, self.doc.length(&tags_list), tag.clone())
                    .map_err(|e| BookmarkError::InvalidUrl(format!("Failed to add tag: {}", e)))?;
            }
        }

        Ok(())
    }

    fn update_notes_list(&mut self, obj_id: &automerge::ObjId, new_notes: &[crate::types::Note]) -> BookmarkResult<()> {
        // Clear existing notes and recreate (simple approach for now)
        // TODO: Implement proper CRDT sequence semantics with conflict resolution
        let _ = self.doc.delete(obj_id, "notes");
        
        if !new_notes.is_empty() {
            let notes_list = self.doc.put_object(obj_id, "notes", ObjType::List)
                .map_err(|e| BookmarkError::InvalidUrl(format!("Failed to create notes list: {}", e)))?;
            
            for note in new_notes {
                self.add_note_to_list(&notes_list, note)?;
            }
        }

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

    #[tokio::test]
    async fn test_enhanced_bookmark_with_all_fields() {
        let (mut repo, _temp_dir) = create_test_repo();
        
        // Create bookmark with all fields
        let mut bookmark = Bookmark::new("https://example.com", "Example Site").unwrap()
            .with_tags(vec!["rust".to_string(), "programming".to_string()])
            .with_priority(4).unwrap();
        
        bookmark.author = Some("Test Author".to_string());
        bookmark.reading_status = crate::types::ReadingStatus::Reading;
        
        // Add some notes
        let note_id1 = bookmark.add_note("First note");
        let note_id2 = bookmark.add_note("Second note");
        
        let bookmark_id = bookmark.id.clone();
        
        // Save bookmark
        repo.create(bookmark.clone()).await.unwrap();
        
        // Retrieve and verify all fields
        let retrieved = repo.find_by_id(&bookmark_id).await.unwrap();
        
        assert_eq!(retrieved.url, "https://example.com");
        assert_eq!(retrieved.title, "Example Site");
        assert_eq!(retrieved.author, Some("Test Author".to_string()));
        assert_eq!(retrieved.reading_status, crate::types::ReadingStatus::Reading);
        assert_eq!(retrieved.priority_rating, Some(4));
        assert_eq!(retrieved.tags, vec!["rust", "programming"]);
        assert_eq!(retrieved.notes.len(), 2);
        assert_eq!(retrieved.notes[0].content, "First note");
        assert_eq!(retrieved.notes[1].content, "Second note");
        assert_eq!(retrieved.notes[0].id, note_id1);
        assert_eq!(retrieved.notes[1].id, note_id2);
    }

    #[tokio::test]
    async fn test_search_by_text_functionality() {
        let (mut repo, _temp_dir) = create_test_repo();
        
        // Create bookmarks with different content
        let bookmark1 = Bookmark::new("https://rust-lang.org", "Rust Programming Language").unwrap()
            .with_tags(vec!["rust".to_string(), "programming".to_string()]);
        
        let mut bookmark2 = Bookmark::new("https://python.org", "Python Programming").unwrap()
            .with_tags(vec!["python".to_string(), "programming".to_string()]);
        bookmark2.author = Some("Python Foundation".to_string());
        bookmark2.add_note("Great for data science");
        
        let bookmark3 = Bookmark::new("https://golang.org", "Go Language").unwrap();
        
        repo.create(bookmark1).await.unwrap();
        repo.create(bookmark2).await.unwrap();
        repo.create(bookmark3).await.unwrap();
        
        // Test search by title
        let results = repo.search_by_text("rust").await.unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].title, "Rust Programming Language");
        
        // Test search by URL
        let results = repo.search_by_text("python.org").await.unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].title, "Python Programming");
        
        // Test search by author
        let results = repo.search_by_text("Foundation").await.unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].author, Some("Python Foundation".to_string()));
        
        // Test search by note content
        let results = repo.search_by_text("data science").await.unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].title, "Python Programming");
        
        // Test search for common word
        let results = repo.search_by_text("programming").await.unwrap();
        assert_eq!(results.len(), 2);
    }

    #[tokio::test]
    async fn test_find_by_tags_functionality() {
        let (mut repo, _temp_dir) = create_test_repo();
        
        let bookmark1 = Bookmark::new("https://example1.com", "Rust Web").unwrap()
            .with_tags(vec!["rust".to_string(), "web".to_string(), "programming".to_string()]);
        
        let bookmark2 = Bookmark::new("https://example2.com", "Rust CLI").unwrap()
            .with_tags(vec!["rust".to_string(), "cli".to_string(), "programming".to_string()]);
        
        let bookmark3 = Bookmark::new("https://example3.com", "Python Web").unwrap()
            .with_tags(vec!["python".to_string(), "web".to_string(), "programming".to_string()]);
        
        repo.create(bookmark1).await.unwrap();
        repo.create(bookmark2).await.unwrap();
        repo.create(bookmark3).await.unwrap();
        
        // Test single tag
        let results = repo.find_by_tags(&["rust".to_string()]).await.unwrap();
        assert_eq!(results.len(), 2);
        
        // Test multiple tags (AND logic)
        let results = repo.find_by_tags(&["rust".to_string(), "web".to_string()]).await.unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].title, "Rust Web");
        
        // Test common tag
        let results = repo.find_by_tags(&["programming".to_string()]).await.unwrap();
        assert_eq!(results.len(), 3);
        
        // Test non-existent tag
        let results = repo.find_by_tags(&["nonexistent".to_string()]).await.unwrap();
        assert_eq!(results.len(), 0);
    }

    #[tokio::test]
    async fn test_add_remove_notes_crdt_semantics() {
        let (mut repo, _temp_dir) = create_test_repo();
        
        let bookmark = Bookmark::new("https://example.com", "Test Bookmark").unwrap();
        let bookmark_id = bookmark.id.clone();
        
        repo.create(bookmark).await.unwrap();
        
        // Add notes using repository method
        let note_id1 = repo.add_note(&bookmark_id, "First note").await.unwrap();
        let note_id2 = repo.add_note(&bookmark_id, "Second note").await.unwrap();
        
        // Verify notes were added
        let retrieved = repo.find_by_id(&bookmark_id).await.unwrap();
        assert_eq!(retrieved.notes.len(), 2);
        assert_eq!(retrieved.notes[0].content, "First note");
        assert_eq!(retrieved.notes[1].content, "Second note");
        
        // Remove first note
        repo.remove_note(&bookmark_id, &note_id1).await.unwrap();
        
        // Verify note was removed
        let retrieved = repo.find_by_id(&bookmark_id).await.unwrap();
        assert_eq!(retrieved.notes.len(), 1);
        assert_eq!(retrieved.notes[0].id, note_id2);
        assert_eq!(retrieved.notes[0].content, "Second note");
        
        // Try to remove non-existent note
        let result = repo.remove_note(&bookmark_id, "nonexistent").await;
        assert!(matches!(result, Err(BookmarkError::NotFound(_))));
    }

    #[tokio::test]
    async fn test_field_level_updates() {
        let (mut repo, _temp_dir) = create_test_repo();
        
        // Create initial bookmark
        let mut bookmark = Bookmark::new("https://example.com", "Original Title").unwrap();
        let bookmark_id = bookmark.id.clone();
        
        repo.create(bookmark.clone()).await.unwrap();
        
        // Update fields
        bookmark.title = "Updated Title".to_string();
        bookmark.author = Some("New Author".to_string());
        bookmark.reading_status = crate::types::ReadingStatus::Completed;
        bookmark.priority_rating = Some(5);
        bookmark.tags = vec!["updated".to_string(), "test".to_string()];
        
        // Perform field-level update
        repo.update(bookmark.clone()).await.unwrap();
        
        // Verify all fields were updated
        let retrieved = repo.find_by_id(&bookmark_id).await.unwrap();
        assert_eq!(retrieved.title, "Updated Title");
        assert_eq!(retrieved.author, Some("New Author".to_string()));
        assert_eq!(retrieved.reading_status, crate::types::ReadingStatus::Completed);
        assert_eq!(retrieved.priority_rating, Some(5));
        assert_eq!(retrieved.tags, vec!["updated", "test"]);
        
        // URL and ID should remain unchanged
        assert_eq!(retrieved.url, "https://example.com");
        assert_eq!(retrieved.id, bookmark_id);
    }
}