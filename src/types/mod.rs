pub mod bookmark;

pub use bookmark::{Bookmark, Note, ReadingStatus, SortOrder, BookmarkFilters};

use thiserror::Error;

#[derive(Debug, Error)]
pub enum BookmarkError {
    #[error("Invalid URL format: {0}")]
    InvalidUrl(String),
    #[error("Bookmark not found: {0}")]
    NotFound(String),
    #[error("Title cannot be empty")]
    EmptyTitle,
    #[error("Invalid or ambiguous ID: {0}")]
    InvalidId(String),
}

pub type BookmarkResult<T> = Result<T, BookmarkError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display_messages() {
        let invalid_url_error = BookmarkError::InvalidUrl("bad-url".to_string());
        assert_eq!(invalid_url_error.to_string(), "Invalid URL format: bad-url");

        let not_found_error = BookmarkError::NotFound("123".to_string());
        assert_eq!(not_found_error.to_string(), "Bookmark not found: 123");

        let empty_title_error = BookmarkError::EmptyTitle;
        assert_eq!(empty_title_error.to_string(), "Title cannot be empty");

        let invalid_id_error = BookmarkError::InvalidId("ambiguous".to_string());
        assert_eq!(invalid_id_error.to_string(), "Invalid or ambiguous ID: ambiguous");
    }

    #[test]
    fn test_error_types() {
        let invalid_url = BookmarkError::InvalidUrl("test".to_string());
        assert!(matches!(invalid_url, BookmarkError::InvalidUrl(_)));

        let not_found = BookmarkError::NotFound("test".to_string());
        assert!(matches!(not_found, BookmarkError::NotFound(_)));

        let empty_title = BookmarkError::EmptyTitle;
        assert!(matches!(empty_title, BookmarkError::EmptyTitle));

        let invalid_id = BookmarkError::InvalidId("test".to_string());
        assert!(matches!(invalid_id, BookmarkError::InvalidId(_)));
    }
}