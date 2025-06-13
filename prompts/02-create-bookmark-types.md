# Task 2: Create Basic Bookmark Domain Types

## Objective
Implement the core Bookmark struct and related types for the MVP using TDD.

## Requirements

1. **Create Bookmark struct** in `src/types/bookmark.rs`:
   - Fields: id (String), url (String), title (String), bookmarked_date (DateTime<Utc>)
   - Add serde derives for serialization
   - Implement `new()` method with URL and title validation
   - Auto-generate UUID for id and current timestamp for date

2. **Create BookmarkError enum** in `src/types/mod.rs`:
   - Add thiserror dependency
   - Error variants: InvalidUrl, NotFound, EmptyTitle
   - Create BookmarkResult<T> type alias
   - Implement proper error messages

3. **Write comprehensive tests** for:
   - Valid bookmark creation
   - URL validation (reject invalid URLs)
   - Title validation (reject empty titles)
   - ID uniqueness
   - Serialization/deserialization

4. **Update module exports** to make types accessible from main

## Success Criteria
- All tests pass using TDD approach
- Bookmark creation works with valid inputs
- Invalid inputs return appropriate errors
- Types are properly exported and usable