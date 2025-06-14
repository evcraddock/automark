use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use url::Url;

use super::{BookmarkError, BookmarkResult};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Bookmark {
    pub id: String,
    pub url: String,
    pub title: String,
    pub bookmarked_date: DateTime<Utc>,
    pub author: Option<String>,
    pub tags: Vec<String>,
    pub publish_date: Option<DateTime<Utc>>,
    pub notes: Vec<Note>,
    pub reading_status: ReadingStatus,
    pub priority_rating: Option<u8>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Note {
    pub id: String,
    pub content: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, clap::ValueEnum)]
pub enum ReadingStatus {
    Unread,
    Reading,
    Completed,
}


#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BookmarkFilters {
    pub text_query: Option<String>,
    pub tags: Option<Vec<String>>,
    pub reading_status: Option<ReadingStatus>,
    pub priority_range: Option<(u8, u8)>,
    pub bookmarked_since: Option<DateTime<Utc>>,
    pub bookmarked_until: Option<DateTime<Utc>>,
    pub published_since: Option<DateTime<Utc>>,
    pub published_until: Option<DateTime<Utc>>,
    pub sort_by: Option<SortBy>,
    pub sort_order: Option<SortDirection>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, clap::ValueEnum)]
pub enum SortBy {
    BookmarkedDate,
    PublishDate,
    Title,
    Priority,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, clap::ValueEnum)]
pub enum SortDirection {
    Ascending,
    Descending,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExtractedMetadata {
    pub title: Option<String>,
    pub author: Option<String>,
    pub publish_date: Option<DateTime<Utc>>,
}

impl Bookmark {
    pub fn new(url: &str, title: &str) -> BookmarkResult<Self> {
        // Validate title is not empty
        if title.trim().is_empty() {
            return Err(BookmarkError::EmptyTitle);
        }

        // Validate URL format
        Url::parse(url).map_err(|_| BookmarkError::InvalidUrl(url.to_string()))?;

        Ok(Bookmark {
            id: Uuid::new_v4().to_string(),
            url: url.to_string(),
            title: title.trim().to_string(),
            bookmarked_date: Utc::now(),
            author: None,
            tags: Vec::new(),
            publish_date: None,
            notes: Vec::new(),
            reading_status: ReadingStatus::Unread,
            priority_rating: None,
        })
    }

    pub fn with_tags(mut self, tags: Vec<String>) -> Self {
        self.tags = tags.into_iter().map(|tag| tag.to_lowercase()).collect();
        self
    }

    pub fn with_priority(mut self, priority: u8) -> BookmarkResult<Self> {
        if !(1..=5).contains(&priority) {
            return Err(BookmarkError::InvalidId(format!("Priority must be between 1 and 5, got {}", priority)));
        }
        self.priority_rating = Some(priority);
        Ok(self)
    }

    pub fn add_note(&mut self, content: &str) -> String {
        let note = Note::new(content);
        let note_id = note.id.clone();
        self.notes.push(note);
        note_id
    }

