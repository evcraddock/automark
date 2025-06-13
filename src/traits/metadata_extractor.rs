use async_trait::async_trait;
use std::time::Duration;
use crate::types::{ExtractedMetadata, ExtractorError};

#[async_trait]
pub trait MetadataExtractor: Send + Sync {
    async fn extract_metadata(&self, url: &str, timeout: Duration) -> Result<ExtractedMetadata, ExtractorError>;
}