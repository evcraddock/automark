# Task 8: Implement Sync Command for Document Synchronization

**GitHub Issue**: [#8](https://github.com/evcraddock/automark/issues/8)

## Objective
Create sync command for merging Automerge documents between devices using file-based synchronization.

## Requirements

1. **Create sync command structure** in `src/commands/sync.rs`:
   - SyncArgs struct with source file path
   - Support for bidirectional synchronization
   - Options for merge conflict reporting
   - Dry-run mode for preview operations

2. **Implement SyncCommand**:
   - Load remote Automerge document from file path
   - Merge remote document with local document
   - Save merged result to local storage
   - Report synchronization statistics

3. **Define sync argument patterns**:
   - Required positional argument: path to remote document
   - --dry-run flag for preview mode
   - --force flag for overwriting conflicts
   - --backup flag for creating backup before sync
   - JSON output support for automation

4. **Implement document merging**:
   - Use Automerge merge capabilities
   - Handle merge conflicts according to CRDT semantics
   - Preserve causal ordering during merge
   - Validate document compatibility before merge

5. **Add sync safety features**:
   - Create backup before sync operations
   - Validate remote document format
   - Handle corrupted or incompatible documents
   - Provide rollback capability on failure

6. **Implement sync reporting**:
   - Count of added, modified, deleted bookmarks
   - Merge conflict detection and reporting
   - Document version information before/after
   - Operation timing and performance metrics

7. **Error handling for sync operations**:
   - Handle missing or inaccessible remote files
   - Validate document format compatibility
   - Handle network/file system errors gracefully
   - Provide recovery suggestions for failures

8. **Write comprehensive tests** using TDD approach:
   - Test successful document merging
   - Test conflict resolution scenarios
   - Test error handling for invalid documents
   - Test dry-run mode functionality
   - Test backup and rollback operations

## Success Criteria
- Sync command merges documents correctly
- CRDT semantics are preserved during merge
- Error handling is robust and informative
- Both JSON and human output work properly
- Backup and recovery functions work reliably