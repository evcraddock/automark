# Task 3: Enhance Repository Trait for Full CRUD and Search Operations

**GitHub Issue**: [#3](https://github.com/evcraddock/automark/issues/3)

## Objective
Expand the BookmarkRepository trait to support full CRUD operations, search, and filtering capabilities.

## Requirements

1. **Enhance BookmarkRepository trait** in `src/traits/repository.rs`:
   - Add methods: find_by_id, update, search_by_text, find_by_tags
   - Modify find_all to accept optional BookmarkFilters parameter
   - Ensure all methods are async and return BookmarkResult types
   - Add method for adding/removing notes from bookmarks

2. **Define search and filter operations**:
   - search_by_text: Search in title, URL, author, and note content
   - find_by_tags: Find bookmarks containing specific tags (AND logic)
   - find_all with filters: Support filtering by reading status, priority range, date range
   - All search operations should be case-insensitive

3. **Update mock implementation** for testing:
   - Implement all new trait methods in mock
   - Use in-memory storage for testing
   - Handle filtering and search logic properly
   - Maintain existing test compatibility

4. **Define CRDT-specific requirements**:
   - Update operations must preserve concurrent modifications
   - Collections (tags, notes) use merge semantics
   - Document field-level update strategy
   - Handle delete operations with tombstone markers

5. **Add comprehensive method documentation**:
   - Document expected behavior for each method
   - Specify error conditions and return types
   - Document CRDT-specific behavior patterns
   - Include examples in documentation

6. **Write comprehensive tests** using TDD approach:
   - Test all CRUD operations
   - Test search functionality with various inputs
   - Test filtering with different criteria combinations
   - Test edge cases (empty results, invalid filters)
   - Test concurrent modification scenarios

## Success Criteria
- Repository trait supports full bookmark lifecycle
- Search and filtering work correctly
- Mock implementation passes all tests
- CRDT requirements are clearly specified
- Documentation is comprehensive and clear