# Task 9: Wire Everything Together in Main

## Objective
Connect all components in main.rs to create a working MVP application.

## Requirements

1. **Update main.rs** to use real components:
   - Initialize AutomergeBookmarkRepository with data file path
   - Use hardcoded path: `~/.local/share/automark/bookmarks.automerge`
   - Create data directory if it doesn't exist
   - Replace placeholder commands with actual command handlers

2. **Implement repository initialization**:
   - Use dirs crate to get user data directory
   - Create AutomergeBookmarkRepository instance
   - Handle initialization errors gracefully
   - Add dirs dependency if not already present

3. **Connect command handlers**:
   - Call handle_add_command for Add variant
   - Call handle_list_command for List variant  
   - Call handle_delete_command for Delete variant
   - Propagate errors to main error handling

4. **Add error handling**:
   - Convert BookmarkError to appropriate exit codes
   - Display user-friendly error messages
   - Handle repository initialization failures

5. **Write integration tests**:
   - Test full command flow with temporary directories
   - Test data persistence across runs
   - Test error scenarios
   - Use real Automerge repository for tests

6. **Verify complete functionality**:
   - Test adding bookmarks
   - Test listing bookmarks
   - Test deleting bookmarks
   - Test data persistence between runs

## Success Criteria
- MVP application is fully functional
- Data persists between application runs
- All commands work end-to-end
- Error handling is robust
- Integration tests pass
- Application can be built and run successfully