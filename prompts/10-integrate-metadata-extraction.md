# Task 10: Integrate Metadata Extraction into Add Command

**GitHub Issue**: [#10](https://github.com/evcraddock/automark/issues/10)

## Objective
Enhance the add command to automatically extract metadata from URLs and integrate with the MetadataExtractor.

## Requirements

1. **Update add command** to use MetadataExtractor:
   - Automatically fetch metadata when adding bookmarks
   - Allow manual override of extracted metadata
   - Support --no-fetch flag to skip metadata extraction
   - Handle extraction failures gracefully

2. **Enhance AddArgs structure**:
   - Add optional --title flag to override extracted title
   - Add optional --author flag to manually specify author
   - Add optional --tags flag for initial tags
   - Add --no-fetch flag to disable metadata extraction

3. **Implement metadata integration workflow**:
   - Extract metadata first if not disabled
   - Use extracted title if no manual title provided
   - Merge extracted metadata with manual overrides
   - Display extraction results to user

4. **Add timeout and error handling**:
   - Configurable timeout for metadata extraction
   - Graceful fallback when extraction fails
   - User-friendly error messages for network issues
   - Continue bookmark creation even if extraction fails

5. **Update command output**:
   - Show extracted metadata in confirmation message
   - Indicate when metadata extraction was skipped
   - Display extraction time and success status
   - Include metadata in JSON output format

6. **Handle edge cases**:
   - URLs that don't return HTML content
   - Pages with missing or malformed metadata
   - Network connectivity issues
   - Very slow or unresponsive websites

7. **Add configuration for metadata extraction**:
   - Default timeout setting in config file
   - Option to disable extraction by default
   - User agent string configuration
   - Retry policy for failed extractions

8. **Write comprehensive tests** using TDD approach:
   - Test successful metadata extraction and integration
   - Test manual override functionality
   - Test extraction failure handling
   - Test timeout scenarios
   - Test --no-fetch flag behavior
   - Use mock HTTP responses for testing

## Success Criteria
- Metadata extraction works automatically for most URLs
- Manual overrides function correctly
- Extraction failures don't prevent bookmark creation
- Both JSON and human output include metadata info
- Configuration options work as expected