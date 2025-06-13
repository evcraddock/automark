# Task 8: Implement Delete Command

## Objective
Implement the delete command that removes a bookmark by its ID from the repository.

## Requirements

1. **Create DeleteCommand struct** in `src/commands/delete.rs`:
   - Store DeleteArgs in the struct
   - Implement CommandHandler trait
   - Support both full ID and partial ID matching (first 8 chars)
   - Show confirmation of what was deleted

2. **Implement ID matching logic**:
   - Try exact ID match first
   - If no exact match and input is ≤8 chars, try partial match
   - Return error if partial ID matches multiple bookmarks
   - Return NotFound if no matches

3. **Add new error type**:
   - Add InvalidId variant to BookmarkError enum
   - Use for ambiguous partial ID matches

4. **Create factory function**:
   - `handle_delete_command()` that creates DeleteCommand and executes it
   - Takes DeleteArgs and repository reference

5. **Write comprehensive tests** using TDD:
   - Test deleting with full ID
   - Test deleting with unique partial ID
   - Test error when partial ID matches multiple bookmarks
   - Test error when ID doesn't exist
   - Test success message format
   - Use mock repository

6. **Display confirmation**:
   - Show title, URL, and full ID of deleted bookmark
   - Confirm the deletion occurred

## Success Criteria
- All tests pass following TDD approach
- Full and partial ID deletion work correctly
- Ambiguous partial IDs return helpful errors
- User sees confirmation of what was deleted
- Repository integration works properly
- Error handling is comprehensive