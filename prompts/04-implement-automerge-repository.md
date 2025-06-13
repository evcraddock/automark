# Task 4: Implement Automerge Repository Adapter

## Objective
Implement the BookmarkRepository trait using Automerge for CRDT-based local storage.

## Requirements

1. **Create AutomergeBookmarkRepository struct** in `src/adapters/automerge_repo.rs`:
   - Fields: Automerge document and file path
   - Constructor that loads existing document or creates new one
   - Save method that persists document to file
   - Handle directory creation for parent paths

2. **Implement BookmarkRepository trait**:
   - create(): Insert bookmark into Automerge map, save to file
   - find_all(): Extract all bookmarks from document
   - delete(): Remove bookmark by ID, return NotFound if missing
   - Use bookmark ID as map key, serialize bookmark as Automerge value

3. **Add file I/O handling**:
   - Create parent directories if needed
   - Handle missing files gracefully (create new document)
   - Use atomic writes (temp file + rename)
   - Convert Automerge errors to BookmarkError types

4. **Write comprehensive tests**:
   - Test persistence across repository instances
   - Test loading existing data
   - Test error handling for corrupted files
   - Test all CRUD operations with file persistence
   - Use temporary directories for testing

5. **Update module exports** to make repository accessible

## Success Criteria
- All tests pass with real file operations
- Data persists between application restarts
- Error handling is robust
- Automerge integration works correctly
- Repository follows the established trait interface