    pub fn remove_note(&mut self, note_id: &str) -> bool {
        if let Some(pos) = self.notes.iter().position(|n| n.id == note_id) {
            self.notes.remove(pos);
            true
        } else {
            false
        }
    }
}

impl Note {
    pub fn new(content: &str) -> Self {
        Note {
            id: Uuid::new_v4().to_string(),
            content: content.to_string(),
            created_at: Utc::now(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_enhanced_bookmark_creation() {
        let bookmark = Bookmark::new("https://example.com", "Example Site").unwrap();
        
        assert_eq!(bookmark.url, "https://example.com");
        assert_eq!(bookmark.title, "Example Site");
        assert_eq!(bookmark.author, None);
        assert_eq!(bookmark.tags, Vec::<String>::new());
        assert_eq!(bookmark.publish_date, None);
        assert_eq!(bookmark.notes, Vec::<Note>::new());
        assert_eq!(bookmark.reading_status, ReadingStatus::Unread);
        assert_eq!(bookmark.priority_rating, None);
    }

    #[test]
    fn test_tag_normalization() {
        let bookmark = Bookmark::new("https://example.com", "Test")
            .unwrap()
            .with_tags(vec!["Rust".to_string(), "PROGRAMMING".to_string(), "Web".to_string()]);
        
        assert_eq!(bookmark.tags, vec!["rust", "programming", "web"]);
    }

    #[test]
    fn test_priority_validation() {
        let bookmark = Bookmark::new("https://example.com", "Test").unwrap();
        
        // Valid priorities
        assert!(bookmark.clone().with_priority(1).is_ok());
        assert!(bookmark.clone().with_priority(3).is_ok());
        assert!(bookmark.clone().with_priority(5).is_ok());
        
        // Invalid priorities
        assert!(bookmark.clone().with_priority(0).is_err());
        assert!(bookmark.clone().with_priority(6).is_err());
    }

    #[test]
    fn test_note_immutability() {
        let mut bookmark = Bookmark::new("https://example.com", "Test").unwrap();
        
        // Add notes
        let note_id1 = bookmark.add_note("First note");
        let _note_id2 = bookmark.add_note("Second note");
        
        assert_eq!(bookmark.notes.len(), 2);
        assert_eq!(bookmark.notes[0].content, "First note");
        assert_eq!(bookmark.notes[1].content, "Second note");
        
        // Remove a note
        assert!(bookmark.remove_note(&note_id1));
        assert_eq!(bookmark.notes.len(), 1);
        assert_eq!(bookmark.notes[0].content, "Second note");
        
        // Try to remove non-existent note
        assert!(!bookmark.remove_note("non-existent"));
    }

    #[test]
    fn test_note_creation() {
        let note = Note::new("Test content");
        
        assert_eq!(note.content, "Test content");
        assert!(!note.id.is_empty());
        assert!(note.created_at <= Utc::now());
    }

    #[test]
    fn test_reading_status_serialization() {
        let status = ReadingStatus::Reading;
        let json = serde_json::to_string(&status).unwrap();
        let deserialized: ReadingStatus = serde_json::from_str(&json).unwrap();
        
        assert_eq!(status, deserialized);
    }

    #[test]
    fn test_bookmark_filters_creation() {
        let filters = BookmarkFilters {
            text_query: Some("rust".to_string()),
            tags: Some(vec!["programming".to_string()]),
            reading_status: Some(ReadingStatus::Unread),
            priority_range: Some((3, 5)),
            bookmarked_since: None,
            bookmarked_until: None,
            published_since: None,
            published_until: None,
            sort_by: Some(SortBy::BookmarkedDate),
            sort_order: Some(SortDirection::Descending),
        };
        
        assert_eq!(filters.text_query, Some("rust".to_string()));
        assert_eq!(filters.tags, Some(vec!["programming".to_string()]));
        assert_eq!(filters.reading_status, Some(ReadingStatus::Unread));
        assert_eq!(filters.priority_range, Some((3, 5)));
        assert_eq!(filters.sort_by, Some(SortBy::BookmarkedDate));
        assert_eq!(filters.sort_order, Some(SortDirection::Descending));
    }

    #[test]
    fn test_extended_bookmark_filters() {
        let now = Utc::now();
        let one_day_ago = now - chrono::Duration::days(1);
        
        let filters = BookmarkFilters {
            text_query: None,
            tags: None,
            reading_status: None,
            priority_range: None,
            bookmarked_since: Some(one_day_ago),
            bookmarked_until: Some(now),
            published_since: Some(one_day_ago),
            published_until: Some(now),
            sort_by: Some(SortBy::Title),
            sort_order: Some(SortDirection::Ascending),
        };
        
        assert_eq!(filters.bookmarked_since, Some(one_day_ago));
        assert_eq!(filters.bookmarked_until, Some(now));
        assert_eq!(filters.published_since, Some(one_day_ago));
        assert_eq!(filters.published_until, Some(now));
        assert_eq!(filters.sort_by, Some(SortBy::Title));
        assert_eq!(filters.sort_order, Some(SortDirection::Ascending));
    }

    #[test]
    fn test_sort_enums() {
        // Test SortBy variants
        assert_eq!(SortBy::BookmarkedDate, SortBy::BookmarkedDate);
        assert_eq!(SortBy::PublishDate, SortBy::PublishDate);
        assert_eq!(SortBy::Title, SortBy::Title);
        assert_eq!(SortBy::Priority, SortBy::Priority);
        
        // Test SortDirection variants
        assert_eq!(SortDirection::Ascending, SortDirection::Ascending);
        assert_eq!(SortDirection::Descending, SortDirection::Descending);
    }

    #[test]
    fn test_sort_enums_serialization() {
        let sort_by = SortBy::Title;
        let json = serde_json::to_string(&sort_by).unwrap();
        let deserialized: SortBy = serde_json::from_str(&json).unwrap();
        assert_eq!(sort_by, deserialized);
        
        let sort_direction = SortDirection::Descending;
        let json = serde_json::to_string(&sort_direction).unwrap();
        let deserialized: SortDirection = serde_json::from_str(&json).unwrap();
        assert_eq!(sort_direction, deserialized);
    }

    #[test]
    fn test_enhanced_serialization() {
        let mut bookmark = Bookmark::new("https://example.com", "Example")
            .unwrap()
            .with_tags(vec!["rust".to_string()])
            .with_priority(4).unwrap();
        
        bookmark.add_note("Test note");
        
        // Test serialization
        let json = serde_json::to_string(&bookmark).unwrap();
        assert!(json.contains("rust"));
        assert!(json.contains("Test note"));
        assert!(json.contains("Unread"));
        
        // Test deserialized
        let deserialized: Bookmark = serde_json::from_str(&json).unwrap();
        assert_eq!(bookmark, deserialized);
    }

    #[test]
    fn test_extracted_metadata_creation() {
        use super::ExtractedMetadata;
        
        let metadata = ExtractedMetadata {
            title: Some("Test Title".to_string()),
            author: Some("Test Author".to_string()),
            publish_date: Some(Utc::now()),
        };
        
        assert_eq!(metadata.title, Some("Test Title".to_string()));
        assert_eq!(metadata.author, Some("Test Author".to_string()));
        assert!(metadata.publish_date.is_some());
    }

    #[test]
    fn test_extracted_metadata_serialization() {
        use super::ExtractedMetadata;
        
        let metadata = ExtractedMetadata {
            title: Some("Test Title".to_string()),
            author: None,
            publish_date: None,
        };
        
        let json = serde_json::to_string(&metadata).unwrap();
        let deserialized: ExtractedMetadata = serde_json::from_str(&json).unwrap();
        
        assert_eq!(metadata, deserialized);
    }

    #[test]
    fn test_create_valid_bookmark() {
        let bookmark = Bookmark::new("https://example.com", "Example Site").unwrap();
        
        assert_eq!(bookmark.url, "https://example.com");
        assert_eq!(bookmark.title, "Example Site");
        assert!(!bookmark.id.is_empty());
        assert!(bookmark.bookmarked_date <= Utc::now());
    }

    #[test]
    fn test_reject_invalid_url() {
        let result = Bookmark::new("not-a-url", "Title");
        
        assert!(matches!(result, Err(BookmarkError::InvalidUrl(_))));
    }

    #[test]
    fn test_reject_empty_title() {
        let result = Bookmark::new("https://example.com", "");
        
        assert!(matches!(result, Err(BookmarkError::EmptyTitle)));
    }

    #[test]
    fn test_reject_whitespace_only_title() {
        let result = Bookmark::new("https://example.com", "   ");
        
        assert!(matches!(result, Err(BookmarkError::EmptyTitle)));
    }

    #[test]
    fn test_trim_title_whitespace() {
        let bookmark = Bookmark::new("https://example.com", "  Title  ").unwrap();
        
        assert_eq!(bookmark.title, "Title");
    }

    #[test]
    fn test_id_uniqueness() {
        let bookmark1 = Bookmark::new("https://example.com", "Title 1").unwrap();
        let bookmark2 = Bookmark::new("https://example.com", "Title 2").unwrap();
        
        assert_ne!(bookmark1.id, bookmark2.id);
    }

    #[test]
    fn test_serialization() {
        let bookmark = Bookmark::new("https://example.com", "Example").unwrap();
        
        // Test serialization
        let json = serde_json::to_string(&bookmark).unwrap();
        assert!(json.contains("https://example.com"));
        assert!(json.contains("Example"));
        
        // Test deserialization
        let deserialized: Bookmark = serde_json::from_str(&json).unwrap();
        assert_eq!(bookmark, deserialized);
    }

    #[test]
    fn test_various_valid_urls() {
        let test_cases = vec![
            "https://www.example.com",
            "http://example.com",
            "https://subdomain.example.com/path",
            "https://example.com:8080/path?query=value",
        ];

        for url in test_cases {
            let result = Bookmark::new(url, "Test Title");
            assert!(result.is_ok(), "Failed to create bookmark for URL: {}", url);
        }
    }
}