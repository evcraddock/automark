# Task 2: Create MetadataExtractor Trait and WebExtractor Implementation

**GitHub Issue**: [#2](https://github.com/evcraddock/automark/issues/2)

## Objective
Implement metadata extraction capability for automatically populating bookmark fields from web pages.

## Requirements

1. **Create MetadataExtractor trait** in `src/traits/metadata_extractor.rs`:
   - Single async method: extract_metadata(url: &str) -> Result<ExtractedMetadata, ExtractorError>
   - Include timeout parameter in trait design
   - Define for HTTP-based metadata extraction

2. **Create ExtractedMetadata struct** in `src/types/bookmark.rs`:
   - Fields: title (Option<String>), author (Option<String>), publish_date (Option<DateTime<Utc>>)
   - All fields optional as websites may not provide complete metadata
   - Implement serde derives for JSON serialization

3. **Create ExtractorError enum** in `src/types/mod.rs`:
   - Error variants: NetworkError, ParseError, Timeout, InvalidUrl
   - Implement proper error messages and Display trait
   - Add to BookmarkError enum as MetadataExtraction(ExtractorError)

4. **Implement WebExtractor** in `src/adapters/web_extractor.rs`:
   - Use reqwest for HTTP requests
   - Use scraper for HTML parsing
   - Extract title from `<title>` tag
   - Extract author from meta tags (author, article:author)
   - Extract publish date from meta tags (article:published_time, published_time)
   - Handle timeout settings (configurable, default 10 seconds)

5. **Add required dependencies** to Cargo.toml:
   - reqwest with default features
   - scraper for HTML parsing
   - Additional error handling dependencies as needed

6. **Write comprehensive tests** using TDD approach:
   - Test successful metadata extraction
   - Test timeout handling
   - Test malformed HTML handling
   - Test network error scenarios
   - Test missing metadata fields
   - Use mock HTTP responses for testing

## Success Criteria
- MetadataExtractor trait is well-defined and async
- WebExtractor successfully extracts common metadata
- Error handling is robust for network and parsing issues
- Tests cover all error scenarios
- Configurable timeout works properly