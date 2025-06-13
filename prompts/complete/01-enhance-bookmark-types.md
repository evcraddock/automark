# Task 1: Enhance Bookmark Types with Full Domain Model

**GitHub Issue**: [#1](https://github.com/evcraddock/automark/issues/1)

## Objective
Expand the basic Bookmark struct to include all domain features specified in the technical architecture.

## Requirements

1. **Enhance Bookmark struct** in `src/types/bookmark.rs`:
   - Add fields: author (Option<String>), tags (Vec<String>), publish_date (Option<DateTime<Utc>>), notes (Vec<Note>), reading_status (ReadingStatus), priority_rating (Option<u8>)
   - Implement tag normalization (lowercase conversion)
   - Add priority validation (1-5 range)
   - Maintain existing fields: id, url, title, bookmarked_date

2. **Create Note struct** in `src/types/bookmark.rs`:
   - Fields: id (String), content (String), created_at (DateTime<Utc>)
   - Implement immutable design (no update methods)
   - Add constructor with auto-generated ID and timestamp

3. **Create supporting enums** in `src/types/bookmark.rs`:
   - ReadingStatus enum: Unread, Reading, Completed
   - SortOrder enum: PublishDate, BookmarkedDate, Title

4. **Create BookmarkFilters struct** in `src/types/bookmark.rs`:
   - Fields for filtering: text_query, tags, reading_status, priority_range
   - All fields optional for flexible filtering

5. **Update validation**:
   - Extend existing URL and title validation
   - Add priority rating validation (1-5 if provided)
   - Implement tag normalization in constructor
   - Add comprehensive validation tests

6. **Write comprehensive tests** using TDD approach:
   - Test all new field validations
   - Test tag normalization
   - Test Note immutability
   - Test enum serialization/deserialization
   - Test BookmarkFilters construction

## Success Criteria
- All tests pass following TDD approach
- Enhanced Bookmark supports full domain model
- Validation rules properly enforced
- Types are serializable for JSON output
- Note immutability is maintained