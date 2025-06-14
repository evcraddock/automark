# Task 5: Enhance CLI Commands with JSON Output Support

**GitHub Issue**: [#5](https://github.com/evcraddock/automark/issues/5)

## Objective
Add JSON output formatting to all CLI commands and implement the global --json flag for machine-readable output.

## Requirements

1. **Enhance CLI structure** in `src/commands/mod.rs`:
   - Add global --json flag to Cli struct
   - Create OutputFormat enum (Human, Json)
   - Add output formatting helper functions
   - Ensure JSON output goes to stdout, errors to stderr (or stdout in JSON mode)

2. **Implement JSON serialization** for all types:
   - Add serde derives to all domain types
   - Create consistent JSON schema for responses
   - Implement JSON error format with error codes
   - Add schema versioning for API compatibility

3. **Enhance add command** in `src/commands/add.rs`:
   - Support --json flag for machine-readable output
   - Return created bookmark in JSON format when requested
   - Include metadata extraction results in JSON output
   - Maintain human-readable success messages for default mode

4. **Enhance list command** in `src/commands/list.rs`:
   - Support --json flag for structured bookmark listing
   - Include all bookmark fields in JSON output
   - Implement pagination info in JSON responses
   - Format human output as readable tables

5. **Enhance delete command** in `src/commands/delete.rs`:
   - Support --json flag for deletion confirmation
   - Return deleted bookmark details in JSON format
   - Include operation status and affected record count
   - Maintain confirmation messages for human output

6. **Implement consistent error formatting**:
   - Create standardized JSON error schema
   - Map BookmarkError variants to error codes
   - Include contextual details in error responses
   - Ensure human errors go to stderr, JSON errors to stdout

7. **Add output format utilities**:
   - Create formatter modules for human vs JSON output
   - Implement table formatting for human-readable lists
   - Add JSON pretty-printing option
   - Handle output routing correctly

8. **Write comprehensive tests** using TDD approach:
   - Test JSON output format for all commands
   - Test error JSON formatting
   - Test output routing (stdout vs stderr)
   - Test schema consistency
   - Test both human and JSON modes

## Success Criteria
- All commands support --json flag correctly
- JSON output is well-structured and consistent
- Error handling works in both output modes
- Human output remains user-friendly
- All tests pass for both output formats