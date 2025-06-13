pub mod repository;
pub mod metadata_extractor;

pub use metadata_extractor::MetadataExtractor;

#[cfg(test)]
pub use metadata_extractor::MockMetadataExtractor;

pub use repository::BookmarkRepository;