# Task 3: Create BookmarkRepository Trait

## Objective
Define the BookmarkRepository trait that abstracts bookmark storage operations for the MVP.

## Requirements

1. **Create async trait** in `src/traits/repository.rs`:
   - Add async-trait dependency
   - Define three methods: create, find_all, delete
   - Use BookmarkResult for return types
   - Methods should be async

2. **Create mock implementation** for testing:
   - In-memory HashMap storage
   - Implement the trait for the mock
   - Handle NotFound errors for missing IDs
   - Use #[cfg(test)] attribute

3. **Write tests** using the mock implementation:
   - Test creating bookmarks
   - Test retrieving all bookmarks
   - Test deleting existing bookmarks
   - Test error handling for non-existent IDs

4. **Update module exports** to make trait accessible

## Success Criteria
- Trait compiles and tests pass
- Mock implementation works correctly
- Async operations function properly
- Error handling behaves as expected
- Interface is clean and minimal for MVP needs