# Task 7: Implement List Command

## Objective
Implement the list command that displays all bookmarks in a user-friendly format.

## Requirements

1. **Create ListCommand struct** in `src/commands/list.rs`:
   - Implement CommandHandler trait
   - Retrieve all bookmarks from repository
   - Format and display bookmarks in readable format
   - Handle empty repository case with helpful message

2. **Implement formatting methods**:
   - Format individual bookmarks (title, URL, date)
   - Format bookmark list with numbering and partial IDs (first 8 chars)
   - Use consistent date formatting
   - Show bookmark count in header

3. **Create factory function**:
   - `handle_list_command()` that creates and executes ListCommand
   - Takes repository reference and returns BookmarkResult<()>

4. **Write comprehensive tests** using TDD:
   - Test listing multiple bookmarks
   - Test empty repository handling
   - Test output formatting
   - Test date formatting consistency
   - Test ID truncation
   - Use mock repository

5. **Define output format**:
   - Show: "Found X bookmark(s):"
   - Number each bookmark: "1. [partial-id] Title"
   - Include URL and formatted date
   - Empty case: helpful message with usage hint

## Success Criteria
- All tests pass following TDD approach
- Output is readable and well-formatted
- Empty repository handled gracefully
- Partial IDs shown for use with delete command
- Repository errors propagated correctly