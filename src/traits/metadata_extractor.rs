use async_trait::async_trait;
use std::time::Duration;
use crate::types::{ExtractedMetadata, ExtractorError};

#[async_trait]
pub trait MetadataExtractor: Send + Sync {
    async fn extract_metadata(&self, url: &str, timeout: Duration) -> Result<ExtractedMetadata, ExtractorError>;
}

#[cfg(test)]
pub struct MockMetadataExtractor {
    pub should_fail: bool,
    pub extracted_title: Option<String>,
    pub extracted_author: Option<String>,
}

#[cfg(test)]
impl MockMetadataExtractor {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self {
            should_fail: false,
            extracted_title: Some("Extracted Title".to_string()),
            extracted_author: Some("Extracted Author".to_string()),
        }
    }

    pub fn with_failure() -> Self {
        Self {
            should_fail: true,
            extracted_title: None,
            extracted_author: None,
        }
    }

    pub fn with_title(title: &str) -> Self {
        Self {
            should_fail: false,
            extracted_title: Some(title.to_string()),
            extracted_author: None,
        }
    }

    pub fn with_metadata(title: Option<String>, author: Option<String>, _publish_date: Option<chrono::DateTime<chrono::Utc>>) -> Self {
        Self {
            should_fail: false,
            extracted_title: title,
            extracted_author: author,
        }
    }
}

#[cfg(test)]
#[async_trait]
impl MetadataExtractor for MockMetadataExtractor {
    async fn extract_metadata(&self, _url: &str, _timeout: Duration) -> Result<ExtractedMetadata, ExtractorError> {
        if self.should_fail {
            return Err(ExtractorError::NetworkError("Mock network error".to_string()));
        }

        Ok(ExtractedMetadata {
            title: self.extracted_title.clone(),
            author: self.extracted_author.clone(),
            publish_date: None,
        })
    }
}