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
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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