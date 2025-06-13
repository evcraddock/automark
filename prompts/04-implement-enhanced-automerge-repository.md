# Task 4: Implement Enhanced Automerge Repository with CRDT Operations

**GitHub Issue**: [#4](https://github.com/evcraddock/automark/issues/4)

## Objective
Implement the full BookmarkRepository trait using Automerge with proper CRDT semantics for all data operations.

## Requirements

1. **Enhance AutomergeBookmarkRepository** in `src/adapters/automerge_repo.rs`:
   - Implement all new repository trait methods
   - Use Automerge map for bookmark storage (ID as key)
   - Use Automerge sequences for collections (tags, notes)
   - Implement field-level updates to preserve concurrent modifications

2. **Implement CRDT-specific data operations**:
   - Store tags as Automerge list for proper CRDT merging
   - Store notes as sequence of immutable objects
   - Use Automerge counters for timestamps when appropriate
   - Implement proper delete semantics with tombstone markers

3. **Implement search and filtering**:
   - search_by_text: Full-text search across all text fields
   - find_by_tags: Tag intersection logic
   - Filtering: Apply BookmarkFilters to find_all results
   - Optimize search performance for large datasets

4. **Handle Automerge document structure**:
   - Root document as map of bookmark IDs to bookmark objects
   - Each bookmark as nested map with scalar and collection fields
   - Maintain document consistency across operations
   - Handle document loading and saving efficiently

5. **Implement atomic operations**:
   - Ensure all mutations create new document states
   - Batch related operations into single commits
   - Handle partial update failures gracefully
   - Maintain causal ordering for operations

6. **Error handling and validation**:
   - Convert Automerge errors to BookmarkError types
   - Validate document structure on load
   - Handle corrupted document recovery
   - Implement robust file I/O with atomic writes

7. **Write comprehensive tests** using TDD approach:
   - Test all repository operations with real Automerge documents
   - Test concurrent modification scenarios
   - Test document persistence and loading
   - Test search and filtering accuracy
   - Test error recovery scenarios
   - Use temporary files for testing

## Success Criteria
- All repository operations work with CRDT semantics
- Search and filtering perform efficiently
- Document structure is consistent and recoverable
- Concurrent modifications merge correctly
- All tests pass with real file persistence