pub mod bookmark;
pub mod config;

pub use bookmark::{Bookmark, Note, ReadingStatus, BookmarkFilters, ExtractedMetadata, SortBy, SortDirection};
pub use config::{Config, ConfigError, ConfigResult};

use thiserror::Error;

#[derive(Debug, Error)]
pub enum ExtractorError {
    #[error("Network request failed: {0}")]
    NetworkError(String),
    #[error("Request timed out")]
    Timeout,
    #[error("Invalid URL: {0}")]
    InvalidUrl(String),
}

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
    #[error("Metadata extraction failed: {0}")]
    MetadataExtraction(#[from] ExtractorError),
    #[error("Sync failed: {0}")]
    SyncError(String),
    #[error("Terminal I/O error: {0}")]
    TerminalError(#[from] std::io::Error),
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

    #[test]
    fn test_extractor_error_types() {
        let network_error = ExtractorError::NetworkError("connection failed".to_string());
        assert!(matches!(network_error, ExtractorError::NetworkError(_)));
        assert_eq!(network_error.to_string(), "Network request failed: connection failed");

        let timeout_error = ExtractorError::Timeout;
        assert!(matches!(timeout_error, ExtractorError::Timeout));
        assert_eq!(timeout_error.to_string(), "Request timed out");

        let invalid_url_error = ExtractorError::InvalidUrl("bad-url".to_string());
        assert!(matches!(invalid_url_error, ExtractorError::InvalidUrl(_)));
        assert_eq!(invalid_url_error.to_string(), "Invalid URL: bad-url");
    }

    #[test]
    fn test_bookmark_error_from_extractor_error() {
        let extractor_error = ExtractorError::NetworkError("test".to_string());
        let bookmark_error: BookmarkError = extractor_error.into();
        assert!(matches!(bookmark_error, BookmarkError::MetadataExtraction(_)));
    }
}